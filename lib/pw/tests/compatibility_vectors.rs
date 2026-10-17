use secret_sorbet_pw::{
    ALGORITHM_VERSION, CharacterClass, Encoder, KDF_PROFILE, LoginInput, OutputPolicy, Variable,
    generate_login_password,
};
use serde::Deserialize;
use std::collections::HashSet;

const LOGIN_VECTORS: &str = include_str!("../../../test-vectors/default-login-v1.json");
const SYNTHETIC_VECTORS: &str = include_str!("../../../test-vectors/synthetic-login-v1.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntheticSuite {
    schema_version: u32,
    algorithm: String,
    profile: String,
    kdf: KdfVector,
    provenance: Provenance,
    cases: Vec<SyntheticCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    generator: String,
    reference: String,
    inputs: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SyntheticCase {
    id: String,
    raw: RawLogin,
    effective_variables: Vec<[String; 2]>,
    policy: PolicyVector,
    output: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLogin {
    passphrase: String,
    service: String,
    username: String,
    counter: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityVector {
    algorithm: String,
    profile: String,
    #[serde(default)]
    kdf: Option<KdfVector>,
    passphrase: String,
    variables: Vec<[String; 2]>,
    policy: PolicyVector,
    output: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KdfVector {
    algorithm: String,
    version: u32,
    memory_kib: u32,
    iterations: u32,
    lanes: u32,
    output_length: usize,
    salt_derivation: String,
    salt_domain: String,
    salt_length: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyVector {
    length: usize,
    classes: Vec<CharacterClassVector>,
    #[serde(deserialize_with = "explicit_allowed")]
    allowed: Option<String>,
}

// `null` explicitly selects union mode. A missing field must not silently do so.
fn explicit_allowed<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterClassVector {
    name: String,
    chars: String,
    min_count: usize,
}

impl PolicyVector {
    fn encoder(&self) -> Encoder {
        Encoder::new(OutputPolicy {
            length: self.length,
            classes: self
                .classes
                .iter()
                .map(|class| CharacterClass::new(&class.name, &class.chars, class.min_count))
                .collect(),
            allowed: self.allowed.clone(),
        })
    }
}

fn validate_case_ids(suite: &SyntheticSuite) -> Result<(), &'static str> {
    if suite.cases.is_empty() {
        return Err("synthetic suite must contain cases");
    }
    let mut ids = HashSet::new();
    for case in &suite.cases {
        if case.id.trim().is_empty() {
            return Err("synthetic case ID must not be empty");
        }
        if !ids.insert(case.id.as_str()) {
            return Err("synthetic case IDs must be unique");
        }
    }
    Ok(())
}

#[test]
fn synthetic_raw_login_vectors_pin_the_public_derivation_contract() {
    let suite: SyntheticSuite = serde_json::from_str(SYNTHETIC_VECTORS)
        .expect("synthetic compatibility vectors must have the strict v1 schema");
    assert_eq!(suite.schema_version, 1);
    assert_eq!(suite.algorithm, ALGORITHM_VERSION);
    assert_eq!(suite.profile, "secret-sorbet-login-v1");
    assert_kdf_metadata(Some(&suite.kdf));
    assert_eq!(
        suite.provenance.generator,
        "test-vectors/generate_synthetic.py"
    );
    assert_eq!(
        suite.provenance.reference,
        "Python 3 stdlib + libargon2 argon2id_hash_raw (v19)"
    );
    assert_eq!(suite.provenance.inputs, "public synthetic test data only");
    validate_case_ids(&suite).unwrap();
    // Bound expensive Argon2 work and allocations on emulated targets. Each
    // case runs in this single test, never as parallel per-case tests.
    assert!(suite.cases.len() <= 16);
    for case in &suite.cases {
        assert!((1..=512).contains(&case.policy.length), "{}", case.id);
        let mut input =
            LoginInput::new(&case.raw.passphrase, &case.raw.service, &case.raw.username);
        input.counter.clone_from(&case.raw.counter);
        input.encoder = case.policy.encoder();
        let expected_variables: Vec<Variable> = case
            .effective_variables
            .iter()
            .map(|[name, value]| Variable::new(name, value))
            .collect();
        assert_eq!(input.variables(), expected_variables, "{}", case.id);
        assert_eq!(
            generate_login_password(&input).expect("synthetic fixture must be valid"),
            case.output,
            "{}",
            case.id
        );
    }
}

#[test]
fn synthetic_fixture_schema_rejects_unknown_fields_at_every_object_level() {
    let original: serde_json::Value = serde_json::from_str(SYNTHETIC_VECTORS).unwrap();
    for pointer in [
        "",
        "/kdf",
        "/provenance",
        "/cases/0",
        "/cases/0/raw",
        "/cases/0/policy",
        "/cases/0/policy/classes/0",
    ] {
        let mut changed = original.clone();
        changed
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("unexpected".to_owned(), serde_json::Value::Null);
        assert!(
            serde_json::from_value::<SyntheticSuite>(changed).is_err(),
            "{pointer}"
        );
    }
    let mut legacy: serde_json::Value = serde_json::from_str(LOGIN_VECTORS).unwrap();
    legacy[0]
        .as_object_mut()
        .unwrap()
        .insert("unexpected".to_owned(), serde_json::Value::Null);
    assert!(serde_json::from_value::<Vec<CompatibilityVector>>(legacy).is_err());
}

#[test]
fn synthetic_fixture_requires_nonempty_cases_and_nonempty_unique_ids() {
    let mut suite: SyntheticSuite = serde_json::from_str(SYNTHETIC_VECTORS).unwrap();
    suite.cases[0].id = suite.cases[1].id.clone();
    assert_eq!(
        validate_case_ids(&suite),
        Err("synthetic case IDs must be unique")
    );
    suite.cases[0].id = " \t\n".to_owned();
    assert_eq!(
        validate_case_ids(&suite),
        Err("synthetic case ID must not be empty")
    );
    suite.cases.clear();
    assert_eq!(
        validate_case_ids(&suite),
        Err("synthetic suite must contain cases")
    );
}

#[test]
fn synthetic_fixture_schema_requires_every_declared_field_including_allowed_mode() {
    let original: serde_json::Value = serde_json::from_str(SYNTHETIC_VECTORS).unwrap();
    for pointer in [
        "",
        "/kdf",
        "/provenance",
        "/cases/0",
        "/cases/0/raw",
        "/cases/0/policy",
        "/cases/0/policy/classes/0",
    ] {
        for field in original
            .pointer(pointer)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
        {
            let mut changed = original.clone();
            changed
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(
                serde_json::from_value::<SyntheticSuite>(changed).is_err(),
                "missing synthetic field {pointer}/{field}"
            );
        }
    }
}

#[test]
fn login_validation_and_core_validation_reject_their_own_invalid_inputs() {
    use secret_sorbet_pw::{CoreError, Error, validate_login_input};

    for (service, username, expected) in [
        ("", "synthetic", Error::EmptyService),
        ("\u{00a0}\t", "synthetic", Error::EmptyService),
        ("example", "synthetic", Error::InvalidService),
        ("exa\u{2003}mple.test", "synthetic", Error::InvalidService),
        ("example.test", "", Error::EmptyUsername),
        ("example.test", "\u{3000}\n", Error::EmptyUsername),
    ] {
        let input = LoginInput::new("synthetic", service, username);
        assert_eq!(validate_login_input(&input), Err(expected.clone()));
        assert_eq!(generate_login_password(&input), Err(expected));
    }
    for counter in ["", " \t\n", "\u{0085}\u{00a0}\u{2003}\u{3000}"] {
        let mut input = LoginInput::new("synthetic", "example.test", "synthetic");
        input.counter = counter.to_owned();
        assert_eq!(validate_login_input(&input), Ok(()));
        assert_eq!(
            generate_login_password(&input),
            Err(Error::Core(CoreError::EmptyVariableValue {
                name: "counter".to_owned()
            }))
        );
    }
    let input = LoginInput::new("", "example.test", "synthetic");
    assert_eq!(validate_login_input(&input), Ok(()));
    assert_eq!(
        generate_login_password(&input),
        Err(Error::Core(CoreError::EmptySecret))
    );
}

#[test]
fn checked_in_login_vectors_are_the_compatibility_contract() {
    let vectors: Vec<CompatibilityVector> =
        serde_json::from_str(LOGIN_VECTORS).expect("compatibility vectors must be valid JSON");
    assert!(
        !vectors.is_empty(),
        "at least one vector must be checked in"
    );

    for vector in vectors {
        assert_eq!(vector.profile, "secret-sorbet-login-v1");
        assert_eq!(vector.algorithm, ALGORITHM_VERSION);
        assert_kdf_metadata(vector.kdf.as_ref());

        let variables: Vec<Variable> = vector
            .variables
            .iter()
            .map(|pair| Variable::new(&pair[0], &pair[1]))
            .collect();
        assert_eq!(variables[0], Variable::new("purpose", "login"));

        let service = variable_value(&vector.variables, "service");
        let username = variable_value(&vector.variables, "username");
        let counter = variable_value(&vector.variables, "counter");
        let mut input = LoginInput::new(&vector.passphrase, service, username);
        input.counter = counter.to_owned();
        input.encoder = Encoder::new(OutputPolicy {
            length: vector.policy.length,
            classes: vector
                .policy
                .classes
                .into_iter()
                .map(|class| CharacterClass::new(class.name, class.chars, class.min_count))
                .collect(),
            allowed: vector.policy.allowed,
        });

        assert_eq!(input.variables(), variables);
        assert_eq!(
            generate_login_password(&input).expect("vector inputs must be valid"),
            vector.output,
            "compatibility vector failed for {}",
            vector.algorithm
        );
    }
}

fn variable_value<'a>(variables: &'a [[String; 2]], name: &str) -> &'a str {
    variables
        .iter()
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
        .unwrap_or_else(|| panic!("compatibility vector is missing variable {name}"))
}

fn assert_kdf_metadata(vector: Option<&KdfVector>) {
    let vector = vector.expect("the supported algorithm must declare its memory-hard KDF");
    assert_eq!(vector.algorithm, KDF_PROFILE.algorithm);
    assert_eq!(vector.version, KDF_PROFILE.version);
    assert_eq!(vector.memory_kib, KDF_PROFILE.memory_kib);
    assert_eq!(vector.iterations, KDF_PROFILE.iterations);
    assert_eq!(vector.lanes, KDF_PROFILE.lanes);
    assert_eq!(vector.output_length, KDF_PROFILE.output_length);
    assert_eq!(vector.salt_derivation, KDF_PROFILE.salt_derivation);
    assert_eq!(vector.salt_domain, KDF_PROFILE.salt_domain);
    assert_eq!(vector.salt_length, KDF_PROFILE.salt_length);
}

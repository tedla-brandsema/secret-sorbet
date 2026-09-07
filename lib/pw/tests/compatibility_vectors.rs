use secret_sorbet_pw::{
    ALGORITHM_VERSION, CharacterClass, Encoder, KDF_PROFILE, LoginInput, OutputPolicy, Variable,
    generate_login_password,
};
use serde::Deserialize;

const LOGIN_VECTORS: &str = include_str!("../../../test-vectors/default-login-v1.json");

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
    allowed: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CharacterClassVector {
    name: String,
    chars: String,
    min_count: usize,
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

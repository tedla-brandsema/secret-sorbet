mod context;
mod encoder;
mod error;
mod kdf;
mod policy;
mod stream;
mod variable;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod compatibility_fixture {
    include!("../../../test-vectors/synthetic-core-v1.rs");

    pub(crate) fn variables() -> Vec<crate::Variable> {
        vec![
            crate::Variable::new("名", "é🧪"),
            crate::Variable::new("counter", "1"),
        ]
    }

    pub(crate) fn policy() -> crate::OutputPolicy {
        crate::OutputPolicy {
            length: 10,
            classes: vec![crate::CharacterClass::new("文字", "aab12", 2)],
            allowed: None,
        }
    }
}

pub use encoder::{ByteSource, Encoder};
pub use error::Error;
pub use policy::{CharacterClass, OutputPolicy};
pub use variable::Variable;

use context::canonical_context;
use stream::HmacByteStream;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KdfProfile {
    pub algorithm: &'static str,
    pub version: u32,
    pub memory_kib: u32,
    pub iterations: u32,
    pub lanes: u32,
    pub output_length: usize,
    pub salt_derivation: &'static str,
    pub salt_domain: &'static str,
    pub salt_length: usize,
}

pub const KDF_PROFILE: KdfProfile = KdfProfile {
    algorithm: "argon2id",
    version: 19,
    memory_kib: 65_536,
    iterations: 3,
    lanes: 4,
    output_length: 32,
    salt_derivation: "sha256-domain-context-v1",
    salt_domain: "secret-sorbet-core-v1:argon2id:salt",
    salt_length: 16,
};

pub const ALGORITHM_VERSION: &str = "secret-sorbet-core-v1";

pub fn derive_with_encoder(
    secret: &str,
    variables: &[Variable],
    encoder: &Encoder,
) -> Result<String, Error> {
    validate_secret_and_variables(secret, variables)?;
    policy::validate_policy(&encoder.policy)?;

    let context = canonical_context(variables, &encoder.policy);
    let key = kdf::derive_key(secret.as_bytes(), &context)?;
    let mut stream = HmacByteStream::new(key.as_ref(), &context)?;
    encoder.encode(&mut stream)
}

pub fn derive(
    secret: &str,
    variables: &[Variable],
    policy: &OutputPolicy,
) -> Result<String, Error> {
    derive_with_encoder(secret, variables, &Encoder::new(policy.clone()))
}

pub fn default_encoder() -> Encoder {
    Encoder::new(default_password_policy())
}

pub fn default_password_policy() -> OutputPolicy {
    OutputPolicy {
        length: 20,
        classes: vec![
            CharacterClass::new("lowercase", "abcdefghijklmnopqrstuvwxyz", 1),
            CharacterClass::new("uppercase", "ABCDEFGHIJKLMNOPQRSTUVWXYZ", 1),
            CharacterClass::new("digit", "0123456789", 1),
            CharacterClass::new("special", "!@#$%^&*()-_+=<>?", 1),
        ],
        allowed: None,
    }
}

fn validate_secret_and_variables(secret: &str, variables: &[Variable]) -> Result<(), Error> {
    if secret.is_empty() {
        return Err(Error::EmptySecret);
    }
    for variable in variables {
        if variable.name.is_empty() {
            return Err(Error::EmptyVariableName);
        }
        if variable.value.is_empty() {
            return Err(Error::EmptyVariableValue {
                name: variable.name.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_core_outputs_pin_variable_order_and_empty_list_acceptance() {
        use compatibility_fixture as fixture;

        let mut variables = fixture::variables();
        let policy = fixture::policy();
        assert_eq!(
            derive("Synthetic core 🔐", &variables, &policy).unwrap(),
            fixture::OUTPUT
        );
        variables.reverse();
        assert_eq!(
            derive("Synthetic core 🔐", &variables, &policy).unwrap(),
            fixture::REORDERED_OUTPUT
        );
        assert_eq!(
            derive("Synthetic core 🔐", &[], &policy).unwrap(),
            fixture::EMPTY_VARIABLES_OUTPUT
        );
    }

    #[test]
    fn core_rejects_empty_secret_and_fields_but_accepts_empty_variable_lists() {
        assert_eq!(
            validate_secret_and_variables("", &[]),
            Err(Error::EmptySecret)
        );
        assert_eq!(validate_secret_and_variables(" ", &[]), Ok(()));
        assert_eq!(
            validate_secret_and_variables("synthetic", &[Variable::new("", "value")]),
            Err(Error::EmptyVariableName)
        );
        assert_eq!(
            validate_secret_and_variables("synthetic", &[Variable::new("synthetic-name", "")]),
            Err(Error::EmptyVariableValue {
                name: "synthetic-name".to_owned()
            })
        );
    }

    #[test]
    fn policy_validation_rejects_invalid_charsets_counts_and_overflow_without_deriving() {
        let cases = [
            (
                OutputPolicy {
                    length: 0,
                    classes: vec![],
                    allowed: Some("a".into()),
                },
                Error::InvalidLength,
            ),
            (
                OutputPolicy {
                    length: 1,
                    classes: vec![],
                    allowed: None,
                },
                Error::EmptyCharset {
                    name: "allowed".into(),
                },
            ),
            (
                OutputPolicy {
                    length: 1,
                    classes: vec![],
                    allowed: Some("".into()),
                },
                Error::EmptyCharset {
                    name: "allowed".into(),
                },
            ),
            (
                OutputPolicy {
                    length: 1,
                    classes: vec![],
                    allowed: Some("é".into()),
                },
                Error::NonAsciiCharset {
                    name: "allowed".into(),
                },
            ),
            (
                OutputPolicy {
                    length: 1,
                    classes: vec![CharacterClass::new("synthetic", "", 0)],
                    allowed: Some("a".into()),
                },
                Error::EmptyCharset {
                    name: "synthetic".into(),
                },
            ),
            (
                OutputPolicy {
                    length: 1,
                    classes: vec![CharacterClass::new("synthetic", "🧪", 0)],
                    allowed: Some("a".into()),
                },
                Error::NonAsciiCharset {
                    name: "synthetic".into(),
                },
            ),
            (
                OutputPolicy {
                    length: 1,
                    classes: vec![CharacterClass::new("synthetic", "a", 2)],
                    allowed: None,
                },
                Error::RequiredCountExceedsLength,
            ),
            (
                OutputPolicy {
                    length: usize::MAX,
                    classes: vec![
                        CharacterClass::new("first", "a", usize::MAX),
                        CharacterClass::new("second", "a", 1),
                    ],
                    allowed: None,
                },
                Error::RequiredCountExceedsLength,
            ),
            (
                OutputPolicy {
                    length: 1,
                    classes: vec![CharacterClass::new("synthetic", "b", 0)],
                    allowed: Some("a".into()),
                },
                Error::RequiredClassNotAllowed {
                    name: "synthetic".into(),
                    ch: 'b',
                },
            ),
        ];
        for (policy, expected) in cases {
            assert_eq!(policy::validate_policy(&policy), Err(expected));
        }
        // Class labels are context metadata: unlike charsets they may be empty
        // or non-ASCII. Do not accidentally impose login validation on core.
        let policy = OutputPolicy {
            length: 1,
            classes: vec![
                CharacterClass::new("", "a", 0),
                CharacterClass::new("名", "a", 1),
            ],
            allowed: None,
        };
        assert_eq!(policy::validate_policy(&policy), Ok(()));
    }

    fn login_vars() -> Vec<Variable> {
        vec![
            Variable::new("purpose", "login"),
            Variable::new("service", "github.com"),
            Variable::new("username", "ted@example.com"),
            Variable::new("counter", "1"),
        ]
    }

    #[test]
    fn same_inputs_produce_same_output() {
        let policy = default_password_policy();

        let a = derive("long private passphrase", &login_vars(), &policy).unwrap();
        let b = derive("long private passphrase", &login_vars(), &policy).unwrap();

        assert_eq!(a, b);
        assert_eq!(a.len(), 20);
    }

    #[test]
    fn encoder_derivation_uses_the_supported_algorithm() {
        let encoder = default_encoder();

        let output =
            derive_with_encoder("long private passphrase", &login_vars(), &encoder).unwrap();

        assert_eq!(output.len(), 20);
    }

    #[test]
    fn variable_order_is_part_of_the_contract() {
        let policy = default_password_policy();
        let mut vars = login_vars();
        let original = derive("long private passphrase", &vars, &policy).unwrap();

        vars.swap(1, 2);
        let reordered = derive("long private passphrase", &vars, &policy).unwrap();

        assert_ne!(original, reordered);
    }

    #[test]
    fn default_policy_meets_required_classes() {
        let output = derive(
            "long private passphrase",
            &login_vars(),
            &default_password_policy(),
        )
        .unwrap();

        assert!(output.chars().any(|ch| ch.is_ascii_lowercase()));
        assert!(output.chars().any(|ch| ch.is_ascii_uppercase()));
        assert!(output.chars().any(|ch| ch.is_ascii_digit()));
        assert!(output.chars().any(|ch| "!@#$%^&*()-_+=<>?".contains(ch)));
    }

    #[test]
    fn custom_policy_can_require_repetition() {
        let policy = OutputPolicy {
            length: 12,
            classes: vec![
                CharacterClass::new("digit", "0123456789", 4),
                CharacterClass::new("symbol", "@#", 2),
            ],
            allowed: Some("abcdef0123456789@#".to_string()),
        };

        let output = derive("long private passphrase", &login_vars(), &policy).unwrap();

        assert_eq!(output.len(), 12);
        assert!(output.chars().filter(|ch| ch.is_ascii_digit()).count() >= 4);
        assert!(output.chars().filter(|ch| "@#".contains(*ch)).count() >= 2);
    }

    #[test]
    fn supported_algorithm_is_memory_hard_v1() {
        assert_eq!(ALGORITHM_VERSION, "secret-sorbet-core-v1");
        assert_eq!(KDF_PROFILE.algorithm, "argon2id");

        let output = derive(
            "long private passphrase",
            &login_vars(),
            &default_password_policy(),
        )
        .unwrap();

        assert_eq!(output.len(), 20);
    }
}

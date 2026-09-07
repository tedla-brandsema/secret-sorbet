mod context;
mod encoder;
mod error;
mod kdf;
mod policy;
mod stream;
mod variable;

#[cfg(test)]
mod property_tests;

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

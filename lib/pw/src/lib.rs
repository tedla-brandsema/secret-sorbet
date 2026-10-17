//! Login-specific preparation over the byte-exact core v1 algorithm.
//!
//! Service, username, and counter use Rust Unicode whitespace trimming; only
//! ASCII service letters are lowercased. Passphrases are unchanged. Visually
//! equivalent Unicode spellings can produce different passwords. Preserve the
//! original inputs and profile for recovery; never silently normalize old inputs.
//! The core's memory/thread requirements apply to every generation call.

pub use secret_sorbet_core::{
    ALGORITHM_VERSION, CharacterClass, Encoder, Error as CoreError, KDF_PROFILE, OutputPolicy,
    Variable,
};
use std::fmt;
use zeroize::{ZeroizeOnDrop, Zeroizing};

/// Initial revision string; counters are text, not parsed numbers.
pub const DEFAULT_COUNTER: &str = "1";

/// Failures returned by password-specific input validation and generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Core(CoreError),
    EmptyService,
    EmptyUsername,
    InvalidService,
    InvalidEmail,
}

impl From<CoreError> for Error {
    fn from(value: CoreError) -> Self {
        Self::Core(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Core(error) => return error.fmt(formatter),
            Self::EmptyService => "the service must not be empty",
            Self::EmptyUsername => "the username must not be empty",
            Self::InvalidService => "the service must be a domain name without whitespace",
            Self::InvalidEmail => "the email address is invalid",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Core(error) => Some(error),
            _ => None,
        }
    }
}

/// Inputs for deterministic login-password generation.
///
/// The passphrase is owned in zeroizing memory, is excluded from debug output,
/// and can only be changed through [`Self::replace_passphrase`]. This type is
/// intentionally not `Clone` so callers cannot duplicate the secret by cloning
/// the surrounding input.
#[derive(PartialEq, Eq)]
pub struct LoginInput {
    passphrase: Zeroizing<String>,
    pub service: String,
    pub username: String,
    pub counter: String,
    pub encoder: Encoder,
}

impl LoginInput {
    /// Own a passphrase in zeroizing storage with default counter and policy.
    ///
    /// Inputs are stored verbatim; preparation and validation occur later. The
    /// passphrase is never trimmed or Unicode-normalized. Other fields, including
    /// values exposed by `Debug`, can contain sensitive account metadata.
    pub fn new(
        passphrase: impl Into<String>,
        service: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            passphrase: Zeroizing::new(passphrase.into()),
            service: service.into(),
            username: username.into(),
            counter: DEFAULT_COUNTER.to_string(),
            encoder: default_login_encoder(),
        }
    }

    /// Replace the passphrase, zeroizing the previous allocation before it is released.
    pub fn replace_passphrase(&mut self, passphrase: impl Into<String>) {
        self.passphrase = Zeroizing::new(passphrase.into());
    }

    fn passphrase(&self) -> &str {
        self.passphrase.as_str()
    }

    /// Prepare ordered login variables under the v1 whitespace and case rules.
    ///
    /// Trims Unicode whitespace around service, username, and counter, then
    /// lowercases ASCII service letters only. No NFC/NFD or DNS/IDNA normalization
    /// occurs. The returned strings are caller-owned and may be sensitive.
    pub fn variables(&self) -> Vec<Variable> {
        vec![
            Variable::new("purpose", "login"),
            Variable::new("service", self.service.trim().to_ascii_lowercase()),
            Variable::new("username", self.username.trim()),
            Variable::new("counter", self.counter.trim()),
        ]
    }
}

impl fmt::Debug for LoginInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginInput")
            .field("passphrase", &"[REDACTED]")
            .field("service", &self.service)
            .field("username", &self.username)
            .field("counter", &self.counter)
            .field("encoder", &self.encoder)
            .finish()
    }
}

impl ZeroizeOnDrop for LoginInput {}

/// Validate and derive using the original passphrase and prepared login variables.
///
/// Returns [`Error`] for login or core validation/KDF failures. Empty effective
/// counters are rejected by core validation. The core's resource limitations
/// apply; allocation or thread-pool failure need not be recoverable. The returned
/// credential is caller-owned and should use an explicit secret-output channel.
pub fn generate_login_password(input: &LoginInput) -> Result<String, Error> {
    validate_login_input(input)?;
    Ok(secret_sorbet_core::derive_with_encoder(
        input.passphrase(),
        &input.variables(),
        &input.encoder,
    )?)
}

/// Return the core's compatibility-sensitive default login encoder.
pub fn default_login_encoder() -> Encoder {
    secret_sorbet_core::default_encoder()
}

/// Return the core's compatibility-sensitive default 20-character policy.
pub fn default_login_policy() -> OutputPolicy {
    secret_sorbet_core::default_password_policy()
}

/// Check service and username shape; secret, counter, and policy checks are deferred.
///
/// Returns [`Error::EmptyService`], [`Error::InvalidService`], or
/// [`Error::EmptyUsername`]. Successful validation alone does not guarantee that
/// [`generate_login_password`] succeeds or that a service is a valid DNS name.
pub fn validate_login_input(input: &LoginInput) -> Result<(), Error> {
    validate_service(&input.service)?;
    validate_username(&input.username)?;
    Ok(())
}

/// Require nonempty trimmed text containing a dot and no Unicode whitespace.
///
/// This is not a full DNS or IDNA validator. Returns [`Error::EmptyService`] or
/// [`Error::InvalidService`] without including the service in display text.
pub fn validate_service(service: &str) -> Result<(), Error> {
    let service = service.trim();
    if service.is_empty() {
        return Err(Error::EmptyService);
    }
    if service.chars().any(char::is_whitespace) {
        return Err(Error::InvalidService);
    }
    if !service.contains('.') {
        return Err(Error::InvalidService);
    }
    Ok(())
}

/// Reject usernames empty after Unicode whitespace trimming.
///
/// Case, Unicode spelling, and account syntax are otherwise unconstrained.
pub fn validate_username(username: &str) -> Result<(), Error> {
    if username.trim().is_empty() {
        return Err(Error::EmptyUsername);
    }
    Ok(())
}

/// Check a minimal email shape after trimming, without DNS or Unicode normalization.
///
/// Requires nonempty parts around the first `@`, a dot in the domain, and no
/// whitespace. Returns [`Error::InvalidEmail`]. Login usernames are not required
/// to pass this helper.
pub fn validate_email(email: &str) -> Result<(), Error> {
    let email = email.trim();
    let Some((local, domain)) = email.split_once('@') else {
        return Err(Error::InvalidEmail);
    };
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err(Error::InvalidEmail);
    }
    if email.chars().any(char::is_whitespace) {
        return Err(Error::InvalidEmail);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_generation_is_stable() {
        let input = LoginInput::new("long private passphrase", "GitHub.com", "ted@example.com");

        let a = generate_login_password(&input).unwrap();
        let b = generate_login_password(&input).unwrap();

        assert_eq!(a, b);
    }

    #[test]
    fn service_is_canonicalized_to_lowercase() {
        let upper = LoginInput::new("long private passphrase", "GitHub.com", "ted@example.com");
        let lower = LoginInput::new("long private passphrase", "github.com", "ted@example.com");

        assert_eq!(
            generate_login_password(&upper).unwrap(),
            generate_login_password(&lower).unwrap()
        );
    }

    #[test]
    fn counter_rotates_password() {
        let mut input = LoginInput::new("long private passphrase", "github.com", "ted@example.com");
        let first = generate_login_password(&input).unwrap();

        input.counter = "2".to_string();
        let second = generate_login_password(&input).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn validates_email_shape() {
        assert!(validate_email("ted@example.com").is_ok());
        assert_eq!(validate_email("ted example.com"), Err(Error::InvalidEmail));
    }

    #[test]
    fn debug_output_redacts_passphrase() {
        let input = LoginInput::new("do not print me", "github.com", "ted@example.com");
        let debug = format!("{input:?}");

        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains("do not print me"));
    }

    #[test]
    fn login_input_guarantees_zeroization_on_drop() {
        fn assert_zeroizes_on_drop<T: ZeroizeOnDrop>() {}

        assert_zeroizes_on_drop::<LoginInput>();
    }

    #[test]
    fn replacing_passphrase_changes_the_generated_password() {
        let mut input =
            LoginInput::new("first private passphrase", "github.com", "ted@example.com");
        let first = generate_login_password(&input).unwrap();

        input.replace_passphrase("second private passphrase");
        let second = generate_login_password(&input).unwrap();

        assert_ne!(first, second);
    }
}

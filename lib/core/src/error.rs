use std::fmt;

/// Failures returned by deterministic derivation and output encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptySecret,
    EmptyVariableName,
    EmptyVariableValue { name: String },
    EmptyCharset { name: String },
    NonAsciiCharset { name: String },
    EmptyAllowedCharset,
    InvalidLength,
    RequiredCountExceedsLength,
    RequiredClassNotAllowed { name: String, ch: char },
    Kdf,
    HmacKey,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptySecret => "the secret must not be empty",
            Self::EmptyVariableName => "derivation variable names must not be empty",
            Self::EmptyVariableValue { .. } => "derivation variable values must not be empty",
            Self::EmptyCharset { .. } => "character sets must not be empty",
            Self::NonAsciiCharset { .. } => "character sets must contain only ASCII characters",
            Self::EmptyAllowedCharset => "the allowed character set must not be empty",
            Self::InvalidLength => "the output length must be greater than zero",
            Self::RequiredCountExceedsLength => {
                "required character counts exceed the output length"
            }
            Self::RequiredClassNotAllowed { .. } => {
                "a required character class is absent from the allowed character set"
            }
            Self::Kdf => "key derivation failed",
            Self::HmacKey => "HMAC initialization failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_do_not_expose_embedded_input() {
        let variable = Error::EmptyVariableValue {
            name: "private-variable-name".to_owned(),
        };
        let character = Error::RequiredClassNotAllowed {
            name: "private-class-name".to_owned(),
            ch: 'x',
        };

        assert!(!variable.to_string().contains("private-variable-name"));
        assert!(!character.to_string().contains("private-class-name"));
        assert!(!character.to_string().contains("'x'"));
    }
}

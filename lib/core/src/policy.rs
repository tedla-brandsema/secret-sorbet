use crate::Error;

/// Ordered ASCII alphabet and minimum count for a required output class.
///
/// The name and raw character string (including duplicates) enter the context.
/// Labels may be empty or Unicode; alphabets must be nonempty ASCII. ASCII control
/// bytes are accepted, so callers needing printable output must choose printable
/// alphabets. Validation occurs when deriving or encoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterClass {
    pub name: String,
    pub chars: String,
    pub min_count: usize,
}

impl CharacterClass {
    /// Store class metadata without validation; order and duplicates are preserved.
    pub fn new(name: impl Into<String>, chars: impl Into<String>, min_count: usize) -> Self {
        Self {
            name: name.into(),
            chars: chars.into(),
            min_count,
        }
    }
}

/// Caller-owned, ordered output contract; every field enters v1 derivation.
///
/// Length must be positive and fit target memory. Required counts are summed with
/// overflow checking and must not exceed length. Every class character must be in
/// the effective allowed alphabet, even when its minimum count is zero. `None`
/// selects class-union mode; `Some` selects an explicit alphabet and a different
/// context even when their effective alphabets match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPolicy {
    pub length: usize,
    pub classes: Vec<CharacterClass>,
    pub allowed: Option<String>,
}

impl OutputPolicy {
    /// Return the nonempty ASCII alphabet, preserving each byte's first occurrence.
    ///
    /// Returns an empty/non-ASCII charset error for invalid effective alphabets.
    /// This checks the alphabet only, not output length or required-class counts.
    pub fn allowed_chars(&self) -> Result<Vec<u8>, Error> {
        let allowed = match &self.allowed {
            Some(chars) => chars.clone(),
            None => {
                let mut chars = String::new();
                for class in &self.classes {
                    chars.push_str(&class.chars);
                }
                chars
            }
        };

        validate_charset("allowed", &allowed)?;
        Ok(dedupe_ascii(allowed.as_bytes()))
    }
}

pub(crate) fn validate_policy(policy: &OutputPolicy) -> Result<(), Error> {
    if policy.length == 0 {
        return Err(Error::InvalidLength);
    }

    let mut required = 0usize;
    let allowed = policy.allowed_chars()?;
    if allowed.is_empty() {
        return Err(Error::EmptyAllowedCharset);
    }

    for class in &policy.classes {
        validate_charset(&class.name, &class.chars)?;
        required = required
            .checked_add(class.min_count)
            .ok_or(Error::RequiredCountExceedsLength)?;
        for ch in class.chars.chars() {
            if !allowed.contains(&(ch as u8)) {
                return Err(Error::RequiredClassNotAllowed {
                    name: class.name.clone(),
                    ch,
                });
            }
        }
    }

    if required > policy.length {
        return Err(Error::RequiredCountExceedsLength);
    }

    Ok(())
}

fn validate_charset(name: &str, chars: &str) -> Result<(), Error> {
    if chars.is_empty() {
        return Err(Error::EmptyCharset {
            name: name.to_string(),
        });
    }
    if !chars.is_ascii() {
        return Err(Error::NonAsciiCharset {
            name: name.to_string(),
        });
    }
    Ok(())
}

pub(crate) fn dedupe_ascii(chars: &[u8]) -> Vec<u8> {
    let mut seen = [false; 128];
    let mut out = Vec::new();
    for &ch in chars {
        if !seen[ch as usize] {
            seen[ch as usize] = true;
            out.push(ch);
        }
    }
    out
}

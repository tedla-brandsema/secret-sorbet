use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterClass {
    pub name: String,
    pub chars: String,
    pub min_count: usize,
}

impl CharacterClass {
    pub fn new(name: impl Into<String>, chars: impl Into<String>, min_count: usize) -> Self {
        Self {
            name: name.into(),
            chars: chars.into(),
            min_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputPolicy {
    pub length: usize,
    pub classes: Vec<CharacterClass>,
    pub allowed: Option<String>,
}

impl OutputPolicy {
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

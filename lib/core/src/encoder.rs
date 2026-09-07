use crate::policy::{dedupe_ascii, validate_policy};
use crate::{Error, OutputPolicy};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encoder {
    pub policy: OutputPolicy,
}

impl Encoder {
    pub fn new(policy: OutputPolicy) -> Self {
        Self { policy }
    }

    pub fn encode(&self, source: &mut impl ByteSource) -> Result<String, Error> {
        validate_policy(&self.policy)?;

        let allowed = self.policy.allowed_chars()?;
        let mut out = vec![0_u8; self.policy.length];
        let mut occupied = vec![false; self.policy.length];

        for class in &self.policy.classes {
            let class_chars = dedupe_ascii(class.chars.as_bytes());
            for _ in 0..class.min_count {
                let pos = choose_free_position(source, &occupied)?;
                let ch = choose_byte(source, &class_chars)?;
                out[pos] = ch;
                occupied[pos] = true;
            }
        }

        for i in 0..out.len() {
            if !occupied[i] {
                out[i] = choose_byte(source, &allowed)?;
            }
        }

        String::from_utf8(out).map_err(|_| Error::NonAsciiCharset {
            name: "output".to_string(),
        })
    }
}

pub trait ByteSource {
    fn next_byte(&mut self) -> Result<u8, Error>;
}

fn choose_free_position(source: &mut impl ByteSource, occupied: &[bool]) -> Result<usize, Error> {
    let free_count = occupied.iter().filter(|&&taken| !taken).count();
    let selected = choose_index(source, free_count)?;
    let mut seen = 0;
    for (pos, taken) in occupied.iter().enumerate() {
        if !taken {
            if seen == selected {
                return Ok(pos);
            }
            seen += 1;
        }
    }
    Err(Error::InvalidLength)
}

fn choose_byte(source: &mut impl ByteSource, alphabet: &[u8]) -> Result<u8, Error> {
    let index = choose_index(source, alphabet.len())?;
    Ok(alphabet[index])
}

fn choose_index(source: &mut impl ByteSource, len: usize) -> Result<usize, Error> {
    if len == 0 {
        return Err(Error::EmptyAllowedCharset);
    }
    let limit = 256 - (256 % len);
    loop {
        let b = source.next_byte()? as usize;
        if b < limit {
            return Ok(b % len);
        }
    }
}

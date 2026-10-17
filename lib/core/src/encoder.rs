use crate::policy::{dedupe_ascii, validate_policy};
use crate::{Error, OutputPolicy};

/// Deterministically encode a byte source according to an ASCII output policy.
///
/// Policy order and source byte consumption are compatibility-sensitive. A
/// caller-supplied source must enforce its own resource budget when required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Encoder {
    pub policy: OutputPolicy,
}

impl Encoder {
    /// Store a policy for validation when [`Self::encode`] is called.
    pub fn new(policy: OutputPolicy) -> Self {
        Self { policy }
    }

    /// Validate the policy and encode exactly its requested number of ASCII bytes.
    ///
    /// Required classes are placed in policy order, then remaining positions
    /// are filled from the allowed alphabet. Selection among at most 256 values
    /// retains v1's single-byte rejection mapping. Larger position ranges use
    /// eight-byte big-endian draws, rejecting draws below `2^64 mod range`.
    /// Every rejected draw consumes its full width.
    ///
    /// Returns policy validation errors before reading the source, and propagates
    /// source errors unchanged. Rejection sampling can read indefinitely if a
    /// source never supplies an acceptable draw; untrusted sources should impose
    /// a read budget. Allocation and running time also depend on policy length.
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

/// Fallible byte input for deterministic encoding.
///
/// Implementations control any read or resource budget. An exhausted source
/// should return an error; the encoder propagates it without logging input.
pub trait ByteSource {
    /// Return the next byte, or an error if another byte cannot be supplied.
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
    // Preserve v1's single-byte consumption and rejection mapping for all
    // previously terminating selections, including every character alphabet.
    if len <= 256 {
        let limit = 256 - (256 % len);
        loop {
            let b = source.next_byte()? as usize;
            if b < limit {
                return Ok(b % len);
            }
        }
    }

    let range = u64::try_from(len).map_err(|_| Error::InvalidLength)?;
    // 2^64 modulo range, without representing 2^64 or overflowing. Discarding
    // this prefix leaves a multiple of range equally likely values.
    let threshold = range.wrapping_neg() % range;
    loop {
        // Each attempt consumes exactly eight bytes, most significant first.
        // Rejected attempts consume all eight; source failures propagate at
        // the failing byte. A source that never provides an acceptable draw
        // must provide its own read budget if termination is required.
        let mut bytes = [0_u8; 8];
        for byte in &mut bytes {
            *byte = source.next_byte()?;
        }
        let draw = u64::from_be_bytes(bytes);
        if draw >= threshold {
            return usize::try_from(draw % range).map_err(|_| Error::InvalidLength);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CharacterClass;

    // Finite by construction: regression failures exhaust the source instead
    // of hanging on the old zero rejection limit for lengths above 256.
    struct FiniteSource<'a> {
        bytes: &'a [u8],
        consumed: usize,
    }

    impl<'a> FiniteSource<'a> {
        fn new(bytes: &'a [u8]) -> Self {
            Self { bytes, consumed: 0 }
        }
    }

    impl ByteSource for FiniteSource<'_> {
        fn next_byte(&mut self) -> Result<u8, Error> {
            let byte = self
                .bytes
                .get(self.consumed)
                .copied()
                .ok_or(Error::HmacKey)?;
            self.consumed += 1;
            Ok(byte)
        }
    }

    #[test]
    fn single_byte_boundaries_preserve_mapping_and_consumption() {
        for (len, bytes, expected, consumed) in [
            (1, &[255][..], 0, 1),
            (255, &[255, 254][..], 254, 2),
            (256, &[255][..], 255, 1),
        ] {
            let mut source = FiniteSource::new(bytes);
            assert_eq!(choose_index(&mut source, len), Ok(expected));
            assert_eq!(source.consumed, consumed);
        }
    }

    #[test]
    fn large_sampler_rejects_prefix_and_uses_big_endian_draws() {
        let bytes = [0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8];
        let mut source = FiniteSource::new(&bytes);
        assert_eq!(choose_index(&mut source, 257), Ok(4));
        assert_eq!(source.consumed, 16);

        // 258 rejects 255 but accepts the threshold 256 itself.
        let bytes = [0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 1, 0];
        let mut source = FiniteSource::new(&bytes);
        assert_eq!(choose_index(&mut source, 258), Ok(256));
        assert_eq!(source.consumed, 16);
    }

    #[test]
    fn large_sampler_covers_word_boundaries_without_allocations() {
        let high_bit = 1usize << (usize::BITS - 1);
        for (len, draw, expected) in [
            (512, 0, 0),
            (high_bit, u64::MAX, high_bit - 1),
            (usize::MAX, u64::MAX, 0),
            (usize::MAX, u64::MAX - 1, usize::MAX - 1),
        ] {
            let bytes = draw.to_be_bytes();
            let mut source = FiniteSource::new(&bytes);
            assert_eq!(choose_index(&mut source, len), Ok(expected));
            assert_eq!(source.consumed, 8);
        }
    }

    #[test]
    fn empty_ranges_do_not_read_and_source_failures_propagate() {
        let mut source = FiniteSource::new(&[]);
        assert_eq!(
            choose_index(&mut source, 0),
            Err(Error::EmptyAllowedCharset)
        );
        assert_eq!(source.consumed, 0);
        for len in [1, 255, 256, 257, usize::MAX] {
            assert_eq!(choose_index(&mut source, len), Err(Error::HmacKey));
        }
        for available in 0..8 {
            let mut source = FiniteSource::new(&[255; 8][..available]);
            assert_eq!(choose_index(&mut source, 257), Err(Error::HmacKey));
            assert_eq!(source.consumed, available);
        }
        // A source containing only rejected draws must fail within its budget.
        let mut source = FiniteSource::new(&[0; 16]);
        assert_eq!(choose_index(&mut source, 257), Err(Error::HmacKey));
        assert_eq!(source.consumed, 16);
    }

    #[test]
    fn required_positions_cross_the_256_boundary_with_a_finite_source() {
        for length in [255, 256, 257] {
            let encoder = Encoder::new(OutputPolicy {
                length,
                classes: vec![CharacterClass::new("synthetic", "a", 1)],
                allowed: None,
            });
            // Repeated 254 is acceptable in both samplers, and selects a valid
            // position. The remaining bytes choose the sole allowed character.
            let bytes = vec![254; length + 8];
            let mut source = FiniteSource::new(&bytes);
            assert_eq!(encoder.encode(&mut source), Ok("a".repeat(length)));
            assert_eq!(source.consumed, length + if length > 256 { 8 } else { 1 });
        }
    }

    #[test]
    fn long_output_with_no_required_positions_preserves_v1_bytes() {
        let encoder = Encoder::new(OutputPolicy {
            length: 257,
            classes: vec![CharacterClass::new("synthetic", "abc", 0)],
            allowed: None,
        });
        let bytes: Vec<u8> = (0..=255).chain(0..=1).collect();
        let mut source = FiniteSource::new(&bytes);
        assert_eq!(encoder.encode(&mut source), Ok("abc".repeat(85) + "ab"));
        assert_eq!(source.consumed, 258);
    }
}

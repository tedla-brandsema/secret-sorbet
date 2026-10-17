#![no_main]

use libfuzzer_sys::fuzz_target;
use secret_sorbet_core::{ByteSource, CharacterClass, Encoder, Error, OutputPolicy};

struct FuzzSource {
    next: u8,
    remaining: usize,
    exhausted: bool,
}

impl ByteSource for FuzzSource {
    fn next_byte(&mut self) -> Result<u8, Error> {
        if self.remaining == 0 {
            self.exhausted = true;
            return Err(Error::HmacKey);
        }
        self.remaining -= 1;
        let byte = self.next;
        self.next = self.next.wrapping_add(1);
        Ok(byte)
    }
}

fuzz_target!(|data: &[u8]| {
    let policy = policy_from_bytes(data);
    let encoder = Encoder::new(policy.clone());
    let seed = data.last().copied().unwrap_or_default();

    // Cycling bytes accept a draw within 256 attempts in the byte sampler;
    // for the bounded larger ranges, eight-byte cycling draws are above the
    // rejection prefix. This budget exceeds all valid policies' consumption
    // while making a nonterminating rejection regression fail deterministically.
    let mut first_source = FuzzSource {
        next: seed,
        remaining: 8192,
        exhausted: false,
    };
    let mut second_source = FuzzSource {
        next: seed,
        remaining: 8192,
        exhausted: false,
    };
    let first = encoder.encode(&mut first_source);
    let second = encoder.encode(&mut second_source);
    assert!(!first_source.exhausted && !second_source.exhausted);
    assert_eq!(first, second);

    if let Ok(output) = first {
        assert_eq!(output.len(), policy.length);
        assert!(output.is_ascii());

        let allowed = policy
            .allowed_chars()
            .expect("a successful policy has an alphabet");
        assert!(output.bytes().all(|byte| allowed.contains(&byte)));
        for class in &policy.classes {
            let count = output
                .bytes()
                .filter(|byte| class.chars.as_bytes().contains(byte))
                .count();
            assert!(count >= class.min_count);
        }
    }
});

fn policy_from_bytes(data: &[u8]) -> OutputPolicy {
    let byte = |index: usize| data.get(index).copied().unwrap_or_default();
    let class_count = usize::from(byte(1) % 8);
    let classes = (0..class_count)
        .map(|index| {
            let offset = 2 + index * 2;
            let min_count = if byte(offset + 1) == u8::MAX {
                usize::MAX
            } else {
                usize::from(byte(offset + 1))
            };
            CharacterClass::new(format!("class-{index}"), charset(byte(offset)), min_count)
        })
        .collect();

    OutputPolicy {
        length: (usize::from(byte(0)) | (usize::from(byte(19)) << 8)) % 513,
        classes,
        allowed: match byte(18) % 5 {
            0 => None,
            1 => Some(String::new()),
            2 => Some("abcXYZ012!@#".to_owned()),
            3 => Some("<union>".to_owned()),
            _ => Some("é".to_owned()),
        },
    }
}

fn charset(selector: u8) -> &'static str {
    match selector % 7 {
        0 => "",
        1 => "abc",
        2 => "XYZ",
        3 => "012",
        4 => "!@#",
        5 => "a0!",
        _ => "é",
    }
}

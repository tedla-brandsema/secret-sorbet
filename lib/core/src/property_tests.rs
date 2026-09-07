use proptest::prelude::*;

use crate::context::canonical_context;
use crate::policy::dedupe_ascii;
use crate::{ByteSource, CharacterClass, Encoder, Error, OutputPolicy, Variable};

struct CyclingSource {
    next: u8,
}

impl CyclingSource {
    fn new(seed: u8) -> Self {
        Self { next: seed }
    }
}

impl ByteSource for CyclingSource {
    fn next_byte(&mut self) -> Result<u8, Error> {
        let byte = self.next;
        self.next = self.next.wrapping_add(1);
        Ok(byte)
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    #[test]
    fn deduplication_preserves_first_occurrence(bytes in prop::collection::vec(0u8..=127, 0..256)) {
        let deduped = dedupe_ascii(&bytes);

        prop_assert!(deduped.iter().all(|byte| bytes.contains(byte)));
        prop_assert_eq!(deduped.len(), deduped.iter().collect::<std::collections::HashSet<_>>().len());

        let expected: Vec<u8> = bytes
            .iter()
            .copied()
            .enumerate()
            .filter_map(|(index, byte)| (!bytes[..index].contains(&byte)).then_some(byte))
            .collect();
        prop_assert_eq!(deduped, expected);
    }

    #[test]
    fn changing_a_variable_value_changes_the_context(
        left in "[ -~]{1,24}",
        right in "[ -~]{1,24}",
    ) {
        prop_assume!(left != right);
        let policy = crate::default_password_policy();
        let left = canonical_context(&[Variable::new("account", left)], &policy);
        let right = canonical_context(&[Variable::new("account", right)], &policy);

        prop_assert_ne!(left, right);
    }

    #[test]
    fn valid_policies_always_satisfy_their_contract(
        length in 2usize..65,
        lowercase_min in 0usize..17,
        digit_min in 0usize..17,
        seed in any::<u8>(),
    ) {
        prop_assume!(lowercase_min + digit_min <= length);
        let policy = OutputPolicy {
            length,
            classes: vec![
                CharacterClass::new("lowercase", "abc", lowercase_min),
                CharacterClass::new("digit", "012", digit_min),
            ],
            allowed: None,
        };
        let encoder = Encoder::new(policy);
        let output = encoder.encode(&mut CyclingSource::new(seed)).unwrap();

        prop_assert_eq!(output.len(), length);
        prop_assert!(output.bytes().all(|byte| b"abc012".contains(&byte)));
        prop_assert!(output.bytes().filter(|byte| b"abc".contains(byte)).count() >= lowercase_min);
        prop_assert!(output.bytes().filter(|byte| b"012".contains(byte)).count() >= digit_min);
    }

    #[test]
    fn encoding_is_deterministic_for_the_same_byte_stream(
        length in 1usize..65,
        seed in any::<u8>(),
    ) {
        let encoder = Encoder::new(OutputPolicy {
            length,
            classes: vec![CharacterClass::new("symbol", "!@#", 1)],
            allowed: Some("abc123!@#".to_owned()),
        });

        let first = encoder.encode(&mut CyclingSource::new(seed)).unwrap();
        let second = encoder.encode(&mut CyclingSource::new(seed)).unwrap();
        prop_assert_eq!(first, second);
    }

    #[test]
    fn arbitrary_required_counts_do_not_panic(
        length in 0usize..65,
        first_min in any::<usize>(),
        second_min in any::<usize>(),
    ) {
        let encoder = Encoder::new(OutputPolicy {
            length,
            classes: vec![
                CharacterClass::new("first", "abc", first_min),
                CharacterClass::new("second", "012", second_min),
            ],
            allowed: None,
        });

        let _ = encoder.encode(&mut CyclingSource::new(0));
    }
}

#[test]
fn absent_and_explicit_allowed_charsets_have_distinct_contexts() {
    let variables = [Variable::new("account", "alice")];
    let implicit = OutputPolicy {
        length: 8,
        classes: vec![CharacterClass::new("literal", "<union>", 0)],
        allowed: None,
    };
    let explicit = OutputPolicy {
        allowed: Some("<union>".to_owned()),
        ..implicit.clone()
    };

    assert_ne!(
        canonical_context(&variables, &implicit),
        canonical_context(&variables, &explicit)
    );
}

#[test]
fn overflowing_required_counts_are_rejected() {
    let encoder = Encoder::new(OutputPolicy {
        length: 8,
        classes: vec![
            CharacterClass::new("first", "a", usize::MAX),
            CharacterClass::new("second", "b", 1),
        ],
        allowed: None,
    });

    assert_eq!(
        encoder.encode(&mut CyclingSource::new(0)),
        Err(Error::RequiredCountExceedsLength)
    );
}

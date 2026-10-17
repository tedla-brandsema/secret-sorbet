use crate::{ALGORITHM_VERSION, OutputPolicy, Variable};

pub(crate) fn canonical_context(variables: &[Variable], policy: &OutputPolicy) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_field(&mut bytes, b"version", ALGORITHM_VERSION.as_bytes());
    push_us64(&mut bytes, policy.length as u64);

    push_us64(&mut bytes, variables.len() as u64);
    for variable in variables {
        push_field(&mut bytes, b"var-name", variable.name.as_bytes());
        push_field(&mut bytes, b"var-value", variable.value.as_bytes());
    }

    push_us64(&mut bytes, policy.classes.len() as u64);
    for class in &policy.classes {
        push_field(&mut bytes, b"class-name", class.name.as_bytes());
        push_field(&mut bytes, b"class-chars", class.chars.as_bytes());
        push_us64(&mut bytes, class.min_count as u64);
    }

    match &policy.allowed {
        Some(allowed) => {
            push_field(&mut bytes, b"allowed-explicit", allowed.as_bytes());
        }
        None => {
            push_field(&mut bytes, b"allowed-class-union", b"");
        }
    }

    bytes
}

fn push_field(dst: &mut Vec<u8>, label: &[u8], value: &[u8]) {
    push_us64(dst, label.len() as u64);
    dst.extend_from_slice(label);
    push_us64(dst, value.len() as u64);
    dst.extend_from_slice(value);
}

fn push_us64(dst: &mut Vec<u8>, value: u64) {
    dst.extend_from_slice(&value.to_be_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compatibility_fixture as fixture;

    #[test]
    fn synthetic_context_pins_utf8_byte_lengths_field_order_and_union_marker() {
        assert_eq!(
            canonical_context(&fixture::variables(), &fixture::policy()),
            fixture::CONTEXT
        );
        let mut explicit = fixture::policy();
        explicit.allowed = Some("ab12".to_owned());
        assert_eq!(
            canonical_context(&fixture::variables(), &explicit),
            fixture::EXPLICIT_CONTEXT
        );
    }

    #[test]
    fn integer_encoding_is_fixed_width_big_endian_at_portability_boundaries() {
        let cases = [
            (0, [0, 0, 0, 0, 0, 0, 0, 0]),
            (1, [0, 0, 0, 0, 0, 0, 0, 1]),
            (255, [0, 0, 0, 0, 0, 0, 0, 255]),
            (256, [0, 0, 0, 0, 0, 0, 1, 0]),
            (65535, [0, 0, 0, 0, 0, 0, 255, 255]),
            (65536, [0, 0, 0, 0, 0, 1, 0, 0]),
            (0xffff_ffff, [0, 0, 0, 0, 255, 255, 255, 255]),
            (0x1_0000_0000, [0, 0, 0, 1, 0, 0, 0, 0]),
            (u64::MAX, [255; 8]),
        ];
        for (value, expected) in cases {
            let mut actual = Vec::new();
            push_us64(&mut actual, value);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn length_prefix_crosses_byte_boundary_without_large_allocations() {
        for (length, prefix) in [
            (255, [0, 0, 0, 0, 0, 0, 0, 255]),
            (256, [0, 0, 0, 0, 0, 0, 1, 0]),
        ] {
            let mut actual = Vec::new();
            push_field(&mut actual, b"x", &vec![b'a'; length]);
            assert_eq!(&actual[..9], &[0, 0, 0, 0, 0, 0, 0, 1, b'x']);
            assert_eq!(&actual[9..17], &prefix);
            assert_eq!(&actual[17..], vec![b'a'; length]);
        }
    }
}

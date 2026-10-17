use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use secret_sorbet_pw::{LoginInput, validate_email, validate_service, validate_username};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 512,
        rng_seed: RngSeed::Fixed(0x5057_5631),
        ..ProptestConfig::default()
    })]

    #[test]
    fn service_variables_are_ascii_case_insensitive(
        label in "[a-z0-9]{1,24}",
        suffix in "[a-z]{2,8}",
    ) {
        let lower = format!("{label}.{suffix}");
        let upper = lower.to_ascii_uppercase();
        let lower = LoginInput::new("passphrase", lower, "alice@example.com");
        let upper = LoginInput::new("passphrase", upper, "alice@example.com");

        prop_assert_eq!(lower.variables(), upper.variables());
    }

    #[test]
    fn surrounding_whitespace_does_not_change_variables(
        label in "[a-z0-9]{1,24}",
        username in "[ -~]{1,24}",
    ) {
        let service = format!("{label}.example");
        let padded_service = format!("  {service}\t");
        let padded_username = format!("\n{username}  ");
        let plain = LoginInput::new("passphrase", service, username);
        let padded = LoginInput::new("passphrase", padded_service, padded_username);

        prop_assert_eq!(plain.variables(), padded.variables());
    }

    #[test]
    fn public_validators_accept_arbitrary_text_without_panicking(
        value in prop::collection::vec(any::<char>(), 0..128)
            .prop_map(|chars| chars.into_iter().collect::<String>()),
    ) {
        let _ = validate_service(&value);
        let _ = validate_username(&value);
        let _ = validate_email(&value);
    }
}

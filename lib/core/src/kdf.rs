use argon2::{Algorithm, Argon2, Params, Version};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::{Error, KDF_PROFILE};

const DERIVED_KEY_LENGTH: usize = KDF_PROFILE.output_length;

pub(crate) fn derive_key(
    passphrase: &[u8],
    context: &[u8],
) -> Result<Zeroizing<[u8; DERIVED_KEY_LENGTH]>, Error> {
    let salt_digest = Sha256::new()
        .chain_update(KDF_PROFILE.salt_domain.as_bytes())
        .chain_update((context.len() as u64).to_be_bytes())
        .chain_update(context)
        .finalize();
    let salt = &salt_digest[..KDF_PROFILE.salt_length];

    let params = Params::new(
        KDF_PROFILE.memory_kib,
        KDF_PROFILE.iterations,
        KDF_PROFILE.lanes,
        Some(KDF_PROFILE.output_length),
    )
    .map_err(|_| Error::Kdf)?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; DERIVED_KEY_LENGTH]);
    argon2
        .hash_password_into(passphrase, salt, key.as_mut())
        .map_err(|_| Error::Kdf)?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_salt_digest_truncation_and_argon2_key_match_independent_reference() {
        use crate::compatibility_fixture as fixture;

        let digest = Sha256::new()
            .chain_update(KDF_PROFILE.salt_domain.as_bytes())
            .chain_update((fixture::CONTEXT.len() as u64).to_be_bytes())
            .chain_update(fixture::CONTEXT)
            .finalize();
        assert_eq!(digest.as_slice(), fixture::SALT_DIGEST);
        assert_eq!(&digest[..KDF_PROFILE.salt_length], fixture::SALT);
        assert_eq!(
            derive_key("Synthetic core 🔐".as_bytes(), fixture::CONTEXT)
                .unwrap()
                .as_ref(),
            fixture::KEY
        );
    }

    #[test]
    fn uses_rfc_9106_memory_constrained_parameters() {
        assert_eq!(KDF_PROFILE.memory_kib, 64 * 1024);
        assert_eq!(KDF_PROFILE.iterations, 3);
        assert_eq!(KDF_PROFILE.lanes, 4);
        assert_eq!(DERIVED_KEY_LENGTH, 32);
        assert_eq!(KDF_PROFILE.salt_length, 16);
    }
}

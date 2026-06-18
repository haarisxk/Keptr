use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, Params,
};
use rand::rngs::OsRng;
use crate::memory::SecretBytes;

/// Derives a cryptographic key from a master password using Argon2id.
/// Returns the derived key and the salt used.
pub fn derive_master_key(password: &SecretBytes, salt: Option<&str>) -> Result<(SecretBytes, String), String> {
    let salt = match salt {
        Some(s) => SaltString::new(s).map_err(|e| e.to_string())?,
        None => SaltString::generate(&mut OsRng),
    };

    // Minimum security baseline required by Keptr Master Prompt
    // 64 MB memory cost, 8 iterations minimum.
    let params = Params::new(65536, 8, 4, Some(32)).map_err(|e| e.to_string())?;
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        params,
    );

    let mut output_key = vec![0u8; 32];
    argon2.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut output_key).map_err(|e| e.to_string())?;

    Ok((SecretBytes::new(output_key), salt.as_str().to_string()))
}

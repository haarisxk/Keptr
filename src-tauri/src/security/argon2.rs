//! Argon2id Key Derivation
//!
//! Memory-hard KDF resistant to GPU/ASIC attacks. Winner of the
//! Password Hashing Competition. Supports configurable security tiers.

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Algorithm, Argon2, Params, Version,
};
use zeroize::Zeroizing;

use super::secret::SecretString;
use super::{SecureKey, CryptoError};

/// Configurable Argon2id parameters for different security contexts.
pub struct Argon2Params {
    pub memory_cost_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub output_len: usize,
}

impl Default for Argon2Params {
    /// Standard security: 64 MiB, 3 iterations, 4 lanes
    fn default() -> Self {
        Self {
            memory_cost_kib: 65536,
            time_cost: 3,
            parallelism: 4,
            output_len: 32,
        }
    }
}

impl Argon2Params {
    /// High security for master key derivation: 128 MiB, 4 iterations
    #[must_use]
    pub const fn high_security() -> Self {
        Self {
            memory_cost_kib: 131072,
            time_cost: 4,
            parallelism: 4,
            output_len: 32,
        }
    }

    /// Interactive parameters for quick user-facing operations: 32 MiB, 2 iterations
    #[must_use]
    pub const fn interactive() -> Self {
        Self {
            memory_cost_kib: 32768,
            time_cost: 2,
            parallelism: 4,
            output_len: 32,
        }
    }
}

/// Generates a cryptographically random salt string suitable for Argon2.
pub fn generate_salt() -> String {
    SaltString::generate(&mut OsRng).to_string()
}

/// Derives a 256-bit key from a password using Argon2id with the given parameters.
pub fn derive_key_with_params(
    password: &SecretString,
    salt: &str,
    params: &Argon2Params,
) -> Result<SecureKey, CryptoError> {
    let salt = SaltString::from_b64(salt)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let argon2_params = Params::new(
        params.memory_cost_kib,
        params.time_cost,
        params.parallelism,
        Some(params.output_len),
    )
    .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let mut key_material = [0u8; 32];
    argon2
        .hash_password_into(
            password.exposed().as_bytes(),
            salt.as_salt().to_string().as_bytes(),
            &mut key_material,
        )
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(Zeroizing::new(key_material))
}

/// Derives a 256-bit key from a password using Argon2id with signature parameters and a secret pepper.
pub fn derive_key_with_pepper(
    password: &SecretString,
    salt: &str,
    pepper: &[u8],
    params: &Argon2Params,
) -> Result<SecureKey, CryptoError> {
    let salt = SaltString::from_b64(salt)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let argon2_params = Params::new(
        params.memory_cost_kib,
        params.time_cost,
        params.parallelism,
        Some(params.output_len),
    )
    .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    // Use new_with_secret to mix in the application pepper
    let argon2 = Argon2::new_with_secret(
        pepper,
        Algorithm::Argon2id,
        Version::V0x13,
        argon2_params,
    )
    .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let mut key_material = [0u8; 32];
    argon2
        .hash_password_into(
            password.exposed().as_bytes(),
            salt.as_str().as_bytes(), // Use salt bytes directly
            &mut key_material,
        )
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(Zeroizing::new(key_material))
}

/// Derives a 256-bit key using high-security parameters (128 MiB, 4 iterations).
///
/// This is the default for master key derivation from user passwords.
pub fn derive_key(password: &SecretString, salt: &str) -> Result<SecureKey, CryptoError> {
    derive_key_with_params(password, salt, &Argon2Params::high_security())
}

/// Hashes a password for storage using Argon2id (PHC string format).
///
/// Returns a self-contained string that includes algorithm, parameters,
/// salt, and hash — suitable for storing in a database.
pub fn hash_password(password: &[u8]) -> Result<String, CryptoError> {
    let salt = SaltString::generate(&mut OsRng);

    let params = Params::new(
        Argon2Params::default().memory_cost_kib,
        Argon2Params::default().time_cost,
        Argon2Params::default().parallelism,
        None,
    )
    .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let hash = argon2
        .hash_password(password, &salt)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(hash.to_string())
}

/// Verifies a password against a stored PHC-formatted hash.
///
/// Uses constant-time comparison to prevent timing attacks.
pub fn verify_password(password: &[u8], hash: &str) -> Result<bool, CryptoError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| CryptoError::KeyDerivationFailed(format!("Invalid hash format: {}", e)))?;

    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password, &parsed_hash).is_ok())
}

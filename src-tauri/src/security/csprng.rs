//! Cryptographically Secure Random Number Generation
//!
//! Uses the OS CSPRNG via `try_fill_bytes()` with proper error propagation
//! (never panics on RNG failure).

use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroizing;

use super::CryptoError;

/// Generates cryptographically secure random bytes.
pub fn generate(length: usize) -> Vec<u8> {
    let mut buf = vec![0u8; length];
    OsRng.fill_bytes(&mut buf);
    buf
}

/// Fills a mutable byte slice with cryptographically secure random bytes.
///
/// Uses `try_fill_bytes()` with error propagation instead of panicking.
pub fn try_fill_random(dest: &mut [u8]) -> Result<(), CryptoError> {
    OsRng
        .try_fill_bytes(dest)
        .map_err(|e| CryptoError::RandomGenerationFailed(e.to_string()))
}

/// Generates a random fixed-size array.
pub fn generate_array<const N: usize>() -> Result<[u8; N], CryptoError> {
    let mut bytes = [0u8; N];
    try_fill_random(&mut bytes)?;
    Ok(bytes)
}

/// Generates a 24-byte nonce for XChaCha20-Poly1305.
pub fn generate_nonce_24() -> Result<[u8; 24], CryptoError> {
    generate_array::<24>()
}

/// Generates a 12-byte nonce for AES-256-GCM.
pub fn generate_nonce_12() -> Result<[u8; 12], CryptoError> {
    generate_array::<12>()
}

/// Generates a 32-byte salt for key derivation.
pub fn generate_salt() -> Result<[u8; 32], CryptoError> {
    generate_array::<32>()
}

/// Generates a 32-byte symmetric key wrapped in Zeroizing.
pub fn generate_key() -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let mut key = Zeroizing::new([0u8; 32]);
    try_fill_random(&mut *key)?;
    Ok(key)
}

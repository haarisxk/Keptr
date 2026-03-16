//! Security module: provides all cryptographic primitives used by the vault.
//!
//! Each algorithm lives in its own sub-module:
//! - `argon2`    — Argon2id key derivation (configurable tiers)
//! - `xchacha20` — XChaCha20-Poly1305 authenticated encryption (with AAD)
//! - `aes_gcm`   — AES-256-GCM authenticated encryption (with AAD)
//! - `hashing`   — SHA-256, SHA-512, HMAC-SHA256, HMAC-SHA512
//! - `ed25519`   — Ed25519 digital signatures (with strict verification)
//! - `csprng`    — Cryptographically secure random byte generation
//! - `secret`    — Secure string types (SecretString)

pub mod secret;
pub mod argon2;
pub mod xchacha20;
pub mod hashing;
pub mod ed25519;
pub mod csprng;
pub mod key_hierarchy;
pub mod pepper;
pub mod shamir;
pub mod asymmetric;
pub mod anti_forensic;

pub use self::key_hierarchy::KeyHierarchy;
pub use self::pepper::get_pepper;
pub use self::secret::SecretString;

use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

/// A 256-bit key that is zeroed from memory on drop.
pub type SecureKey = Zeroizing<[u8; 32]>;

/// Errors returned by cryptographic operations.
#[derive(Debug, Serialize)]
pub enum CryptoError {
    KeyDerivationFailed(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    InvalidKey(String),
    VerificationFailed(String),
    SigningFailed(String),
    HmacError(String),
    RandomGenerationFailed(String),
    ShamirError(String),
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CryptoError {}

/// Container for encrypted data, including nonce and algorithm identifier.
#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct EncryptedData {
    #[zeroize(skip)]
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub algorithm: String,
}

/// Unified facade for all cryptographic operations.
pub struct CryptoService;

impl CryptoService {
    // ── Key Derivation (Argon2id) ──

    pub fn generate_salt() -> String {
        argon2::generate_salt()
    }

    pub fn derive_key(password: &secret::SecretString, salt: &str) -> Result<SecureKey, CryptoError> {
        argon2::derive_key(password, salt)
    }

    /// Derives with explicit parameter tier.
    pub fn derive_key_with_params(
        password: &secret::SecretString,
        salt: &str,
        params: &argon2::Argon2Params,
    ) -> Result<SecureKey, CryptoError> {
        argon2::derive_key_with_params(password, salt, params)
    }

    /// Hashes a password for storage (PHC format). Includes salt + params.
    pub fn hash_password(password: &[u8]) -> Result<String, CryptoError> {
        argon2::hash_password(password)
    }

    /// Verifies a password against a stored PHC hash (constant-time).
    pub fn verify_password(password: &[u8], hash: &str) -> Result<bool, CryptoError> {
        argon2::verify_password(password, hash)
    }

    // ── Symmetric Encryption ──

    pub fn encrypt_xchacha20(data: &[u8], key: &SecureKey) -> Result<EncryptedData, CryptoError> {
        xchacha20::encrypt(data, key, None)
    }

    pub fn encrypt_xchacha20_aad(data: &[u8], key: &SecureKey, aad: &[u8]) -> Result<EncryptedData, CryptoError> {
        xchacha20::encrypt(data, key, Some(aad))
    }

    pub fn decrypt_xchacha20(encrypted: &EncryptedData, key: &SecureKey) -> Result<Vec<u8>, CryptoError> {
        xchacha20::decrypt(encrypted, key, None).map(|z| z.to_vec())
    }

    pub fn decrypt_xchacha20_aad(encrypted: &EncryptedData, key: &SecureKey, aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        xchacha20::decrypt(encrypted, key, Some(aad)).map(|z| z.to_vec())
    }

    // ── Hashing & HMAC ──

    pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
        hashing::sha256(data)
    }

    pub fn hash_sha512(data: &[u8]) -> [u8; 64] {
        hashing::sha512(data)
    }

    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
        hashing::hmac_sha256(key, data)
    }

    pub fn verify_hmac_sha256(key: &[u8], data: &[u8], tag: &[u8; 32]) -> bool {
        hashing::verify_hmac_sha256(key, data, tag)
    }

    pub fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
        hashing::hmac_sha512(key, data)
    }

    pub fn verify_hmac_sha512(key: &[u8], data: &[u8], tag: &[u8; 64]) -> bool {
        hashing::verify_hmac_sha512(key, data, tag)
    }

    // ── Digital Signatures (Ed25519) ──

    pub fn ed25519_generate_keypair() -> (ed25519::Ed25519SigningKeyBytes, ed25519::Ed25519VerifyingKeyBytes) {
        ed25519::generate_keypair()
    }

    pub fn ed25519_sign(message: &[u8], signing_key: &ed25519::Ed25519SigningKeyBytes) -> Result<ed25519::Ed25519SignatureBytes, CryptoError> {
        ed25519::sign(message, signing_key.as_ref())
    }

    pub fn ed25519_verify(
        message: &[u8],
        signature: &ed25519::Ed25519SignatureBytes,
        verifying_key: &ed25519::Ed25519VerifyingKeyBytes,
    ) -> Result<(), CryptoError> {
        ed25519::verify(message, signature, verifying_key)
    }

    pub fn ed25519_verify_strict(
        message: &[u8],
        signature: &ed25519::Ed25519SignatureBytes,
        verifying_key: &ed25519::Ed25519VerifyingKeyBytes,
    ) -> Result<(), CryptoError> {
        ed25519::verify_strict(message, signature, verifying_key)
    }



    // ── Key Wrapping ──

    pub fn wrap_key(key_to_wrap: &[u8], wrapping_key_bytes: &[u8]) -> Result<EncryptedData, CryptoError> {
        let wrapping_key = Zeroizing::new(
            <[u8; 32]>::try_from(wrapping_key_bytes)
                .map_err(|_| CryptoError::InvalidKey("Wrapping key must be 32 bytes".into()))?
        );
        xchacha20::encrypt(key_to_wrap, &wrapping_key, None)
    }

    pub fn unwrap_key(encrypted: &EncryptedData, wrapping_key_bytes: &[u8]) -> Result<SecureKey, CryptoError> {
        let wrapping_key = Zeroizing::new(
             <[u8; 32]>::try_from(wrapping_key_bytes)
                .map_err(|_| CryptoError::InvalidKey("Wrapping key must be 32 bytes".into()))?
        );
        let decrypted_bytes = xchacha20::decrypt(encrypted, &wrapping_key, None)?;
        
        let key_array = <[u8; 32]>::try_from(decrypted_bytes.as_slice())
            .map_err(|_| CryptoError::DecryptionFailed("Unwrapped key has incorrect length".into()))?;
            
        Ok(Zeroizing::new(key_array))
    }

    // ── CSPRNG ──

    pub fn generate_csprng_data(length: usize) -> Vec<u8> {
        csprng::generate(length)
    }
}

//! Ed25519 Digital Signatures
//!
//! Provides key generation, signing, standard and strict verification.
//! Uses `ed25519-dalek` v2.2 with zeroize support.

use ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey,
    SECRET_KEY_LENGTH, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};
use rand::rngs::OsRng;
use zeroize::Zeroizing;

use super::CryptoError;

/// Ed25519 signing key bytes (32 bytes).
pub type Ed25519SigningKeyBytes = Zeroizing<[u8; SECRET_KEY_LENGTH]>;

/// Ed25519 verifying key bytes (32 bytes).
pub type Ed25519VerifyingKeyBytes = [u8; PUBLIC_KEY_LENGTH];

/// Ed25519 signature bytes (64 bytes).
pub type Ed25519SignatureBytes = [u8; SIGNATURE_LENGTH];

/// OOP-style keypair for Ed25519 signing operations.
pub struct Ed25519Keypair {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Ed25519Keypair {
    /// Generates a new random keypair using the OS CSPRNG.
    #[must_use]
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }

    /// Reconstructs a keypair from a 32-byte secret key.
    pub fn from_secret_key(secret_key: &[u8]) -> Result<Self, CryptoError> {
        if secret_key.len() != SECRET_KEY_LENGTH {
            return Err(CryptoError::InvalidKey(
                format!("Expected {} byte Ed25519 secret key, got {}", SECRET_KEY_LENGTH, secret_key.len()),
            ));
        }

        let bytes: [u8; SECRET_KEY_LENGTH] = secret_key
            .try_into()
            .map_err(|_| CryptoError::InvalidKey("Invalid Ed25519 secret key".to_string()))?;

        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self { signing_key, verifying_key })
    }

    /// Returns the public (verifying) key bytes.
    #[must_use]
    pub fn public_key(&self) -> Ed25519VerifyingKeyBytes {
        self.verifying_key.to_bytes()
    }

    /// Returns the secret (signing) key bytes wrapped in Zeroizing.
    #[must_use]
    pub fn secret_key(&self) -> Ed25519SigningKeyBytes {
        Zeroizing::new(self.signing_key.to_bytes())
    }

    /// Signs a message, returning a 64-byte signature.
    pub fn sign(&self, message: &[u8]) -> Ed25519SignatureBytes {
        self.signing_key.sign(message).to_bytes()
    }

    /// Verifies a signature against a message.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let sig = parse_signature(signature)?;
        self.verifying_key
            .verify(message, &sig)
            .map_err(|_| CryptoError::VerificationFailed("Ed25519 signature verification failed".to_string()))
    }

    /// Strict verification (rejects weak-key forgeries, cofactored signatures).
    pub fn verify_strict(&self, message: &[u8], signature: &[u8]) -> Result<(), CryptoError> {
        let sig = parse_signature(signature)?;
        self.verifying_key
            .verify_strict(message, &sig)
            .map_err(|_| CryptoError::VerificationFailed("Ed25519 strict verification failed".to_string()))
    }
}

// ── Standalone functions (backward-compatible API) ──

/// Generates a new Ed25519 keypair, returning (signing_key, verifying_key).
pub fn generate_keypair() -> (Ed25519SigningKeyBytes, Ed25519VerifyingKeyBytes) {
    let keypair = Ed25519Keypair::generate();
    (keypair.secret_key(), keypair.public_key())
}

/// Signs a message using a 32-byte secret key.
pub fn sign(message: &[u8], signing_key: &[u8]) -> Result<Ed25519SignatureBytes, CryptoError> {
    let keypair = Ed25519Keypair::from_secret_key(signing_key)?;
    Ok(keypair.sign(message))
}

/// Verifies a signature using a 32-byte public key.
pub fn verify(message: &[u8], signature: &[u8], verifying_key: &[u8]) -> Result<(), CryptoError> {
    if verifying_key.len() != PUBLIC_KEY_LENGTH {
        return Err(CryptoError::InvalidKey(
            format!("Expected {} byte Ed25519 public key, got {}", PUBLIC_KEY_LENGTH, verifying_key.len()),
        ));
    }

    let vk_bytes: [u8; PUBLIC_KEY_LENGTH] = verifying_key
        .try_into()
        .map_err(|_| CryptoError::InvalidKey("Invalid public key".to_string()))?;

    let vk = VerifyingKey::from_bytes(&vk_bytes)
        .map_err(|_| CryptoError::InvalidKey("Invalid Ed25519 public key".to_string()))?;

    let sig = parse_signature(signature)?;

    vk.verify(message, &sig)
        .map_err(|_| CryptoError::VerificationFailed("Ed25519 signature verification failed".to_string()))
}

/// Strict verification using a 32-byte public key (rejects weak-key forgeries).
pub fn verify_strict(message: &[u8], signature: &[u8], verifying_key: &[u8]) -> Result<(), CryptoError> {
    if verifying_key.len() != PUBLIC_KEY_LENGTH {
        return Err(CryptoError::InvalidKey(
            format!("Expected {} byte Ed25519 public key, got {}", PUBLIC_KEY_LENGTH, verifying_key.len()),
        ));
    }

    let vk_bytes: [u8; PUBLIC_KEY_LENGTH] = verifying_key
        .try_into()
        .map_err(|_| CryptoError::InvalidKey("Invalid public key".to_string()))?;

    let vk = VerifyingKey::from_bytes(&vk_bytes)
        .map_err(|_| CryptoError::InvalidKey("Invalid Ed25519 public key".to_string()))?;

    let sig = parse_signature(signature)?;

    vk.verify_strict(message, &sig)
        .map_err(|_| CryptoError::VerificationFailed("Ed25519 strict verification failed".to_string()))
}

/// Parses a 64-byte signature from a byte slice.
fn parse_signature(signature: &[u8]) -> Result<Signature, CryptoError> {
    if signature.len() != SIGNATURE_LENGTH {
        return Err(CryptoError::VerificationFailed(
            format!("Expected {} byte signature, got {}", SIGNATURE_LENGTH, signature.len()),
        ));
    }

    let sig_bytes: [u8; SIGNATURE_LENGTH] = signature
        .try_into()
        .map_err(|_| CryptoError::VerificationFailed("Invalid signature format".to_string()))?;

    Ok(Signature::from_bytes(&sig_bytes))
}

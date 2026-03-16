//! XChaCha20-Poly1305 AEAD encryption
//!
//! Primary encryption algorithm for Keptr. Provides authenticated encryption
//! with 192-bit nonces (large enough for safe random generation without collision risk).

use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use zeroize::Zeroizing;

use super::{SecureKey, CryptoError, EncryptedData};

const NONCE_SIZE: usize = 24;
const AUTH_TAG_SIZE: usize = 16;
const KEY_SIZE: usize = 32;

/// Encrypts data using XChaCha20-Poly1305 with a random 192-bit nonce.
///
/// Supports optional Additional Authenticated Data (AAD) which is
/// authenticated but not encrypted — useful for binding metadata
/// (e.g. item IDs, timestamps) to the ciphertext.
pub fn encrypt(data: &[u8], key: &SecureKey, aad: Option<&[u8]>) -> Result<EncryptedData, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKey(
            format!("Expected {} byte key, got {}", KEY_SIZE, key.len()),
        ));
    }

    let cipher = XChaCha20Poly1305::new(key.as_slice().into());

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    super::csprng::try_fill_random(&mut nonce_bytes)?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = match aad {
        Some(aad_data) => {
            let payload = Payload { msg: data, aad: aad_data };
            cipher.encrypt(nonce, payload)
        }
        None => cipher.encrypt(nonce, data),
    }
    .map_err(|_| CryptoError::EncryptionFailed("XChaCha20 encryption failed".to_string()))?;

    Ok(EncryptedData {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        algorithm: "XChaCha20-Poly1305".to_string(),
    })
}

/// Decrypts data encrypted with XChaCha20-Poly1305.
///
/// Returns plaintext wrapped in `Zeroizing` for automatic memory cleanup.
/// The same AAD used during encryption must be provided for decryption.
pub fn decrypt(encrypted: &EncryptedData, key: &SecureKey, aad: Option<&[u8]>) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if encrypted.algorithm != "XChaCha20-Poly1305" {
        return Err(CryptoError::DecryptionFailed("Algorithm mismatch".to_string()));
    }
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKey(
            format!("Expected {} byte key, got {}", KEY_SIZE, key.len()),
        ));
    }
    if encrypted.nonce.len() != NONCE_SIZE {
        return Err(CryptoError::DecryptionFailed(
            format!("Invalid nonce length: expected {}, got {}", NONCE_SIZE, encrypted.nonce.len()),
        ));
    }
    if encrypted.ciphertext.len() < AUTH_TAG_SIZE {
        return Err(CryptoError::DecryptionFailed("Ciphertext too short".to_string()));
    }

    let cipher = XChaCha20Poly1305::new(key.as_slice().into());
    let nonce = XNonce::from_slice(&encrypted.nonce);

    let plaintext = match aad {
        Some(aad_data) => {
            let payload = Payload { msg: encrypted.ciphertext.as_ref(), aad: aad_data };
            cipher.decrypt(nonce, payload)
        }
        None => cipher.decrypt(nonce, encrypted.ciphertext.as_ref()),
    }
    .map_err(|_| CryptoError::DecryptionFailed("XChaCha20 decryption/authentication failed".to_string()))?;

    Ok(Zeroizing::new(plaintext))
}

/// Encrypts data with the nonce prepended to the output.
///
/// Format: `[nonce (24 bytes)][ciphertext + auth_tag]`
pub fn encrypt_with_nonce(data: &[u8], key: &SecureKey, aad: Option<&[u8]>) -> Result<Vec<u8>, CryptoError> {
    let encrypted = encrypt(data, key, aad)?;
    let mut output = Vec::with_capacity(NONCE_SIZE + encrypted.ciphertext.len());
    output.extend_from_slice(&encrypted.nonce);
    output.extend_from_slice(&encrypted.ciphertext);
    Ok(output)
}

/// Decrypts data where the nonce is prepended to the ciphertext.
pub fn decrypt_with_nonce(data: &[u8], key: &SecureKey, aad: Option<&[u8]>) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    if data.len() < NONCE_SIZE + AUTH_TAG_SIZE {
        return Err(CryptoError::DecryptionFailed("Input too short".to_string()));
    }
    let (nonce, ciphertext) = data.split_at(NONCE_SIZE);
    let encrypted = EncryptedData {
        nonce: nonce.to_vec(),
        ciphertext: ciphertext.to_vec(),
        algorithm: "XChaCha20-Poly1305".to_string(),
    };
    decrypt(&encrypted, key, aad)
}

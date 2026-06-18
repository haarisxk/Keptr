use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce, Key,
};
use crate::memory::SecretBytes;

/// Encrypts plaintext using XChaCha20-Poly1305.
/// Returns a tuple of (encrypted_data, nonce)
pub fn encrypt_data(key: &SecretBytes, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    if key.as_bytes().len() != 32 {
        return Err("Invalid key length for XChaCha20-Poly1305".to_string());
    }

    let key = Key::from_slice(key.as_bytes());
    let cipher = XChaCha20Poly1305::new(key);
    
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 24-bytes
    let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(|e| e.to_string())?;

    Ok((ciphertext, nonce.to_vec()))
}

/// Decrypts ciphertext using XChaCha20-Poly1305.
pub fn decrypt_data(key: &SecretBytes, nonce: &[u8], ciphertext: &[u8]) -> Result<SecretBytes, String> {
    if key.as_bytes().len() != 32 {
        return Err("Invalid key length".to_string());
    }
    if nonce.len() != 24 {
        return Err("Invalid nonce length".to_string());
    }

    let key = Key::from_slice(key.as_bytes());
    let cipher = XChaCha20Poly1305::new(key);
    let nonce = XNonce::from_slice(nonce);

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())?;

    Ok(SecretBytes::new(plaintext))
}

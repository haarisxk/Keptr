use crate::models::KoreHeader;
use keptr_crypto::symmetric::{encrypt_data, decrypt_data};
use keptr_crypto::mac::{compute_hmac, verify_hmac};
use keptr_crypto::memory::SecretBytes;

/// Represents a fully encrypted .kore vault item
#[derive(Debug, Clone)]
pub struct EncryptedKoreItem {
    pub header: KoreHeader,
    pub encrypted_payload: Vec<u8>,
    pub hmac: [u8; 64], // SHA-512 HMAC over full header + payload
}

impl EncryptedKoreItem {
    /// Serializes the entire Kore item to a byte vector
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.encrypted_payload);
        bytes.extend_from_slice(&self.hmac);
        bytes
    }

    /// Verifies the integrity of the .kore item
    pub fn verify_integrity(&self, item_key: &SecretBytes) -> Result<(), String> {
        let mut header_and_payload = self.header.to_bytes();
        header_and_payload.extend_from_slice(&self.encrypted_payload);
        verify_hmac(item_key, &header_and_payload, &self.hmac)
    }
}

/// Encrypts raw plaintext into a .kore item
pub fn create_kore_item(
    item_type: crate::models::ItemType,
    item_key: &SecretBytes,
    plaintext: &[u8],
    salt: [u8; 16], // Generated randomly per item
) -> Result<EncryptedKoreItem, String> {
    let (ciphertext, nonce_vec) = encrypt_data(item_key, plaintext)?;
    
    let mut nonce = [0u8; 24];
    nonce.copy_from_slice(&nonce_vec);

    let header = KoreHeader::new(item_type, salt, nonce);
    
    let mut header_and_payload = header.to_bytes();
    header_and_payload.extend_from_slice(&ciphertext);
    
    let hmac = compute_hmac(item_key, &header_and_payload)?;

    Ok(EncryptedKoreItem {
        header,
        encrypted_payload: ciphertext,
        hmac,
    })
}

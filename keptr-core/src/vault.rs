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

    /// Deserializes a Kore item from a byte vector
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 106 + 64 {
            return Err("Item too short".to_string());
        }

        let header = KoreHeader::from_bytes(&bytes[0..106])?;
        
        let payload_len = bytes.len() - 106 - 64;
        let encrypted_payload = bytes[106..(106 + payload_len)].to_vec();
        
        let mut hmac = [0u8; 64];
        hmac.copy_from_slice(&bytes[(106 + payload_len)..]);

        Ok(Self {
            header,
            encrypted_payload,
            hmac,
        })
    }

    /// Verifies the integrity of the .kore item
    pub fn verify_integrity(&self, master_key: &SecretBytes) -> Result<(), String> {
        let mut header_and_payload = self.header.to_bytes();
        header_and_payload.extend_from_slice(&self.encrypted_payload);
        verify_hmac(master_key, &header_and_payload, &self.hmac)
    }
}

/// Encrypts raw plaintext into a .kore item
pub fn create_kore_item(
    item_type: crate::models::ItemType,
    master_key: &SecretBytes,
    plaintext: &[u8],
) -> Result<EncryptedKoreItem, String> {
    // 1. Generate unique random item key
    let item_key = SecretBytes::generate_random(32);
    
    // 2. Encrypt plaintext payload with item_key
    let (payload_ciphertext, data_nonce_vec) = encrypt_data(&item_key, plaintext)?;
    let mut data_nonce = [0u8; 24];
    data_nonce.copy_from_slice(&data_nonce_vec);

    // 3. Encrypt the item_key itself with the master_key
    let (item_key_ciphertext, key_nonce_vec) = encrypt_data(master_key, item_key.as_bytes())?;
    let mut key_nonce = [0u8; 24];
    key_nonce.copy_from_slice(&key_nonce_vec);

    let mut encrypted_item_key = [0u8; 48];
    encrypted_item_key.copy_from_slice(&item_key_ciphertext);

    // 4. Create the Header
    let header = KoreHeader::new(item_type, key_nonce, encrypted_item_key, data_nonce);
    
    // 5. Compute full HMAC using master_key over everything but the MAC itself
    let mut header_and_payload = header.to_bytes();
    header_and_payload.extend_from_slice(&payload_ciphertext);
    let hmac = compute_hmac(master_key, &header_and_payload)?;

    Ok(EncryptedKoreItem {
        header,
        encrypted_payload: payload_ciphertext,
        hmac,
    })
}

/// Decrypts a .kore item back into plaintext
pub fn decrypt_kore_item(
    item: &EncryptedKoreItem,
    master_key: &SecretBytes,
) -> Result<SecretBytes, String> {
    // 1. Verify integrity first
    item.verify_integrity(master_key)?;

    // 2. Decrypt the item key using the master key
    let item_key = decrypt_data(
        master_key,
        &item.header.key_nonce,
        &item.header.encrypted_item_key,
    )?;

    // 3. Decrypt the payload using the decrypted item key
    let plaintext = decrypt_data(
        &item_key,
        &item.header.data_nonce,
        &item.encrypted_payload,
    )?;

    Ok(plaintext)
}

use serde::{Deserialize, Serialize};

/// Supported record types corresponding to the prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ItemType {
    Login = 1,
    EmailAccount = 2,
    Passkey = 3,
    SecureNote = 4,
    PaymentInformation = 5,
    Identity = 6,
    AuthenticationSecret = 7,
    DocumentRecord = 8,
    CustomRecord = 9,
}

impl TryFrom<u8> for ItemType {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ItemType::Login),
            2 => Ok(ItemType::EmailAccount),
            3 => Ok(ItemType::Passkey),
            4 => Ok(ItemType::SecureNote),
            5 => Ok(ItemType::PaymentInformation),
            6 => Ok(ItemType::Identity),
            7 => Ok(ItemType::AuthenticationSecret),
            8 => Ok(ItemType::DocumentRecord),
            9 => Ok(ItemType::CustomRecord),
            _ => Err(format!("Unknown item type: {}", value)),
        }
    }
}

/// The structure of a Kore Encrypted Blob Header
#[derive(Debug, Clone)]
pub struct KoreHeader {
    pub magic: [u8; 4],      // "KORE"
    pub version: u16,        // Current version (e.g., 2)
    pub flags: u16,          // e.g., compression
    pub item_type: ItemType, // 1 byte
    pub algo_id: u8,         // 1: XChaCha20-Poly1305
    pub key_nonce: [u8; 24], // Nonce for the encrypted item key
    pub encrypted_item_key: [u8; 48], // 32-byte item key + 16-byte Poly1305 tag
    pub data_nonce: [u8; 24], // Nonce for the actual item data
}

impl KoreHeader {
    pub fn new(item_type: ItemType, key_nonce: [u8; 24], encrypted_item_key: [u8; 48], data_nonce: [u8; 24]) -> Self {
        Self {
            magic: *b"KORE",
            version: 2,
            flags: 0,
            item_type,
            algo_id: 1, // XChaCha20Poly1305
            key_nonce,
            encrypted_item_key,
            data_nonce,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(106);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.extend_from_slice(&self.flags.to_be_bytes());
        bytes.push(self.item_type as u8);
        bytes.push(self.algo_id);
        bytes.extend_from_slice(&self.key_nonce);
        bytes.extend_from_slice(&self.encrypted_item_key);
        bytes.extend_from_slice(&self.data_nonce);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 106 {
            return Err("KoreHeader too short".to_string());
        }
        if &bytes[0..4] != b"KORE" {
            return Err("Invalid magic bytes".to_string());
        }
        
        let version = u16::from_be_bytes([bytes[4], bytes[5]]);
        if version != 2 {
            return Err(format!("Unsupported KoreHeader version: {}", version));
        }

        let flags = u16::from_be_bytes([bytes[6], bytes[7]]);
        let item_type = ItemType::try_from(bytes[8])?;
        let algo_id = bytes[9];
        
        let mut key_nonce = [0u8; 24];
        key_nonce.copy_from_slice(&bytes[10..34]);
        
        let mut encrypted_item_key = [0u8; 48];
        encrypted_item_key.copy_from_slice(&bytes[34..82]);

        let mut data_nonce = [0u8; 24];
        data_nonce.copy_from_slice(&bytes[82..106]);

        Ok(Self {
            magic: *b"KORE",
            version,
            flags,
            item_type,
            algo_id,
            key_nonce,
            encrypted_item_key,
            data_nonce,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginItem {
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: Vec<u8>, // Stored as bytes, wrapped securely in memory when decrypted
    pub totp: Option<String>,
    pub notes: String,
}

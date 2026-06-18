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
    pub version: u16,        // Current version (e.g., 1)
    pub flags: u16,          // e.g., compression
    pub item_type: ItemType, // 1 byte
    pub algo_id: u8,         // 1: XChaCha20-Poly1305
    pub salt: [u8; 16],      // For item-level key derivation
    pub nonce: [u8; 24],     // Nonce for encryption
}

impl KoreHeader {
    pub fn new(item_type: ItemType, salt: [u8; 16], nonce: [u8; 24]) -> Self {
        Self {
            magic: *b"KORE",
            version: 1,
            flags: 0,
            item_type,
            algo_id: 1, // XChaCha20Poly1305
            salt,
            nonce,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(48);
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_be_bytes());
        bytes.extend_from_slice(&self.flags.to_be_bytes());
        bytes.push(self.item_type as u8);
        bytes.push(self.algo_id);
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.nonce);
        bytes
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

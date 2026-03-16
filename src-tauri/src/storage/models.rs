pub use crate::models::VaultItem;




#[derive(Debug)]
pub enum StorageError {
    DatabaseError(String),
    SerializationError(String),
    CryptoError(String),
    IOError(String),
    NotFoundError(String),
    Other(String),
    VaultNotFound,
    ItemNotFound,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(e) => write!(f, "Database error: {}", e),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StorageError::CryptoError(e) => write!(f, "Crypto error: {}", e),
            StorageError::IOError(e) => write!(f, "IO error: {}", e),
            StorageError::NotFoundError(e) => write!(f, "Not Found: {}", e),
            StorageError::Other(e) => write!(f, "Other error: {}", e),
            StorageError::VaultNotFound => write!(f, "Vault not found"),
            StorageError::ItemNotFound => write!(f, "Item not found"),
        }
    }
}

impl std::error::Error for StorageError {}

// Optional: Keep specific From impls if needed, mapping to String variants
impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        StorageError::DatabaseError(e.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::SerializationError(e.to_string())
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::IOError(e.to_string())
    }
}



/// Internal database row representation.
pub(crate) struct VaultItemRow {
    pub id: String,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub item_salt: Option<Vec<u8>>,
}

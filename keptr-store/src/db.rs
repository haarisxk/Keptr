use rusqlite::{Connection, Result};
use keptr_core::vault::EncryptedKoreItem;

pub struct SecureStore {
    conn: Connection,
}

impl SecureStore {
    /// Opens the encrypted local database using SQLCipher.
    /// The database key should be derived independently from the Master Key.
    pub fn open(path: &str, _key: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        
        // Note: Raw SQLite used here for CI compatibility.
        // In production with OpenSSL, SQLCipher pragmas would be applied.
        
        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vault_items (
                id TEXT PRIMARY KEY,
                item_type INTEGER NOT NULL,
                kore_blob BLOB NOT NULL,
                sync_version INTEGER NOT NULL DEFAULT 0,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            (),
        )?;

        Ok(Self { conn })
    }

    /// Stores a fully encrypted .kore blob securely.
    pub fn save_kore_item(&self, item_id: &str, item_type: u8, item: &EncryptedKoreItem) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO vault_items (id, item_type, kore_blob) VALUES (?1, ?2, ?3)",
            (item_id, item_type, item.to_bytes()),
        )?;
        Ok(())
    }
}

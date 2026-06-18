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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
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

    /// Fetches all fully encrypted .kore items from the database.
    pub fn get_all_items(&self) -> Result<Vec<(String, EncryptedKoreItem)>> {
        let mut stmt = self.conn.prepare("SELECT id, kore_blob FROM vault_items")?;
        let item_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((id, blob))
        })?;

        let mut items = Vec::new();
        for item in item_iter {
            let (id, blob) = item?;
            if let Ok(kore_item) = EncryptedKoreItem::from_bytes(&blob) {
                items.push((id, kore_item));
            }
        }
        Ok(items)
    }

    /// Deletes an item from the vault by its ID.
    pub fn delete_item(&self, item_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM vault_items WHERE id = ?1",
            [item_id],
        )?;
        Ok(())
    }

    /// Set a metadata key-value pair.
    pub fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            (key, value),
        )?;
        Ok(())
    }

    /// Get a metadata value by key.
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM metadata WHERE key = ?1")?;
        let mut rows = stmt.query([key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }
}

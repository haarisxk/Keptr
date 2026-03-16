use crate::security::{CryptoService, EncryptedData, SecureKey, KeyHierarchy};
use crate::services::audit::AuditService;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use std::path::Path;

use super::models::{VaultItem, VaultItemRow, StorageError};

use std::sync::Mutex;

pub struct StorageService {
    db_path: Mutex<String>,
}

impl StorageService {
    pub fn new(app_dir: &str) -> Self {
        let data_dir = Path::new(app_dir).join("data");
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).expect("Failed to create DB dir");
        }
        // Initial path might be empty or default, depending on logic.
        // We'll init with empty and rely on select_vault to set it, 
        // OR migration logic in AppState::new to set default.
        // For compatibility, let's default to vault.kore if it exists, roughly?
        // Actually, AppState::new runs before migration? 
        // Let's just default to vault.kore for now to keep existing flow working until explicit switch.
        let db_path = data_dir.join("vault.kore").to_string_lossy().to_string();
        Self { db_path: Mutex::new(db_path) }
    }
    
    pub fn get_current_path(&self) -> String {
        self.db_path.lock().unwrap().clone()
    }

    pub fn switch_vault(&self, new_path: &str) {
        let mut path = self.db_path.lock().unwrap();
        *path = new_path.to_string();
    }

    fn get_connection(&self) -> Result<Connection> {
        let path = self.db_path.lock().unwrap();
        if path.is_empty() {
             return Err(rusqlite::Error::InvalidPath(std::path::PathBuf::from("")));
        }
        Connection::open(path.as_str())
    }

    pub fn init(&self) -> Result<(), StorageError> {
        let conn = self.get_connection()?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA encoding = 'UTF-8';",
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS vault_items (
                id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                nonce BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        // Add item_salt column if it doesn't exist (v2 migration preparation)
        let mut stmt = conn.prepare("PRAGMA table_info(vault_items)")?;
        let mut rows = stmt.query([])?;
        let mut has_salt = false;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == "item_salt" {
                has_salt = true;
                break;
            }
        }
        
        if !has_salt {
            conn.execute("ALTER TABLE vault_items ADD COLUMN item_salt BLOB", [])?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Initialize Audit Log Table
        AuditService::init(&conn).map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn create_item(&self, item: &VaultItem, key_hierarchy: &KeyHierarchy) -> Result<Uuid, StorageError> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;

        let json_data = serde_json::to_vec(item)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // 1. Generate unique per-item salt
        let item_salt = CryptoService::generate_csprng_data(32);
        
        // 2. Derive per-item key using item ID + salt
        //    Using item ID as info to bind key to this specific item
        let item_key = key_hierarchy.derive_item_key(&item.id, &item_salt)
            .map_err(|e| StorageError::CryptoError(e.to_string()))?;

        // 3. Encrypt with AAD binding (AAD = item ID)
        //    This ensures ciphertext cannot be moved to another item record
        let item_id_bytes = item.id.to_string().into_bytes();
        let encrypted = CryptoService::encrypt_xchacha20_aad(&json_data, &item_key, &item_id_bytes)
            .map_err(|e| StorageError::CryptoError(e.to_string()))?;

        tx.execute(
            "INSERT INTO vault_items (id, data, nonce, item_salt, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.id.to_string(),
                encrypted.ciphertext,
                encrypted.nonce,
                item_salt,
                item.created_at,
                item.updated_at
            ],
        )?;

        // Audit Log
        AuditService::log_event(
            &tx,
            "CREATE_ITEM",
            Some(&item.id.to_string()),
            "SUCCESS",
            None
        ).map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        tx.commit()?;
        Ok(item.id)
    }

    pub fn get_item(&self, id: Uuid, key_hierarchy: &KeyHierarchy) -> Result<VaultItem, StorageError> {
        let conn = self.get_connection()?;

        let mut stmt = conn.prepare(
            "SELECT id, data, nonce, item_salt FROM vault_items WHERE id = ?1",
        )?;

        let row = stmt.query_row(params![id.to_string()], |row| {
            Ok(VaultItemRow {
                id: row.get(0)?,
                encrypted_data: row.get(1)?,
                nonce: row.get(2)?,
                item_salt: row.get(3)?,
            })
        }).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StorageError::ItemNotFound,
            _ => StorageError::DatabaseError(e.to_string()),
        })?;

        // Check for migration necessity (missing item_salt implies V1)
        let item_salt = row.item_salt.ok_or(StorageError::CryptoError("Item missing salt (needs migration)".to_string()))?;

        // Derive per-item key
        let item_key = key_hierarchy.derive_item_key(&id, &item_salt)
             .map_err(|e| StorageError::CryptoError(e.to_string()))?;

        let encrypted_pkg = EncryptedData {
            ciphertext: row.encrypted_data,
            nonce: row.nonce,
            algorithm: "XChaCha20-Poly1305".to_string(),
        };

        // Decrypt with AAD
        let item_id_bytes = id.to_string().into_bytes();
        let plaintext = CryptoService::decrypt_xchacha20_aad(&encrypted_pkg, &item_key, &item_id_bytes)
             .map_err(|e| StorageError::CryptoError(e.to_string()))?;

        let item: VaultItem = serde_json::from_slice(&plaintext)
            .map_err(|e| StorageError::DatabaseError(format!("Data corruption or deserialization error: {}", e)))?;

        // We do NOT log every READ access to avoid write-locks on read, unless strict auditing is required.
        // For now, let's log it.
        // BUT `get_item` doesn't strictly have a transaction. 
        // We'd need to upgrade `conn` to do a write.
        // Let's SKIP logging for reads for performance/concurrency in this iteration, focusing on WRITE operations.
        // If required, we can add it later.
        
        Ok(item)
    }

    /// Extractor for Cloud Sync. Pulls the raw encrypted bytes and metadata without decrypting it.
    pub fn get_encrypted_item(&self, id: Uuid) -> Result<(String, String, String), StorageError> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT data, nonce, updated_at FROM vault_items WHERE id = ?1",
        )?;

        stmt.query_row(params![id.to_string()], |row| {
            // data (blob), nonce (blob), updated_at (string)
            let data: Vec<u8> = row.get(0)?;
            let nonce: Vec<u8> = row.get(1)?;
            let updated_at: String = row.get(2)?;
            
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            Ok((STANDARD.encode(data), STANDARD.encode(nonce), updated_at))
        }).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => StorageError::ItemNotFound,
            _ => StorageError::DatabaseError(e.to_string()),
        })
    }

    /// Pull Sync Ingestion: Takes raw encrypted payloads from Supabase and merges them into the SQLite database.
    pub fn import_sync_payloads(&self, payloads: Vec<crate::services::sync_service::SyncPayload>) -> Result<(), StorageError> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;
        use base64::{Engine as _, engine::general_purpose::STANDARD};

        for payload in payloads {
            // Check for tombstone deletion flags
            if payload.encrypted_payload == "DELETED" {
                tx.execute(
                    "DELETE FROM vault_items WHERE id = ?1",
                    params![payload.item_id],
                )?;
                continue;
            }

            // Decode from the JSON Base64 transport layer
            let data = STANDARD.decode(&payload.encrypted_payload)
               .map_err(|_| StorageError::DatabaseError("Base64 decoding failed for data".into()))?;
            let nonce = STANDARD.decode(&payload.nonce)
               .map_err(|_| StorageError::DatabaseError("Base64 decoding failed for nonce".into()))?;
               
            // Upsert the encrypted chunk natively
            tx.execute(
                "INSERT INTO vault_items (id, data, nonce, item_salt, created_at, updated_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5)
                 ON CONFLICT(id) DO UPDATE SET 
                    data = excluded.data, 
                    nonce = excluded.nonce, 
                    updated_at = excluded.updated_at",
                params![
                    payload.item_id,
                    data,
                    nonce,
                    vec![0u8; 16], // item_salt currently ignored for raw blob transit without KeyHierarchy rotation
                    chrono::Utc::now().to_rfc3339() // Fallback timestamp tracking
                ],
            )?;
        }
        
        tx.commit()?;
        Ok(())
    }

    pub fn update_item(&self, item: &VaultItem, key_hierarchy: &KeyHierarchy) -> Result<(), StorageError> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;

        let json_data = serde_json::to_vec(item)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
        // Rotate the salt on update for better security (fresh key)
        let new_salt = CryptoService::generate_csprng_data(32);
        
        let item_key = key_hierarchy.derive_item_key(&item.id, &new_salt)
            .map_err(|e| StorageError::CryptoError(e.to_string()))?;
            
        let item_id_bytes = item.id.to_string().into_bytes();
        let encrypted = CryptoService::encrypt_xchacha20_aad(&json_data, &item_key, &item_id_bytes)
             .map_err(|e| StorageError::CryptoError(e.to_string()))?;

        tx.execute(
            "UPDATE vault_items SET data = ?1, nonce = ?2, item_salt = ?3, updated_at = ?4 WHERE id = ?5",
            params![
                encrypted.ciphertext,
                encrypted.nonce,
                new_salt,
                item.updated_at,
                item.id.to_string()
            ],
        )?;

        // Audit Log
        AuditService::log_event(
            &tx,
            "UPDATE_ITEM",
            Some(&item.id.to_string()),
            "SUCCESS",
            None
        ).map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        tx.commit()?;
        Ok(())
    }

    pub fn delete_item(&self, id: Uuid) -> Result<(), StorageError> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;

        let count = tx.execute("DELETE FROM vault_items WHERE id = ?1", params![id.to_string()])?;
        if count == 0 {
            return Err(StorageError::ItemNotFound);
        }

        // Audit Log
        AuditService::log_event(
            &tx,
            "DELETE_ITEM",
            Some(&id.to_string()),
            "SUCCESS",
            None
        ).map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        tx.commit()?;
        Ok(())
    }

    pub fn list_items(&self, key_hierarchy: &KeyHierarchy) -> Result<Vec<VaultItem>, StorageError> {
        let conn = self.get_connection()?;
        // Only fetch items that HAVE a salt (v2). v1 items should be migrated before this is called.
        let mut stmt = conn.prepare("SELECT id, data, nonce, item_salt FROM vault_items WHERE item_salt IS NOT NULL ORDER BY created_at DESC")?;

        let item_iter = stmt.query_map([], |row| {
            Ok(VaultItemRow {
                id: row.get(0)?,
                encrypted_data: row.get(1)?,
                nonce: row.get(2)?,
                item_salt: row.get(3)?,
            })
        })?;

        let mut items = Vec::new();
        for row_result in item_iter {
            let row = row_result?;
            
            // Assume salt exists because of WHERE clause
            if let Some(salt) = row.item_salt {
                match Uuid::parse_str(&row.id) {
                    Ok(uuid) => {
                        let item_key = key_hierarchy.derive_item_key(&uuid, &salt)
                            .map_err(|e| StorageError::CryptoError(e.to_string()))?;
                            
                        let encrypted_pkg = EncryptedData {
                            ciphertext: row.encrypted_data,
                            nonce: row.nonce,
                            algorithm: "XChaCha20-Poly1305".to_string(),
                        };
                        
                        let item_id_bytes = row.id.as_bytes();
                        
                        // Try decrypt
                        match CryptoService::decrypt_xchacha20_aad(&encrypted_pkg, &item_key, item_id_bytes) {
                             Ok(plaintext) => {
                                 if let Ok(item) = serde_json::from_slice(&plaintext) {
                                     items.push(item);
                                 }
                             },
                             Err(_) => continue, // Skip items that fail to decrypt (wrong key or corrupted)
                        }
                    },
                    Err(_) => continue,
                }
            }
        }

        Ok(items)
    }

    pub fn save_metadata(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let conn = self.get_connection()?;
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_metadata(&self, key: &str) -> Result<Option<String>, StorageError> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare("SELECT value FROM metadata WHERE key = ?1")?;

        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn vault_exists(&self) -> bool {
        let path_str = self.db_path.lock().unwrap();
        if path_str.is_empty() { return false; }
        
        if !Path::new(&*path_str).exists() {
            return false;
        }
        // Drop lock before calling get_metadata which locks internally? 
        // get_metadata calls get_connection which locks. 
        // Recursive lock! deadlock!
        // We must drop the lock before calling self.get_metadata.
        drop(path_str);
        
        self.get_metadata("salt").unwrap_or(None).is_some()
    }
    
    // --- Migration Logic ---
    
    pub fn migrate_v2(&self, old_key: &SecureKey, new_hierarchy: &KeyHierarchy) -> Result<(), StorageError> {
        let conn = self.get_connection()?;
        
        // 1. Fetch all items that lack item_salt (Legacy V1 items)
        let mut stmt = conn.prepare("SELECT id, data, nonce FROM vault_items WHERE item_salt IS NULL")?;
        
        let rows = stmt.query_map([], |row| {
             Ok(VaultItemRow {
                id: row.get(0)?,
                encrypted_data: row.get(1)?,
                nonce: row.get(2)?,
                item_salt: None,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        
        for row in rows {
            // Decrypt with OLD MASTER KEY (no AAD)
             let encrypted_pkg = EncryptedData {
                ciphertext: row.encrypted_data,
                nonce: row.nonce,
                algorithm: "XChaCha20-Poly1305".to_string(),
            };
            
            // V1 used simple encrypt_xchacha20 without AAD
            let plaintext = CryptoService::decrypt_xchacha20(&encrypted_pkg, old_key)
                 .map_err(|e| StorageError::CryptoError(format!("Migration decrypt failed for {}: {}", row.id, e)))?;
                 
            // Re-encrypt with NEW PER-ITEM KEY (with AAD)
            let new_salt = CryptoService::generate_csprng_data(32);
            let uuid = Uuid::parse_str(&row.id).map_err(|_| StorageError::DatabaseError("Invalid UUID in DB".to_string()))?;
            
            let item_key = new_hierarchy.derive_item_key(&uuid, &new_salt)
                .map_err(|e| StorageError::CryptoError(e.to_string()))?;
                
            let item_id_bytes = row.id.as_bytes();
            let new_encrypted = CryptoService::encrypt_xchacha20_aad(&plaintext, &item_key, item_id_bytes)
                 .map_err(|e| StorageError::CryptoError(e.to_string()))?;
                 
            // Update row
            conn.execute(
                "UPDATE vault_items SET data = ?1, nonce = ?2, item_salt = ?3 WHERE id = ?4",
                params![new_encrypted.ciphertext, new_encrypted.nonce, new_salt, row.id]
            )?;
        }
        
        Ok(())
    }

    pub fn rotate_vault(
        &self, 
        old_hierarchy: &KeyHierarchy, 
        new_hierarchy: &KeyHierarchy,
        new_salt: &str,
        new_canary: &str
    ) -> Result<(), StorageError> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?; // Start transaction

        {
            // Fetch ALL items (must have salt)
            let mut stmt = tx.prepare("SELECT id, data, nonce, item_salt FROM vault_items WHERE item_salt IS NOT NULL")?;
            
            let rows = stmt.query_map([], |row| {
                 Ok(VaultItemRow {
                    id: row.get(0)?,
                    encrypted_data: row.get(1)?,
                    nonce: row.get(2)?,
                    item_salt: row.get(3)?,
                })
            })?.collect::<Result<Vec<_>, _>>()?;
            
            for row in rows {
                let item_salt = row.item_salt.ok_or(StorageError::CryptoError("Item missing salt during rotation".to_string()))?;
                let uuid = Uuid::parse_str(&row.id).map_err(|_| StorageError::DatabaseError("Invalid UUID".to_string()))?;

                // 1. Decrypt with OLD hierarchy
                let old_item_key = old_hierarchy.derive_item_key(&uuid, &item_salt)
                    .map_err(|e| StorageError::CryptoError(e.to_string()))?;
                    
                let encrypted_pkg = EncryptedData {
                    ciphertext: row.encrypted_data,
                    nonce: row.nonce,
                    algorithm: "XChaCha20-Poly1305".to_string(),
                };
                
                let item_id_bytes = row.id.as_bytes();
                let plaintext = CryptoService::decrypt_xchacha20_aad(&encrypted_pkg, &old_item_key, item_id_bytes)
                     .map_err(|e| StorageError::CryptoError(format!("Rotation decrypt failed for {}: {}", row.id, e)))?;
                     
                // 2. Encrypt with NEW hierarchy (and NEW salt)
                let new_salt = CryptoService::generate_csprng_data(32);
                let new_item_key = new_hierarchy.derive_item_key(&uuid, &new_salt)
                    .map_err(|e| StorageError::CryptoError(e.to_string()))?;
                    
                let new_encrypted = CryptoService::encrypt_xchacha20_aad(&plaintext, &new_item_key, item_id_bytes)
                     .map_err(|e| StorageError::CryptoError(e.to_string()))?;
                     
                 tx.execute(
                    "UPDATE vault_items SET data = ?1, nonce = ?2, item_salt = ?3 WHERE id = ?4",
                    params![new_encrypted.ciphertext, new_encrypted.nonce, new_salt, row.id]
                )?;
            }
            
            // 3. Update Metadata
            tx.execute(
                "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
                params!["salt", new_salt],
            )?;
            tx.execute(
                "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
                params!["canary", new_canary],
            )?;

            // Audit Log
            AuditService::log_event(
                &tx,
                "VAULT_ROTATION",
                None,
                "SUCCESS",
                Some("Key rotation performed")
            ).map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn log_auth_event(&self, action: &str, outcome: &str) -> Result<(), StorageError> {
        let mut conn = self.get_connection()?;
        let tx = conn.transaction()?;
        AuditService::log_event(&tx, action, None, outcome, None)
             .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        tx.commit()?;
        Ok(())
    }

    pub fn reset_database(&self) -> Result<(), StorageError> {
        let path_str = self.db_path.lock().unwrap().clone();
        if path_str.is_empty() { return Ok(()); }
        
        let path = Path::new(&path_str);
        
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|e| StorageError::DatabaseError(format!("Failed to delete vault file: {}", e)))?;
        }
        // Also remove shm and wal files if they exist (WAL mode)
        let shm = format!("{}-shm", path_str);
        if Path::new(&shm).exists() {
             std::fs::remove_file(&shm).ok();
        }
        let wal = format!("{}-wal", path_str);
        if Path::new(&wal).exists() {
             std::fs::remove_file(&wal).ok();
        }
        
        Ok(())
    }
}

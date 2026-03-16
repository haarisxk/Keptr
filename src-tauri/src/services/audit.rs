use crate::security::CryptoService;
use rusqlite::{params, Connection, Result, Transaction};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};


#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: i64,
    pub timestamp: i64,
    pub action: String,
    pub target_id: Option<String>,
    pub outcome: String,
    pub metadata: Option<String>,
    pub prev_hash: String,
    pub hash: String,
}

pub struct AuditService;

impl AuditService {

    pub fn init(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS audit_logs (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                action TEXT NOT NULL,
                target_id TEXT,
                outcome TEXT NOT NULL,
                metadata TEXT,
                prev_hash TEXT NOT NULL,
                hash TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn log_event(
        tx: &Transaction,
        action: &str,
        target_id: Option<&str>,
        outcome: &str,
        metadata: Option<&str>,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // 1. Get previous hash
        let prev_hash: String = tx.query_row(
            "SELECT hash FROM audit_logs ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000000".to_string());

        // 2. Compute current hash
        // Hash = SHA256(timestamp + action + target + outcome + metadata + prev_hash)
        let payload = format!(
            "{}{}{}{}{}{}",
            timestamp,
            action,
            target_id.unwrap_or(""),
            outcome,
            metadata.unwrap_or(""),
            prev_hash
        );
        let hash = CryptoService::hash_sha256(payload.as_bytes());
        let hash_hex = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();

        // 3. Insert
        tx.execute(
            "INSERT INTO audit_logs (timestamp, action, target_id, outcome, metadata, prev_hash, hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                timestamp,
                action,
                target_id,
                outcome,
                metadata,
                prev_hash,
                hash_hex
            ],
        )?;

        Ok(())
    }

    pub fn verify_integrity(conn: &Connection) -> Result<bool> {
        let mut stmt = conn.prepare("SELECT id, timestamp, action, target_id, outcome, metadata, prev_hash, hash FROM audit_logs ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(AuditLogEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                action: row.get(2)?,
                target_id: row.get(3)?,
                outcome: row.get(4)?,
                metadata: row.get(5)?,
                prev_hash: row.get(6)?,
                hash: row.get(7)?,
            })
        })?;

        let mut calculated_prev_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

        for row in rows {
            let entry = row?;

            // 1. Check chain link
            if entry.prev_hash != calculated_prev_hash {
                println!("Integrity Broken at ID {}: prev_hash mismatch", entry.id);
                return Ok(false);
            }

            // 2. Re-compute hash
            let payload = format!(
                "{}{}{}{}{}{}",
                entry.timestamp,
                entry.action,
                entry.target_id.as_deref().unwrap_or(""),
                entry.outcome,
                entry.metadata.as_deref().unwrap_or(""),
                entry.prev_hash
            );
            let calculated_hash = CryptoService::hash_sha256(payload.as_bytes());

            let calculated_hash_hex = calculated_hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
            
            if entry.hash != calculated_hash_hex {
                println!("Integrity Broken at ID {}: hash mismatch", entry.id);
                return Ok(false);
            }

            calculated_prev_hash = entry.hash;
        }

        Ok(true)
    }
}

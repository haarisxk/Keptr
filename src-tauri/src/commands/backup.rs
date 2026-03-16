use tauri::State;
use crate::state::AppState;
use crate::security::{SecretString, CryptoService, KeyHierarchy};
use crate::services::share_service::ShareService;
use std::time::Instant;

#[tauri::command]
pub async fn create_backup_shares(
    password: String,
    total_shares: u8,
    threshold: u8,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    // 1. Verify Password & Get Salt
    let salt = state.db.get_metadata("salt")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault not initialized")?;

    let secure_pass = SecretString::new(password);
    
    // 2. Derive L0 Master Key and Verify (Implicitly via L1 derivation check)
    // We need to verify the password is correct before generating shares.
    // The easiest way is to try to unlock the vault (derive L1 and check canary).
    // But ShareService calculates MasterKey internally.
    // Let's just generate the shares. If password is wrong, shares will be "valid" but reconstruct a wrong key.
    // This is bad. We MUST verify password first.
    
    // VERIFICATION STEP:
    let hierarchy = KeyHierarchy::derive(&secure_pass, &salt)
        .map_err(|_: crate::security::CryptoError| "Invalid Password".to_string())?;
        
    let canary_json = state.db.get_metadata("canary")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault corrupted")?;
        
    let canary_encrypted: crate::security::EncryptedData = serde_json::from_str(&canary_json)
        .map_err(|e| format!("Canary parse error: {}", e))?;

    if CryptoService::decrypt_xchacha20(&canary_encrypted, &hierarchy.auth_key).is_err() {
        return Err("Invalid Password".to_string());
    }

    // 3. Generate Shares
    // Note: this re-derives the master key inside ShareService. Ideally we'd pass the verified one,
    // but for separation of concerns, re-deriving is acceptable (Argon2 is slow but 2x is fine for this rare op).
    let shares = ShareService::generate_shares(&secure_pass, &salt, threshold, total_shares)
        .map_err(|e: String| e.to_string())?;

    // 4. Return as Strings
    Ok(shares.iter().map(|s: &SecretString| s.exposed().to_string()).collect())
}

#[tauri::command]
pub async fn recover_vault(
    shares: Vec<String>,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // 1. Reconstruct Master Key
    let master_key = ShareService::recover_master_key(&shares)
        .map_err(|e| format!("Recovery failed: {}", e))?;

    // 2. Derive Hierarchy from Master Key
    let hierarchy = KeyHierarchy::derive_from_master_key(&master_key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // 3. Verify against Canary to ensure shares were correct
    let canary_json = state.db.get_metadata("canary")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault corrupted (missing canary)")?;
        
    let canary_encrypted: crate::security::EncryptedData = serde_json::from_str(&canary_json)
        .map_err(|e| format!("Canary parse error: {}", e))?;

    match CryptoService::decrypt_xchacha20(&canary_encrypted, &hierarchy.auth_key) {
        Ok(bytes) if bytes == b"KEPTR_VERIFY" => {
            // Valid! Unlock the vault.
            {
                let mut key_guard = state.key_hierarchy.lock().unwrap();
                *key_guard = Some(hierarchy);
            }
            {
                let mut activity_guard = state.last_activity.lock().unwrap();
                *activity_guard = Instant::now();
            }
             state.db.log_auth_event("RECOVER_VAULT", "SUCCESS").ok();
             Ok(true)
        },
        _ => Err("Shares are invalid or do not belong to this vault.".to_string())
    }
}

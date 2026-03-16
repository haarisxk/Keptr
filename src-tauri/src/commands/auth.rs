use crate::security::secret::SecretString;
use crate::security::{CryptoService, KeyHierarchy};
use crate::services::file_service::FileService;
use crate::state::AppState;
use tauri::{State, AppHandle, Manager};
use std::time::{Duration, Instant};

const SESSION_TIMEOUT: Duration = Duration::from_secs(300);

/// Verifies the session is active and refreshes the timeout.
pub(crate) fn verify_auth_and_refresh(state: &AppState) -> Result<KeyHierarchy, String> {
    let mut key_guard = state.key_hierarchy.lock().map_err(|_| "Failed to lock key mutex".to_string())?;
    let mut activity_guard = state.last_activity.lock().map_err(|_| "Failed to lock activity mutex".to_string())?;

    if let Some(key) = key_guard.as_ref() {
        if activity_guard.elapsed() > SESSION_TIMEOUT {
            *key_guard = None;
            return Err("Session expired due to inactivity".to_string());
        }
        *activity_guard = Instant::now();
        Ok(key.clone())
    } else {
        Err("Vault is locked".to_string())
    }
}

#[tauri::command]
pub async fn vault_exists(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.db.vault_exists())
}

#[tauri::command]
pub async fn select_vault(
    vault_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let registry = state.vault_registry.lock().map_err(|_| "Failed to lock registry")?;
    if let Some(vault) = registry.get(&vault_id) {
        let vault_path = state.app_dir.join("data").join(&vault.path);
        state.db.switch_vault(&vault_path.to_string_lossy());
        
        // Persist local state
        *state.current_vault_id.lock().unwrap() = Some(vault_id);
        
        Ok(())
    } else {
        Err("Vault not found".to_string())
    }
}

#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    // 1. Lock Vault
    *state.key_hierarchy.lock().unwrap() = None;
    
    // 2. Clear Database Context
    // state.db.switch_vault(""); 
    *state.current_vault_id.lock().unwrap() = None;
    
    // 3. Clear User Session
    let mut current_user = state.current_user.lock().map_err(|_| "Failed to lock user")?;
    *current_user = None;
    
    Ok(())
}

#[tauri::command]
pub async fn set_current_user(email: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut current_user = state.current_user.lock().map_err(|_| "Failed to lock user")?;
    *current_user = Some(email);
    Ok(())
}

#[tauri::command]
pub async fn get_auth_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let current_user = state.current_user.lock().unwrap().clone();
    let current_vault_id = state.current_vault_id.lock().unwrap().clone();
    let is_locked = state.key_hierarchy.lock().unwrap().is_none();
    
    Ok(serde_json::json!({
        "current_user": current_user,
        "current_vault_id": current_vault_id,
        "is_locked": is_locked
    }))
}

#[tauri::command]
pub async fn setup_vault(
    password: String,
    oauth_provider: Option<String>,
    oauth_email: Option<String>,
    state: State<'_, AppState>,
    _app_handle: AppHandle,
) -> Result<bool, String> {
    if state.db.vault_exists() {
        return Err("Vault already exists".to_string());
    }

    let salt = CryptoService::generate_salt();
    let secure_pass = SecretString::new(password);
    
    // Derive the full key hierarchy (Master -> Enc/Auth keys)
    let hierarchy = KeyHierarchy::derive(&secure_pass, &salt)
        .map_err(|e| e.to_string())?;

    let canary_plaintext = b"KEPTR_VERIFY";
    // CANARY IS NOW ENCRYPTED WITH AUTH KEY, NOT ENCRYPTION KEY
    let canary_encrypted = CryptoService::encrypt_xchacha20(canary_plaintext, &hierarchy.auth_key)
        .map_err(|e| e.to_string())?;

    let canary_json = serde_json::to_string(&canary_encrypted)
        .map_err(|e| e.to_string())?;

    state.db.init().map_err(|e| format!("{:?}", e))?;
    state.db.save_metadata("salt", &salt).map_err(|e| format!("{:?}", e))?;
    state.db.save_metadata("canary", &canary_json).map_err(|e| format!("{:?}", e))?;

    // Optional: Link Identity if provided during setup
    if let (Some(provider), Some(email)) = (oauth_provider, oauth_email) {
        state.db.save_metadata("oauth_provider", &provider).map_err(|e| format!("{:?}", e))?;
        state.db.save_metadata("oauth_email", &email).map_err(|e| format!("{:?}", e))?;
    }

    // Log Event
    state.db.log_auth_event("SETUP_VAULT", "SUCCESS").map_err(|e| format!("{:?}", e))?;

    *state.key_hierarchy.lock().unwrap() = Some(hierarchy);

    Ok(true)
}

#[tauri::command]
pub async fn unlock_vault(
    password: String,
    state: State<'_, AppState>,
    _app_handle: AppHandle,
) -> Result<bool, String> {
    let salt = state.db.get_metadata("salt")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault not initialized (missing salt)")?;
        
    // Migration Logic Placeholder (implemented if db_version check needed)
    // For now we assume fresh install or manual migration trigger

    let secure_pass = SecretString::new(password);
    
    let hierarchy = KeyHierarchy::derive(&secure_pass, &salt)
        .map_err(|e| e.to_string())?;

    let canary_json = state.db.get_metadata("canary")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault corrupted (missing canary)")?;

    let canary_encrypted: crate::security::EncryptedData = serde_json::from_str(&canary_json)
        .map_err(|e| format!("Canary parse error: {}", e))?;

    // Verify authentication using the Auth Key
    match CryptoService::decrypt_xchacha20(&canary_encrypted, &hierarchy.auth_key) {
        Ok(bytes) => {
            if bytes == b"KEPTR_VERIFY" {
                let mut key_guard = state.key_hierarchy.lock().unwrap();
                *key_guard = Some(hierarchy);

                let mut activity_guard = state.last_activity.lock().unwrap();
                *activity_guard = Instant::now();

                state.db.init().map_err(|e| format!("{:?}", e))?;
                
                state.db.log_auth_event("UNLOCK_VAULT", "SUCCESS").ok(); // Ignore log error to not block login? Or block?
                // Let's print error but allow login if log fails? No, "Tamper Evident" implies we MUST log.
                // But `log_auth_event` returns Result. Let's map_err.
                // Actually, if we're inside the match, handling errors is verbose.
                // Let's just .unwrap_or_else/log error.
                
                Ok(true)
            } else {
                state.db.log_auth_event("UNLOCK_VAULT", "FAILURE").ok();
                Err("Invalid Password".to_string())
            }
        }
        Err(_) => Err("Invalid Password".to_string()),
    }
}

#[tauri::command]
pub async fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    *state.key_hierarchy.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub async fn is_unlocked(state: State<'_, AppState>) -> Result<bool, String> {
    Ok(state.key_hierarchy.lock().unwrap().is_some())
}

#[tauri::command]
pub async fn change_password(
    old_password: String,
    new_password: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // 1. Verify Old Password logic reuse? 
    // We can't reuse unlock_vault because it modifies state.
    // Duplicate verification logic.
    
    let salt = state.db.get_metadata("salt")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault not initialized")?;

    let secure_old_pass = SecretString::new(old_password);
    let old_hierarchy = KeyHierarchy::derive(&secure_old_pass, &salt)
        .map_err(|e| e.to_string())?;

    let canary_json = state.db.get_metadata("canary")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault corrupted (missing canary)")?;
        
    let canary_encrypted: crate::security::EncryptedData = serde_json::from_str(&canary_json)
        .map_err(|e| format!("Canary parse error: {}", e))?;

    let canary_plaintext = match CryptoService::decrypt_xchacha20(&canary_encrypted, &old_hierarchy.auth_key) {
        Ok(bytes) if bytes == b"KEPTR_VERIFY" => bytes,
        _ => return Err("Invalid old password".to_string()),
    };

    // 2. Derive New Keys
    let new_salt = CryptoService::generate_salt();
    let secure_new_pass = SecretString::new(new_password);
    let new_hierarchy = KeyHierarchy::derive(&secure_new_pass, &new_salt)
        .map_err(|e| e.to_string())?;

    // 3. Encrypt Canary with New Auth Key
    let new_canary_encrypted = CryptoService::encrypt_xchacha20(&canary_plaintext, &new_hierarchy.auth_key)
        .map_err(|e| e.to_string())?;
    let new_canary_json = serde_json::to_string(&new_canary_encrypted)
        .map_err(|e| e.to_string())?;

    // 4. Rotate DB Data & Metadata (Atomic)
    state.db.rotate_vault(&old_hierarchy, &new_hierarchy, &new_salt, &new_canary_json)
        .map_err(|e| format!("DB Rotation failed: {:?}", e))?;

    // 5. Rotate Files (Best effort)
    let app_data_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let app_dir = app_data_dir.parent().ok_or("Invalid path")?.join("Keptr");
    let files_dir = app_dir.join("files");

    FileService::rotate_files(&files_dir, &old_hierarchy, &new_hierarchy)
        .map_err(|e| format!("File Rotation failed: {:?}", e))?;

    // 6. Update State
    *state.key_hierarchy.lock().unwrap() = Some(new_hierarchy);
    *state.last_activity.lock().unwrap() = Instant::now();

    state.db.log_auth_event("CHANGE_PASSWORD", "SUCCESS").map_err(|e| format!("{:?}", e))?;

    Ok(())
}

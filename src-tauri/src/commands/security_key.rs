use crate::security::secret::SecretString;
use crate::security::{CryptoService, KeyHierarchy};
use crate::services::hardware_key::HardwareKeyService;
use crate::state::AppState;
use tauri::State;
use std::time::Instant;

#[tauri::command]
pub async fn register_hardware_key(
    password: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Verify Password & Derive Master Key
    // We need the salt to derive the key roughly the same way the vault setup did.
    let salt_b64 = state.db.get_metadata("salt")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault not initialized")?;

    let pepper = crate::security::pepper::get_pepper();
    // Default high security params used for master key
    let params = crate::security::argon2::Argon2Params::high_security(); 
    
    let secure_pass = SecretString::new(password);
    
    // We derive the L0 Master Key here.
    // Note: We use derive_key_with_pepper directly because KeyHierarchy::derive consumes the master key.
    // We need the raw master key to wrap it.
    let master_key = crate::security::argon2::derive_key_with_pepper(&secure_pass, &salt_b64, pepper, &params)
        .map_err(|e| e.to_string())?;

    // Verify correct password by checking canary? 
    // Yes, we should verify before replacing or adding key.
    // To verify, we must derive L1 keys and check canary.
    let hierarchy = KeyHierarchy::derive_from_master_key(&master_key)
        .map_err(|e| e.to_string())?;

    let canary_json = state.db.get_metadata("canary")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Vault corrupted (missing canary)")?;

    let canary_encrypted: crate::security::EncryptedData = serde_json::from_str(&canary_json)
        .map_err(|e| format!("Canary parse error: {}", e))?;

    match CryptoService::decrypt_xchacha20(&canary_encrypted, &hierarchy.auth_key) {
        Ok(bytes) if bytes == b"KEPTR_VERIFY" => { /* Valid */ },
        _ => return Err("Invalid Password".to_string()),
    }

    // 2. Register Device
    // This might prompt user interaction (touch)
    let credential = HardwareKeyService::register_device()
        .map_err(|e| e.to_string())?;

    // 3. Authenticate to get HDK (Hardware Derived Key)
    // We need the actual key to encrypt the master key.
    // The credential.salt is used as input.
    let hdk = HardwareKeyService::authenticate_and_derive(&credential.id, &credential.salt)
        .map_err(|e| format!("Verification failed (please touch device again): {}", e))?;

    // 4. Wrap Master Key
    let encrypted_mk = CryptoService::wrap_key(&master_key[..], &hdk)
        .map_err(|e| e.to_string())?;

    let encrypted_mk_json = serde_json::to_string(&encrypted_mk)
        .map_err(|e| e.to_string())?;

    // 5. Save Metadata
    state.db.save_metadata("hw_key_id", &credential.id).map_err(|e| format!("{:?}", e))?;
    state.db.save_metadata("hw_key_salt", &credential.salt).map_err(|e| format!("{:?}", e))?;
    state.db.save_metadata("hw_key_wrapped_mk", &encrypted_mk_json).map_err(|e| format!("{:?}", e))?;
    state.db.save_metadata("hw_key_enabled", "true").map_err(|e| format!("{:?}", e))?;

    state.db.log_auth_event("REGISTER_HardwareKey", "SUCCESS").ok();

    Ok(credential.id)
}

#[tauri::command]
pub async fn login_with_hardware_key(
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // 1. Get Metadata
    let cred_id = state.db.get_metadata("hw_key_id")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("No hardware key registered")?;
        
    let cred_salt = state.db.get_metadata("hw_key_salt")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Invalid hardware key state")?;
        
    let wrapped_mk_json = state.db.get_metadata("hw_key_wrapped_mk")
        .map_err(|e| format!("{:?}", e))?
        .ok_or("Invalid hardware key state")?;

    // 2. Derive HDK (Touch required)
    let hdk = HardwareKeyService::authenticate_and_derive(&cred_id, &cred_salt)
        .map_err(|e| e.to_string())?;

    // 3. Unwrap Master Key
    let encrypted_mk: crate::security::EncryptedData = serde_json::from_str(&wrapped_mk_json)
        .map_err(|e| e.to_string())?;
    
    let master_key = CryptoService::unwrap_key(&encrypted_mk, &hdk)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    // 4. Derive L1 Keys & Unlock
    let hierarchy = KeyHierarchy::derive_from_master_key(&master_key)
        .map_err(|e| e.to_string())?;
    
    // 5. Update State
    {
        let mut key_guard = state.key_hierarchy.lock().unwrap();
        *key_guard = Some(hierarchy);
    }
    
    {
        let mut activity_guard = state.last_activity.lock().unwrap();
        *activity_guard = Instant::now();
    }

    state.db.init().ok(); 
    state.db.log_auth_event("UNLOCK_HardwareKey", "SUCCESS").ok();

    Ok(true)
}

#[tauri::command]
pub async fn has_hardware_key(state: State<'_, AppState>) -> Result<bool, String> {
    let val = state.db.get_metadata("hw_key_enabled").map_err(|e| format!("{:?}", e))?;
    Ok(val.as_deref() == Some("true"))
}

use tauri::State;
use crate::state::AppState;
use crate::services::vault_registry::VaultMetadata;
use crate::security::{CryptoService, KeyHierarchy};
use crate::models::VaultItem;
use crate::commands::auth::verify_auth_and_refresh;
use uuid::Uuid;

#[tauri::command]
pub async fn list_vaults(state: State<'_, AppState>) -> Result<Vec<VaultMetadata>, String> {
    let registry = state.vault_registry.lock().map_err(|_| "Failed to lock registry")?;
    let current_user = state.current_user.lock().map_err(|_| "Failed to lock user")?;
    
    // Filter logic:
    // If logged in (Some(email)), show vaults owned by email AND vaults with no owner (Local)?
    // User requirement: "If not signed in [..] only vaults created while signed in should be visible" -> Confusing.
    // Let's stick to: "Show vaults for this user".
    // If user is None, show vaults with owner == None.
    
    let vaults = registry.list(current_user.as_deref());
    Ok(vaults)
}

#[tauri::command]
pub async fn create_vault(
    name: String, 
    password: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    // 1. Create entry in Registry
    let mut registry = state.vault_registry.lock().map_err(|_| "Failed to lock registry")?;
    let current_user = state.current_user.lock().map_err(|_| "Failed to lock user")?;
    
    // Enforcement: Offline users are limited to 1 vault
    if current_user.is_none() {
        let local_vaults = registry.list(None);
        if local_vaults.len() >= 1 {
            return Err("Offline users are limited to a single vault. Please sign in to create more.".to_string());
        }
    }
    
    let metadata = registry.add(&name, current_user.as_deref())
        .map_err(|e| format!("Registry error: {}", e))?;
        
    // 2. Initialize the Database file
    // We need to temporarily switch the StorageService to this new path, Init it, then maybe switch back?
    // Or just treat this as "Create and Select".
    let vault_path = state.app_dir.join("data").join(&metadata.path);
    let path_str = vault_path.to_string_lossy().to_string();
    
    // Lock DB path to switch it
    state.db.switch_vault(&path_str);
    
    // 3. Initialize DB Schema
    state.db.init().map_err(|e| format!("Failed to init DB: {:?}", e))?;
    
    // 4. Setup Security (Salt, Key Hierarchy, Canary)
    use crate::security::secret::SecretString;
    let secret_password = SecretString::new(password);
    
    let salt = CryptoService::generate_salt();
    let hierarchy = KeyHierarchy::derive(&secret_password, &salt)
        .map_err(|e| e.to_string())?;
        
    // Store Salt
    state.db.save_metadata("salt", &salt).map_err(|e| format!("{:?}", e))?;
    
    // Create Canary
    let canary_encrypted = CryptoService::encrypt_xchacha20(b"KEPTR_VERIFY", &hierarchy.auth_key)
        .map_err(|e| e.to_string())?;
    
    // Store Canary (as JSON string of EncryptedData)
    let canary_json = serde_json::to_string(&canary_encrypted).map_err(|e| e.to_string())?;
    state.db.save_metadata("canary", &canary_json).map_err(|e| format!("{:?}", e))?;
    
    // 5. Store Key Hierarchy in State (Login implicitly)
    let mut state_hierarchy = state.key_hierarchy.lock().map_err(|_| "Mutex poison")?;
    *state_hierarchy = Some(hierarchy);
    
    Ok(metadata.id)
}

#[tauri::command]
pub async fn delete_vault(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut registry = state.vault_registry.lock().map_err(|_| "Failed to lock registry")?;
    
    // Check if we are deleting the CURRENTLY selected vault
    let current_path = state.db.get_current_path();
    if let Some(vault) = registry.get(&id) {
        if current_path.ends_with(&vault.path) {
             // We are deleting the active vault.
             // We should close connection? StorageService doesn't allow "close".
             // We can switch to empty path.
             state.db.switch_vault("");
             
             // Clear memory
             let mut hierarchy = state.key_hierarchy.lock().map_err(|_| "Mutex poison")?;
             *hierarchy = None;
        }
    }
    
    registry.remove(&id).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Item Management Commands ---

use crate::services::sync_service::{SyncService, SyncPayload};

pub fn is_cloud_sync_enabled(state: &State<'_, AppState>) -> bool {
    if let Ok(Some(val)) = state.db.get_metadata("setting_cloud_sync_enabled") {
        if let Ok(parsed) = val.parse::<bool>() {
            return parsed;
        }
    }
    true // Default is true
}

#[tauri::command]
pub async fn create_vault_item(
    mut item: VaultItem, 
    state: State<'_, AppState>
) -> Result<String, String> {
    let hierarchy = verify_auth_and_refresh(&state)?;
    item.updated_at = chrono::Utc::now().to_rfc3339();
    if item.created_at.is_empty() {
        item.created_at = item.updated_at.clone();
    }
    
    // Save locally
    let mapped_id = state.db.create_item(&item, &hierarchy).map(|id| id.to_string()).map_err(|e| format!("{:?}", e))?;
    
    // Attempt Cloud Sync Push
    let current_vault = state.current_vault_id.lock().unwrap().clone();
    let cloud_user = state.cloud_user_id.lock().unwrap().clone();

    if is_cloud_sync_enabled(&state) {
        if let (Some(vault_id), Some(uid), Ok((data, nonce, _timestamps))) = (current_vault, cloud_user, state.db.get_encrypted_item(item.id)) {
            if let Ok(sync) = SyncService::new(state.cloud_session_token.lock().unwrap().clone()) {
                let payload = SyncPayload {
                    vault_id: vault_id.to_string(),
                    item_id: mapped_id.clone(),
                    encrypted_payload: data,
                    nonce,
                    last_modified: chrono::Utc::now().timestamp(),
                    user_id: uid,
                };
                tokio::spawn(async move {
                    if let Err(e) = sync.push_item(payload).await {
                        eprintln!("[Cloud Sync] Failed to push item: {}", e);
                    }
                });
            }
        }
    }

    Ok(mapped_id)
}

#[tauri::command]
pub async fn get_vault_items(state: State<'_, AppState>) -> Result<Vec<VaultItem>, String> {
    let hierarchy = verify_auth_and_refresh(&state)?;
    state.db.list_items(&hierarchy).map_err(|e| format!("{:?}", e))
}

#[tauri::command]
pub async fn update_vault_item(
    mut item: VaultItem, 
    state: State<'_, AppState>
) -> Result<(), String> {
    let hierarchy = verify_auth_and_refresh(&state)?;
    item.updated_at = chrono::Utc::now().to_rfc3339();
    
    // Save locally
    state.db.update_item(&item, &hierarchy).map_err(|e| format!("{:?}", e))?;
    
    // Attempt Cloud Sync Push
    let current_vault = state.current_vault_id.lock().unwrap().clone();
    let cloud_user = state.cloud_user_id.lock().unwrap().clone();

    if is_cloud_sync_enabled(&state) {
        if let (Some(vault_id), Some(uid), Ok((data, nonce, _timestamps))) = (current_vault, cloud_user, state.db.get_encrypted_item(item.id)) {
            if let Ok(sync) = SyncService::new(state.cloud_session_token.lock().unwrap().clone()) {
                let payload = SyncPayload {
                    vault_id: vault_id.to_string(),
                    item_id: item.id.to_string(),
                    encrypted_payload: data,
                    nonce,
                    last_modified: chrono::Utc::now().timestamp(),
                    user_id: uid,
                };
                tokio::spawn(async move {
                    let _ = sync.push_item(payload).await;
                });
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn delete_vault_item(
    id: String, 
    state: State<'_, AppState>
) -> Result<(), String> {
    let hierarchy = verify_auth_and_refresh(&state)?; 
    let uuid = Uuid::parse_str(&id).map_err(|_| "Invalid UUID".to_string())?;
    
    // Before deleting, check if this is a file-type item to clean up cloud storage
    let file_name_for_cloud: Option<String> = {
        if let Ok(items) = state.db.list_items(&hierarchy) {
            items.iter().find(|item| item.id == uuid).and_then(|item| {
                // Extract file_path from the flattened JSON data
                let json = serde_json::to_value(item).ok()?;
                let fp = json.get("file_path")?.as_str()?;
                let path = std::path::PathBuf::from(fp);
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
            })
        } else {
            None
        }
    };
    
    state.db.delete_item(uuid).map_err(|e| format!("{:?}", e))?;
    
    // Cloud sync: delete the item metadata AND the file blob (if applicable)
    let current_vault = state.current_vault_id.lock().unwrap().clone();
    let cloud_user = state.cloud_user_id.lock().unwrap().clone();

    if is_cloud_sync_enabled(&state) {
        if let (Some(vault_id), Some(uid)) = (current_vault, cloud_user) {
             if let Ok(sync) = SyncService::new(state.cloud_session_token.lock().unwrap().clone()) {
                let file_name = file_name_for_cloud.clone();
                let vid = vault_id.clone();
                let uid2 = uid.clone();
                tokio::spawn(async move {
                    // Delete item metadata from cloud
                    if let Err(e) = sync.delete_item(&vault_id, &id, &uid).await {
                        eprintln!("[Cloud Sync] Failed to hard-delete item {}: {}", id, e);
                    }
                    // Delete associated .kaps file blob from cloud storage
                    if let Some(fname) = file_name {
                        if let Err(e) = sync.delete_file(&uid2, &vid, &fname).await {
                            eprintln!("[Cloud Storage] Failed to delete file '{}': {}", fname, e);
                        }
                    }
                });
             }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn pull_sync_items(
    since_timestamp: i64, 
    state: State<'_, AppState>
) -> Result<i64, String> {
    // 1. Verify Authentication
    let _hierarchy = verify_auth_and_refresh(&state)?;
    
    // 2. Fetch the current vault context
    let current_vault = state.current_vault_id.lock().unwrap().clone();
    let vault_id = current_vault.ok_or_else(|| "No active vault selected.".to_string())?;

    if !is_cloud_sync_enabled(&state) {
        return Ok(since_timestamp); // Skip sync logic silently
    }

    // 3. Instantiate the Supabase Client
    let jwt = state.cloud_session_token.lock().unwrap().clone();
    let sync_service = SyncService::new(jwt).map_err(|e| format!("Network Configuration Error: {}", e))?;

    // 4. Fetch the Payload Deltas
    let payloads = sync_service.pull_updates(&vault_id, since_timestamp).await
        .map_err(|e| format!("Sync fetch failed: {}", e))?;
        
    if payloads.is_empty() {
        return Ok(since_timestamp); // Nothing to update
    }

    // 5. Native SQLite Merge
    let new_max_timestamp = payloads.iter().map(|p| p.last_modified).max().unwrap_or(since_timestamp);
    state.db.import_sync_payloads(payloads).map_err(|e| format!("{:?}", e))?;

    Ok(new_max_timestamp)
}

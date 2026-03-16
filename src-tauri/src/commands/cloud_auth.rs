use crate::services::cloud_auth::CloudAuthService;
use serde::Serialize;
use tauri::State;
use crate::state::AppState;

#[derive(Serialize)]
pub struct CloudAuthResponse {
    pub success: bool,
    pub access_token: Option<String>,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub error: Option<String>,
}

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default)]
struct CloudIdentity {
    pub session_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user_id: Option<String>,
    pub email: Option<String>,
    
    #[serde(default)]
    pub private_keys: HashMap<String, String>,
    #[serde(default)]
    pub public_keys: HashMap<String, String>,

    pub x25519_private_key: Option<String>,
    pub x25519_public_key: Option<String>,
}

fn load_full_identity(state: &State<'_, AppState>) -> CloudIdentity {
    let path = get_identity_path(state);
    let mut identity = CloudIdentity::default();
    
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(mut ident) = serde_json::from_str::<CloudIdentity>(&content) {
            // Migrate legacy keys if present
            if let (Some(uid), Some(priv_k), Some(pub_k)) = (&ident.user_id, ident.x25519_private_key.take(), ident.x25519_public_key.take()) {
                ident.private_keys.insert(uid.clone(), priv_k);
                ident.public_keys.insert(uid.clone(), pub_k);
            }
            identity = ident;
        }
    }
    identity
}

pub fn get_local_x25519_keys(app_dir: &std::path::PathBuf, target_user_id: &str) -> Result<(String, String), String> {
    let path = app_dir.join("identity.json");
    if let Ok(content) = fs::read_to_string(path) {
        #[derive(serde::Deserialize)]
        struct LocalIdent { 
            #[serde(default)]
            private_keys: std::collections::HashMap<String, String>,
            #[serde(default)]
            public_keys: std::collections::HashMap<String, String>,
            x25519_private_key: Option<String>,
            x25519_public_key: Option<String>,
            user_id: Option<String>,
        }
        
        if let Ok(ident) = serde_json::from_str::<LocalIdent>(&content) {
            if let (Some(priv_k), Some(pub_k)) = (ident.private_keys.get(target_user_id), ident.public_keys.get(target_user_id)) {
                return Ok((priv_k.clone(), pub_k.clone()));
            }
            // Legacy fallback
            if let Some(uid) = ident.user_id {
                if uid == target_user_id {
                    if let (Some(priv_k), Some(pub_k)) = (ident.x25519_private_key, ident.x25519_public_key) {
                        return Ok((priv_k, pub_k));
                    }
                }
            }
        }
    }
    Err("No X25519 keys generated for this local device identity.".to_string())
}

fn handle_auth_success(
    state: &State<'_, AppState>,
    access_token: String,
    refresh_token: String,
    user_id: String,
    email: String,
) {
    *state.cloud_session_token.lock().unwrap() = Some(access_token.clone());
    *state.cloud_refresh_token.lock().unwrap() = Some(refresh_token.clone());
    *state.cloud_user_id.lock().unwrap() = Some(user_id.clone());
    *state.current_user.lock().unwrap() = Some(email.clone());

    let mut identity = load_full_identity(state);
    
    identity.session_token = Some(access_token.clone());
    identity.refresh_token = Some(refresh_token.clone());
    identity.user_id = Some(user_id.clone());
    identity.email = Some(email.clone());

    if !identity.private_keys.contains_key(&user_id) || !identity.public_keys.contains_key(&user_id) {
        let (priv_key, pub_key) = crate::security::asymmetric::generate_x25519_keypair();
        identity.private_keys.insert(user_id.clone(), priv_key);
        identity.public_keys.insert(user_id.clone(), pub_key.clone());

        let sync_pub = pub_key.clone();
        let sync_uid = user_id.clone();
        let sync_token = access_token.clone();
        
        tokio::spawn(async move {
            if let Ok(sync) = crate::services::sync_service::SyncService::new(Some(sync_token)) {
                if let Err(e) = sync.publish_public_key(&sync_uid, &sync_pub).await {
                    eprintln!("[X25519] Failed to publish public key: {}", e);
                }
            }
        });
    } else {
        // Enforce DB Key parities on fresh logins just in case Cloud DB gets reset
        let sync_pub = identity.public_keys.get(&user_id).unwrap().clone();
        let sync_uid = user_id.clone();
        let sync_token = access_token.clone();
        tokio::spawn(async move {
            if let Ok(sync) = crate::services::sync_service::SyncService::new(Some(sync_token)) {
                 let _ = sync.publish_public_key(&sync_uid, &sync_pub).await;
            }
        });
    }
    
    save_identity(state, &identity);
}

#[tauri::command]
pub async fn cloud_signup(
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<CloudAuthResponse, String> {
    let service = CloudAuthService::new().map_err(|e| e.to_string())?;

    match service.sign_up(&email, &password).await {
        Ok(resp) => {
            handle_auth_success(&state, resp.access_token.clone(), resp.refresh_token.clone(), resp.user.id.clone(), resp.user.email.clone());
            
            Ok(CloudAuthResponse {
                success: true,
                access_token: Some(resp.access_token),
                user_id: Some(resp.user.id),
                email: Some(resp.user.email),
                error: None,
            })
        },
        Err(e) => {
            Ok(CloudAuthResponse {
                success: false,
                access_token: None,
                user_id: None,
                email: None,
                error: Some(e),
            })
        }
    }
}

#[tauri::command]
pub async fn cloud_signin(
    email: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<CloudAuthResponse, String> {
    let service = CloudAuthService::new().map_err(|e| e.to_string())?;

    match service.sign_in(&email, &password).await {
        Ok(resp) => {
            handle_auth_success(&state, resp.access_token.clone(), resp.refresh_token.clone(), resp.user.id.clone(), resp.user.email.clone());

            Ok(CloudAuthResponse {
                success: true,
                access_token: Some(resp.access_token),
                user_id: Some(resp.user.id),
                email: Some(resp.user.email),
                error: None,
            })
        },
        Err(e) => {
            Ok(CloudAuthResponse {
                success: false,
                access_token: None,
                user_id: None,
                email: None,
                error: Some(e),
            })
        }
    }
}

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;



fn get_identity_path(state: &State<'_, AppState>) -> PathBuf {
    state.app_dir.join("identity.json")
}

fn save_identity(state: &State<'_, AppState>, identity: &CloudIdentity) {
    if let Ok(json) = serde_json::to_string(identity) {
        let _ = fs::write(get_identity_path(state), json);
    }
}

pub fn load_identity(app_dir: &std::path::PathBuf) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let path = app_dir.join("identity.json");
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(ident) = serde_json::from_str::<CloudIdentity>(&content) {
            return (ident.session_token, ident.refresh_token, ident.user_id, ident.email);
        }
    }
    (None, None, None, None)
}

#[tauri::command]
pub async fn cloud_logout(state: State<'_, AppState>) -> Result<(), String> {
    *state.cloud_session_token.lock().unwrap() = None;
    *state.cloud_refresh_token.lock().unwrap() = None;
    *state.cloud_user_id.lock().unwrap() = None;
    *state.current_user.lock().unwrap() = None;
    
    // Clear session tokens but PRESERVE the keys and user_id!
    let mut identity = load_full_identity(&state);
    identity.session_token = None;
    identity.refresh_token = None;
    save_identity(&state, &identity);
    
    Ok(())
}

#[tauri::command]
pub async fn get_cloud_auth_state(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let session = state.cloud_session_token.lock().unwrap();
    if session.is_some() {
        Ok(Some("Authenticated".to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn cloud_oauth_signin(
    provider: String,
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<CloudAuthResponse, String> {
    let resp = crate::services::oauth::start_supabase_oauth(window, &provider).await?;
    
    if resp.success {
        if let (Some(token), Some(id), Some(email)) = (&resp.access_token, &resp.user_id, &resp.email) {
            // OAuth may not always return a refresh_token in the redirect hash
            let refresh = resp.refresh_token.clone().unwrap_or_default();
            handle_auth_success(&state, token.clone(), refresh, id.clone(), email.clone());
        }
    }
    
    Ok(CloudAuthResponse {
        success: resp.success,
        access_token: resp.access_token,
        user_id: resp.user_id,
        email: resp.email,
        error: resp.error,
    })
}

#[tauri::command]
pub async fn delete_account(state: State<'_, AppState>) -> Result<(), String> {
    let email = state.current_user.lock().unwrap().clone();
    let user_id = state.cloud_user_id.lock().unwrap().clone();
    let token = state.cloud_session_token.lock().unwrap().clone();

    // 1. Delete from Supabase if we have the credentials
    if let (Some(uid), Some(jwt)) = (user_id, token) {
        if let Ok(sync_service) = crate::services::sync_service::SyncService::new(Some(jwt)) {
            let _ = sync_service.delete_all_user_data(&uid).await;
        }
    }

    // 2. Delete all local Vaults (.kore) owned by the user
    if let Some(user_email) = &email {
        let mut to_delete = Vec::new();
        {
            let registry = state.vault_registry.lock().unwrap();
            for vault in registry.list(Some(user_email)) {
                to_delete.push(vault.id.clone());
            }
        }
        
        let mut registry = state.vault_registry.lock().unwrap();
        for id in to_delete {
            let _ = registry.remove(&id);
        }
    }

    // 3. (REMOVED) Explicitly preserving .kaps inside /files and .kept inside /backups as requested.

    // 4. Lock application and reset state
    *state.key_hierarchy.lock().unwrap() = None;
    *state.current_vault_id.lock().unwrap() = None;
    
    // 5. Clear global session identity mapping
    *state.cloud_session_token.lock().unwrap() = None;
    *state.cloud_refresh_token.lock().unwrap() = None;
    *state.cloud_user_id.lock().unwrap() = None;
    *state.current_user.lock().unwrap() = None;
    
    save_identity(&state, &CloudIdentity::default());

    Ok(())
}

/// Silently refreshes the cloud session using the stored refresh token.
/// Called by the background daemon or manually by the frontend.
#[tauri::command]
pub async fn refresh_cloud_session(state: State<'_, AppState>) -> Result<(), String> {
    let refresh_token = state.cloud_refresh_token.lock().unwrap().clone();
    let refresh_token = refresh_token
        .ok_or_else(|| "No refresh token available. Please sign in again.".to_string())?;

    let service = CloudAuthService::new().map_err(|e| e.to_string())?;
    let resp = service.refresh_session(&refresh_token).await?;

    // Update in-memory state with fresh tokens
    *state.cloud_session_token.lock().unwrap() = Some(resp.access_token.clone());
    *state.cloud_refresh_token.lock().unwrap() = Some(resp.refresh_token.clone());

    // Persist to identity.json
    let mut identity = load_full_identity(&state);
    identity.session_token = Some(resp.access_token);
    identity.refresh_token = Some(resp.refresh_token);
    save_identity(&state, &identity);

    println!("[Token Refresh] Session refreshed successfully.");
    Ok(())
}

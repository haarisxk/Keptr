#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use keptr_crypto::memory::SecretBytes;
use keptr_crypto::kdf::derive_master_key;
use keptr_crypto::mac::{compute_hmac, verify_hmac};
use keptr_store::db::SecureStore;
use std::sync::Mutex;
use tauri::State;

struct AppState {
    vault_key: Mutex<Option<SecretBytes>>,
    store: Mutex<SecureStore>,
}

#[tauri::command]
fn initialize_vault(password: String, state: State<'_, AppState>) -> Result<(), String> {
    // 1. Derive new master key and salt
    let (key, salt) = derive_master_key(&SecretBytes::new(password.into_bytes()), None)?;
    
    // 2. Compute a MAC over a known constant to use for password verification later
    let verification_mac = compute_hmac(b"KEPTR_VERIFICATION", &key).map_err(|e| e.to_string())?;
    let mac_hex = hex::encode(verification_mac);
    
    // 3. Save to database
    let store = state.store.lock().unwrap();
    store.set_metadata("kdf_salt", &salt).map_err(|e| e.to_string())?;
    store.set_metadata("password_mac", &mac_hex).map_err(|e| e.to_string())?;
    
    // 4. Save key in memory
    let mut vault_key = state.vault_key.lock().unwrap();
    *vault_key = Some(key);
    
    Ok(())
}

#[tauri::command]
fn unlock_vault(password: String, state: State<'_, AppState>) -> Result<(), String> {
    let store = state.store.lock().unwrap();
    
    let salt = store.get_metadata("kdf_salt").map_err(|e| e.to_string())?.ok_or("Vault not initialized")?;
    let expected_mac_hex = store.get_metadata("password_mac").map_err(|e| e.to_string())?.ok_or("Vault not initialized")?;
    let expected_mac = hex::decode(expected_mac_hex).map_err(|e| e.to_string())?;
    
    // Derive key
    let (key, _) = derive_master_key(&SecretBytes::new(password.into_bytes()), Some(&salt))?;
    
    // Verify password
    verify_hmac(b"KEPTR_VERIFICATION", &expected_mac, &key).map_err(|_| "Incorrect master password".to_string())?;
    
    // Store in memory
    let mut vault_key = state.vault_key.lock().unwrap();
    *vault_key = Some(key);
    
    Ok(())
}

#[tauri::command]
fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_key = state.vault_key.lock().unwrap();
    *vault_key = None;
    Ok(())
}

#[tauri::command]
fn check_vault_status(state: State<'_, AppState>) -> Result<VaultStatus, String> {
    let is_unlocked = state.vault_key.lock().unwrap().is_some();
    let is_initialized = state.store.lock().unwrap().get_metadata("kdf_salt").map_err(|e| e.to_string())?.is_some();
    
    Ok(VaultStatus {
        is_initialized,
        is_unlocked,
    })
}

#[derive(serde::Serialize)]
struct VaultStatus {
    is_initialized: bool,
    is_unlocked: bool,
}

fn main() {
    let app_dir = std::env::current_dir().unwrap(); // For testing/dev
    let db_path = app_dir.join("vault.db");
    
    let store = SecureStore::open(db_path.to_str().unwrap(), "dummy_key").expect("Failed to open DB");
    
    let app_state = AppState {
        vault_key: Mutex::new(None),
        store: Mutex::new(store),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            initialize_vault,
            unlock_vault,
            lock_vault,
            check_vault_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

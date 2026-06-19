#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use keptr_crypto::memory::SecretBytes;
use keptr_crypto::kdf::derive_master_key;
use keptr_crypto::mac::{compute_hmac, verify_hmac};
use keptr_store::db::SecureStore;
use keptr_core::models::{ItemType, LoginItem};
use keptr_core::vault::{create_kore_item, decrypt_kore_item};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
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
    let verification_mac = compute_hmac(&key, b"KEPTR_VERIFICATION").map_err(|e| e.to_string())?;
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
    verify_hmac(&key, b"KEPTR_VERIFICATION", &expected_mac).map_err(|_| "Incorrect master password".to_string())?;
    
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

#[derive(serde::Deserialize)]
struct AddLoginPayload {
    name: String,
    url: String,
    username: String,
    password_str: String,
    notes: String,
}

#[derive(serde::Serialize)]
struct LoginItemDto {
    id: String,
    name: String,
    url: String,
    username: String,
    password_str: String,
    notes: String,
}

#[tauri::command]
fn add_item(payload: AddLoginPayload, state: State<'_, AppState>) -> Result<(), String> {
    let vault_key = state.vault_key.lock().unwrap();
    let master_key = vault_key.as_ref().ok_or("Vault is locked")?;

    let login_item = LoginItem {
        name: payload.name,
        url: payload.url,
        username: payload.username,
        password: payload.password_str.into_bytes(),
        totp: None,
        notes: payload.notes,
    };

    let plaintext = serde_json::to_vec(&login_item).map_err(|e| e.to_string())?;

    let encrypted_item = create_kore_item(ItemType::Login, master_key, &plaintext)?;

    let item_id = format!("item_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis());
    
    let store = state.store.lock().unwrap();
    store.save_kore_item(&item_id, ItemType::Login as u8, &encrypted_item).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_items(state: State<'_, AppState>) -> Result<Vec<LoginItemDto>, String> {
    let vault_key = state.vault_key.lock().unwrap();
    let master_key = vault_key.as_ref().ok_or("Vault is locked")?;

    let store = state.store.lock().unwrap();
    let all_encrypted = store.get_all_items().map_err(|e| e.to_string())?;

    let mut dtos = Vec::new();

    for (id, encrypted_item) in all_encrypted {
        if encrypted_item.header.item_type == ItemType::Login {
            let plaintext_bytes = decrypt_kore_item(&encrypted_item, master_key)?;
            if let Ok(login_item) = serde_json::from_slice::<LoginItem>(plaintext_bytes.as_bytes()) {
                dtos.push(LoginItemDto {
                    id,
                    name: login_item.name,
                    url: login_item.url,
                    username: login_item.username,
                    password_str: String::from_utf8(login_item.password).unwrap_or_default(),
                    notes: login_item.notes,
                });
            }
        }
    }

    Ok(dtos)
}

#[tauri::command]
fn update_item(id: String, payload: AddLoginPayload, state: State<'_, AppState>) -> Result<(), String> {
    let vault_key = state.vault_key.lock().unwrap();
    let master_key = vault_key.as_ref().ok_or("Vault is locked")?;

    let login_item = LoginItem {
        name: payload.name,
        url: payload.url,
        username: payload.username,
        password: payload.password_str.into_bytes(),
        totp: None,
        notes: payload.notes,
    };

    let plaintext = serde_json::to_vec(&login_item).map_err(|e| e.to_string())?;

    let encrypted_item = create_kore_item(ItemType::Login, master_key, &plaintext)?;
    
    let store = state.store.lock().unwrap();
    store.save_kore_item(&id, ItemType::Login as u8, &encrypted_item).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn delete_item(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let vault_key = state.vault_key.lock().unwrap();
    let _ = vault_key.as_ref().ok_or("Vault is locked")?;

    let store = state.store.lock().unwrap();
    store.delete_item(&id).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(serde::Deserialize)]
struct PasswordOptions {
    length: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
}

#[tauri::command]
fn generate_secure_password(options: PasswordOptions) -> Result<String, String> {
    use rand::Rng;
    let mut rng = rand::rngs::OsRng;
    
    let mut charset = String::new();
    if options.uppercase { charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ"); }
    if options.lowercase { charset.push_str("abcdefghijklmnopqrstuvwxyz"); }
    if options.numbers { charset.push_str("0123456789"); }
    if options.symbols { charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?"); }
    
    if charset.is_empty() {
        return Err("Must select at least one character set".to_string());
    }

    let chars: Vec<char> = charset.chars().collect();
    let password: String = (0..options.length)
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx]
        })
        .collect();
        
    Ok(password)
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
            check_vault_status,
            add_item,
            get_items,
            update_item,
            delete_item,
            generate_secure_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

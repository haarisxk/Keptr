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
enum PasswordType {
    Chars,
    Passphrase,
    Pronounceable,
}

#[derive(serde::Deserialize)]
struct AdvancedPasswordOptions {
    pwd_type: PasswordType,
    length: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
    exclude_ambiguous: bool,
    word_count: usize,
    separator: String,
}

#[tauri::command]
fn generate_advanced_password(options: AdvancedPasswordOptions) -> Result<String, String> {
    use rand::Rng;
    let mut rng = rand::rngs::OsRng;
    
    match options.pwd_type {
        PasswordType::Chars => {
            let mut charset = String::new();
            
            let mut upper = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            let mut lower = "abcdefghijklmnopqrstuvwxyz";
            let mut nums = "0123456789";
            let mut syms = "!@#$%^&*()_+-=[]{}|;:,.<>?";
            
            if options.exclude_ambiguous {
                upper = "ABCDEFGHJKLMNPQRSTUVWXYZ"; // removed I, O
                lower = "abcdefghijkmnopqrstuvwxyz"; // removed l
                nums = "23456789"; // removed 1, 0
            }

            if options.uppercase { charset.push_str(upper); }
            if options.lowercase { charset.push_str(lower); }
            if options.numbers { charset.push_str(nums); }
            if options.symbols { charset.push_str(syms); }
            
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
        },
        PasswordType::Passphrase => {
            let wordlist = include_str!("eff_large_wordlist.txt");
            let words: Vec<&str> = wordlist.lines()
                .filter_map(|line| line.split_whitespace().nth(1))
                .collect();
                
            if words.is_empty() {
                return Err("Failed to load wordlist".to_string());
            }
            
            let mut phrase_words = Vec::new();
            for _ in 0..options.word_count {
                let idx = rng.gen_range(0..words.len());
                phrase_words.push(words[idx]);
            }
            
            Ok(phrase_words.join(&options.separator))
        },
        PasswordType::Pronounceable => {
            let consonants = vec!['b','c','d','f','g','h','j','k','l','m','n','p','r','s','t','v','w','y','z'];
            let vowels = vec!['a','e','i','o','u'];
            let mut pwd = String::new();
            let mut is_consonant = rng.gen_bool(0.5);
            
            for _ in 0..options.length {
                if is_consonant {
                    let idx = rng.gen_range(0..consonants.len());
                    pwd.push(consonants[idx]);
                } else {
                    let idx = rng.gen_range(0..vowels.len());
                    pwd.push(vowels[idx]);
                }
                is_consonant = !is_consonant;
            }
            
            Ok(pwd)
        }
    }
}

#[derive(serde::Deserialize)]
enum UsernameType {
    RandomChars,
    Words,
    EmailAlias,
    CatchAll,
}

#[derive(serde::Deserialize)]
struct UsernameOptions {
    usr_type: UsernameType,
    length: usize,
    word_count: usize,
    separator: String,
    base_email: String,
    alias_prefix: String,
    domain: String,
}

#[tauri::command]
fn generate_username(options: UsernameOptions) -> Result<String, String> {
    use rand::Rng;
    let mut rng = rand::rngs::OsRng;

    match options.usr_type {
        UsernameType::RandomChars => {
            let charset = "abcdefghijklmnopqrstuvwxyz0123456789";
            let chars: Vec<char> = charset.chars().collect();
            let mut username = String::new();
            for _ in 0..options.length {
                let idx = rng.gen_range(0..chars.len());
                username.push(chars[idx]);
            }
            Ok(username)
        },
        UsernameType::Words => {
            let wordlist = include_str!("eff_large_wordlist.txt");
            let words: Vec<&str> = wordlist.lines()
                .filter_map(|line| line.split_whitespace().nth(1))
                .collect();
            let mut phrase_words = Vec::new();
            for _ in 0..options.word_count {
                let idx = rng.gen_range(0..words.len());
                phrase_words.push(words[idx]);
            }
            Ok(phrase_words.join(&options.separator))
        },
        UsernameType::EmailAlias => {
            let parts: Vec<&str> = options.base_email.split('@').collect();
            if parts.len() != 2 {
                return Err("Invalid base email format. Must contain '@'".to_string());
            }
            let user = parts[0];
            let domain = parts[1];
            
            let charset = "abcdefghijklmnopqrstuvwxyz0123456789";
            let chars: Vec<char> = charset.chars().collect();
            let mut rnd = String::new();
            for _ in 0..6 {
                let idx = rng.gen_range(0..chars.len());
                rnd.push(chars[idx]);
            }
            
            let prefix = if options.alias_prefix.is_empty() { String::new() } else { format!("{}_", options.alias_prefix) };
            
            Ok(format!("{}+{}{}@{}", user, prefix, rnd, domain))
        },
        UsernameType::CatchAll => {
            let wordlist = include_str!("eff_large_wordlist.txt");
            let words: Vec<&str> = wordlist.lines()
                .filter_map(|line| line.split_whitespace().nth(1))
                .collect();
            
            let mut phrase_words = Vec::new();
            for _ in 0..2 {
                let idx = rng.gen_range(0..words.len());
                phrase_words.push(words[idx]);
            }
            let prefix = phrase_words.join(&options.separator);
            
            let mut dom = options.domain;
            if dom.starts_with('@') {
                dom = dom[1..].to_string();
            }
            if dom.is_empty() {
                return Err("Domain cannot be empty".to_string());
            }
            Ok(format!("{}@{}", prefix, dom))
        }
    }
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
            generate_advanced_password,
            generate_username
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

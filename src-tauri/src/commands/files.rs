use crate::services::file_service::FileService;
use crate::services::sync_service::SyncService;
use crate::state::AppState;
use crate::commands::vault::is_cloud_sync_enabled;
use tauri::State;
use std::path::PathBuf;

use super::auth::verify_auth_and_refresh;

/// File extensions that are blocked from encryption for security reasons.
/// These include executables, scripts, and other high-risk formats that could
/// be used to reverse-engineer the software or compromise user security.
const BLOCKED_EXTENSIONS: &[&str] = &[
    // Windows executables & installers
    "exe", "msi", "bat", "cmd", "com", "scr", "pif",
    // Script files
    "ps1", "psm1", "psd1", "vbs", "vbe", "js", "jse", "ws", "wsf", "wsc", "wsh",
    // Shell / Unix
    "sh", "bash", "csh", "ksh",
    // Compiled / bytecode
    "dll", "sys", "drv", "ocx", "cpl",
    // Macro-enabled Office (can contain malicious macros)
    "docm", "xlsm", "pptm", "dotm", "xltm", "potm",
    // Java / .NET
    "jar", "class", "msp", "mst",
    // Shortcuts & links
    "lnk", "url", "scf",
    // Registry & config
    "reg", "inf",
    // Disk images (can contain anything)
    "iso", "img", "vhd", "vhdx",
    // Other risky
    "appx", "msix", "appxbundle", "cab",
];

#[tauri::command]
pub async fn import_file(
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let hierarchy = verify_auth_and_refresh(&state)?;

    // Validate file extension against blocklist
    let file_path = PathBuf::from(&path);
    if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if BLOCKED_EXTENSIONS.contains(&ext_lower.as_str()) {
            return Err(format!(
                "File type '.{}' is not allowed for security reasons. Executables, scripts, and other high-risk formats cannot be encrypted.",
                ext_lower
            ));
        }
    }

    let files_dir = state.app_dir.join("files");
    if !files_dir.exists() {
        std::fs::create_dir_all(&files_dir).map_err(|e| e.to_string())?;
    }

    let saved_path = FileService::save_attachment(
        &PathBuf::from(&path),
        &files_dir,
        &hierarchy,
    )
    .map_err(|e| format!("{:?}", e))?;

    // ── Cloud File Backup (fire-and-forget) ──────────────────
    // After saving the encrypted .kaps locally, upload the blob
    // to Supabase Storage in the background so it survives reinstalls.
    if is_cloud_sync_enabled(&state) {
        let vault_id = state.current_vault_id.lock().unwrap().clone();
        let user_id = state.cloud_user_id.lock().unwrap().clone();
        let jwt = state.cloud_session_token.lock().unwrap().clone();

        if let (Some(vid), Some(uid)) = (vault_id, user_id) {
            // Read the encrypted .kaps bytes from disk
            if let Ok(encrypted_bytes) = std::fs::read(&saved_path) {
                let file_name = saved_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown.kaps")
                    .to_string();

                tokio::spawn(async move {
                    match SyncService::new(jwt) {
                        Ok(sync) => {
                            if let Err(e) = sync.upload_file(&uid, &vid, &file_name, encrypted_bytes).await {
                                eprintln!("[Cloud Storage] Background upload failed for '{}': {}", file_name, e);
                            }
                        }
                        Err(e) => eprintln!("[Cloud Storage] Failed to init sync service: {}", e),
                    }
                });
            }
        }
    }

    Ok(saved_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn create_full_backup(
    destination: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let hierarchy = verify_auth_and_refresh(&state)?;

    let vault_path_str = state.db.get_current_path();
    let vault_path = std::path::PathBuf::from(vault_path_str);
    let files_dir = state.app_dir.join("files");

    FileService::create_backup(
        &vault_path,
        &files_dir,
        &std::path::PathBuf::from(destination),
        &hierarchy,
    )
    .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn import_backup(
    source: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let hierarchy = verify_auth_and_refresh(&state)?;
    
    // 1. Decrypt ZIP archive
    let backup_path = std::path::PathBuf::from(source);
    let zip_bytes = FileService::restore_backup(&backup_path, &hierarchy)
        .map_err(|e| format!("Failed to decrypt backup: {:?}", e))?;
        
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))
        .map_err(|e| format!("Failed to read ZIP archive: {:?}", e))?;
        
    let files_dir = state.app_dir.join("files");
    if !files_dir.exists() {
        std::fs::create_dir_all(&files_dir).map_err(|e| format!("IO: {}", e))?;
    }
    
    let db_path_str = state.db.get_current_path();
    let current_db_path = std::path::Path::new(&db_path_str);
    
    // 2. Extract and process files
    let mut imported_items = 0;
    let mut skipped_items = 0;
    
    // We need a temp path for the db
    let temp_kore_path = state.app_dir.join(format!("import_tmp_{}.kore", uuid::Uuid::new_v4()));
    
    let mut has_kore = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Zip Error: {}", e))?;
        let name = file.name().to_string();
        
        if name == "vault.kore" {
            let mut out = std::fs::File::create(&temp_kore_path).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
            has_kore = true;
        } else if name.starts_with("attachments/") {
            let file_name = std::path::Path::new(&name).file_name().unwrap();
            let dest_path = files_dir.join(file_name);
            if !dest_path.exists() {
                let mut out = std::fs::File::create(&dest_path).map_err(|e| format!("IO: {}", e))?;
                std::io::copy(&mut file, &mut out).ok();
            }
        }
    }
    
    // 3. Merge Databases
    if has_kore {
        let conn = rusqlite::Connection::open(current_db_path).map_err(|e| format!("DB Open Error: {}", e))?;
        
        conn.execute(
            &format!("ATTACH DATABASE '{}' AS backup_db", temp_kore_path.display().to_string().replace("'", "''")),
            []
        ).map_err(|e| format!("Attach Error: {}", e))?;
        
        let changed = conn.execute(
            "INSERT OR IGNORE INTO vault_items (id, data, nonce, created_at, updated_at, item_salt) 
             SELECT id, data, nonce, created_at, updated_at, item_salt FROM backup_db.vault_items",
            []
        ).map_err(|e| format!("Merge Error: {}", e))?;
        
        imported_items += changed;
        
        let total_in_backup: i32 = conn.query_row("SELECT count(*) FROM backup_db.vault_items", [], |row| row.get(0)).unwrap_or(0);
        skipped_items = total_in_backup as usize - changed;
        
        conn.execute("DETACH DATABASE backup_db", []).ok();
        std::fs::remove_file(&temp_kore_path).ok();
    }
    
    Ok(format!("Successfully imported {} items (skipped {} exact duplicates).", imported_items, skipped_items))
}

#[tauri::command]
pub async fn export_file(
    file_path: String,
    destination: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let hierarchy = verify_auth_and_refresh(&state)?;

    let src_path = PathBuf::from(&file_path);
    let files_dir = state.app_dir.join("files");

    // ── Cloud File Recovery ──────────────────────────────────
    // If the .kaps file doesn't exist locally (post-reinstall),
    // attempt to download it from Supabase Storage automatically.
    if !src_path.exists() {
        let file_name = src_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file path".to_string())?
            .to_string();

        if is_cloud_sync_enabled(&state) {
            let vault_id = state.current_vault_id.lock().unwrap().clone();
            let user_id = state.cloud_user_id.lock().unwrap().clone();
            let jwt = state.cloud_session_token.lock().unwrap().clone();

            if let (Some(vid), Some(uid)) = (vault_id, user_id) {
                let sync = SyncService::new(jwt)
                    .map_err(|e| format!("Cloud recovery init failed: {}", e))?;

                let bytes = sync.download_file(&uid, &vid, &file_name).await
                    .map_err(|e| format!("Cloud file download failed: {}", e))?;

                // Ensure the files directory exists
                if !files_dir.exists() {
                    std::fs::create_dir_all(&files_dir).map_err(|e| e.to_string())?;
                }

                // Write the recovered .kaps to the local disk
                let recovered_path = files_dir.join(&file_name);
                std::fs::write(&recovered_path, &bytes)
                    .map_err(|e| format!("Failed to save recovered file: {}", e))?;

                // Now decrypt the recovered file
                let decrypted_data = FileService::load_attachment(&recovered_path, &hierarchy)
                    .map_err(|e| format!("Decryption failed for recovered {:?}: {:?}", recovered_path, e))?;

                std::fs::write(&destination, decrypted_data)
                    .map_err(|e| format!("Failed to save file: {}", e))?;

                return Ok(());
            }
        }

        return Err(format!("File not found locally and cloud recovery unavailable: {:?}", src_path));
    }

    // Standard local decryption path
    let decrypted_data = FileService::load_attachment(&src_path, &hierarchy)
        .map_err(|e| format!("Decryption failed for {:?}: {:?}", src_path, e))?;

    std::fs::write(&destination, decrypted_data)
        .map_err(|e| format!("Failed to save file: {}", e))?;

    Ok(())
}

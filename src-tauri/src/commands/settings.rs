use tauri::State;
use crate::state::AppState;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_lock_minutes: i32,
    pub clipboard_clear_seconds: i32,
    pub auto_backup_frequency: String,
    pub auto_backup_dir: String,
    pub cloud_sync_enabled: bool,
    pub screenshot_protection: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_lock_minutes: 5,
            clipboard_clear_seconds: 30,
            auto_backup_frequency: "None".to_string(),
            auto_backup_dir: "".to_string(),
            cloud_sync_enabled: true,
            screenshot_protection: true, // ON by default
        }
    }
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let mut settings = AppSettings::default();

    if let Ok(Some(val)) = state.db.get_metadata("setting_auto_lock_mins") {
        if let Ok(parsed) = val.parse::<i32>() {
            settings.auto_lock_minutes = parsed;
        }
    }

    if let Ok(Some(val)) = state.db.get_metadata("setting_clipboard_clear_secs") {
        if let Ok(parsed) = val.parse::<i32>() {
            settings.clipboard_clear_seconds = parsed;
        }
    }

    if let Ok(Some(val)) = state.db.get_metadata("setting_auto_backup_frequency") {
        settings.auto_backup_frequency = val;
    }

    if let Ok(Some(val)) = state.db.get_metadata("setting_auto_backup_dir") {
        if !val.is_empty() {
            settings.auto_backup_dir = val;
        } else {
            // Default to <app_dir>/backups
            let default_backup_dir = state.app_dir.join("backups");
            settings.auto_backup_dir = default_backup_dir.to_string_lossy().to_string();
        }
    } else {
        // Default to <app_dir>/backups if never set
        let default_backup_dir = state.app_dir.join("backups");
        settings.auto_backup_dir = default_backup_dir.to_string_lossy().to_string();
    }

    if let Ok(Some(val)) = state.db.get_metadata("setting_cloud_sync_enabled") {
        if let Ok(parsed) = val.parse::<bool>() {
            settings.cloud_sync_enabled = parsed;
        }
    }

    if let Ok(Some(val)) = state.db.get_metadata("setting_screenshot_protection") {
        if let Ok(parsed) = val.parse::<bool>() {
            settings.screenshot_protection = parsed;
        }
    }

    Ok(settings)
}

#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    state.db.save_metadata("setting_auto_lock_mins", &settings.auto_lock_minutes.to_string())
        .map_err(|e| format!("Database error: {:?}", e))?;
        
    state.db.save_metadata("setting_clipboard_clear_secs", &settings.clipboard_clear_seconds.to_string())
        .map_err(|e| format!("Database error: {:?}", e))?;
        
    state.db.save_metadata("setting_auto_backup_frequency", &settings.auto_backup_frequency)
        .map_err(|e| format!("Database error: {:?}", e))?;
        
    state.db.save_metadata("setting_auto_backup_dir", &settings.auto_backup_dir)
        .map_err(|e| format!("Database error: {:?}", e))?;

    state.db.save_metadata("setting_cloud_sync_enabled", &settings.cloud_sync_enabled.to_string())
        .map_err(|e| format!("Database error: {:?}", e))?;

    state.db.save_metadata("setting_screenshot_protection", &settings.screenshot_protection.to_string())
        .map_err(|e| format!("Database error: {:?}", e))?;

    if settings.screenshot_protection {
        crate::security::anti_forensic::enable_screenshot_protection(&window).ok();
    } else {
        crate::security::anti_forensic::disable_screenshot_protection(&window).ok();
    }

    Ok(())
}

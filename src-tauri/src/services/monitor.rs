use tauri::{AppHandle, Manager, Emitter};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use crate::state::AppState;

const MONITOR_INTERVAL: Duration = Duration::from_secs(1);
const SLEEP_THRESHOLD: Duration = Duration::from_secs(5); // If (now - last_tick) > 5s, assume we slept/suspended.

pub fn start_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let mut last_tick = Instant::now();

        loop {
            sleep(MONITOR_INTERVAL).await;
            
            let now = Instant::now();
            let elapsed_since_tick = now.duration_since(last_tick);

            // 1. Check for System Sleep / Time Jump
            if elapsed_since_tick > SLEEP_THRESHOLD {
                // System likely slept or was suspended
                // Lock the vault immediately
                if let Ok(mut key_guard) = state.key_hierarchy.lock() {
                    if key_guard.is_some() {
                        println!("[Auto-Lock] System sleep detected (waited {:?}). Locking vault.", elapsed_since_tick);
                        *key_guard = None;
                        // Log event (best effort, ignore error if db busy or locking fails)
                        state.db.log_auth_event("AUTO_LOCK", "SYSTEM_SLEEP").ok();
                        
                        // Notify frontend to show lock screen
                        app.emit("vault-locked", ()).ok(); 
                    }
                }
            }

            // 2. Check for Inactivity Timeout
            // Read configured timeout from DB directly
            let configured_mins = match state.db.get_metadata("setting_auto_lock_mins") {
                Ok(Some(val)) => val.parse::<i32>().unwrap_or(5),
                _ => 5, // Default 5 mins
            };
            
            // If <= 0, auto-lock is disabled
            if configured_mins > 0 {
                let timeout_duration = Duration::from_secs((configured_mins as u64) * 60);
                
                if let Ok(activity_guard) = state.last_activity.lock() {
                    if activity_guard.elapsed() > timeout_duration {
                        // Check if already locked to avoid spamming logs/events
                        if let Ok(mut key_guard) = state.key_hierarchy.lock() {
                            if key_guard.is_some() {
                                println!("[Auto-Lock] Inactivity detected (configured {:#?} mins). Locking vault.", configured_mins);
                                *key_guard = None;
                                state.db.log_auth_event("AUTO_LOCK", "INACTIVITY_TIMEOUT").ok();
                                app.emit("vault-locked", ()).ok();
                            }
                        }
                    }
                }
            }

            // 3. Check for Automatic Backups (Every ~60s to avoid DB spam)
            // Just use elapsed_since_tick logic accumulation? No, just a simple static check.
            // Actually, we can just use a static counter or simply check the time since a local Instant
            // Let's add `last_backup_check` at the top of the thread.
            
            // To be elegant within the existing loop without large refactoring:
            // Since loop is 1 sec, we can just do: `if now.duration_since(last_tick).as_secs() == ...` wait, now.duration_since(last_tick) is roughly 1s.
            // I'll add a static counter inside block:
            // Wait, Rust doesn't have local statics like C. I'll just check if `now.timestamp() % 60 == 0`.
            if chrono::Utc::now().timestamp() % 60 == 0 {
                if let Ok(Some(freq)) = state.db.get_metadata("setting_auto_backup_frequency") {
                    if freq != "None" {
                        if let Ok(Some(dir)) = state.db.get_metadata("setting_auto_backup_dir") {
                            if !dir.is_empty() {
                                let now_ts = chrono::Utc::now().timestamp();
                                let last_backup_ts: i64 = state.db.get_metadata("setting_last_backup_time")
                                    .ok().flatten().and_then(|t| t.parse().ok()).unwrap_or(0);
                                
                                let interval_secs: i64 = match freq.as_str() {
                                    "Daily" => 86400,
                                    "Weekly" => 86400 * 7,
                                    "Monthly" => 86400 * 30,
                                    _ => 0,
                                };
                                
                                if interval_secs > 0 && (now_ts - last_backup_ts) >= interval_secs {
                                    // Backup Due! Requires unlocked vault.
                                    if let Ok(key_guard) = state.key_hierarchy.lock() {
                                        if let Some(hierarchy) = &*key_guard {
                                            let dt = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                                            let mut backup_path = std::path::PathBuf::from(dir);
                                            backup_path.push(format!("keptr_backup_{}.kept", dt));
                                            
                                            let vault_path_str = state.db.get_current_path();
                                            let vault_path = std::path::PathBuf::from(vault_path_str);
                                            let files_dir = state.app_dir.join("files");
                                            
                                            if crate::services::file_service::FileService::create_backup(
                                                &vault_path,
                                                &files_dir,
                                                &backup_path,
                                                hierarchy
                                            ).is_ok() {
                                                state.db.save_metadata("setting_last_backup_time", &now_ts.to_string()).ok();
                                                println!("[Auto-Backup] Backup created successfully at {:?}", backup_path);
                                            } else {
                                                println!("[Auto-Backup] Failed to create backup!");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Sleep an extra second to avoid triggering multiple times in the same second
                sleep(Duration::from_secs(1)).await;
            }

            last_tick = Instant::now();
        }
    }); 
}

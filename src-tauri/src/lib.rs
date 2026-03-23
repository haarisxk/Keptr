pub mod security;
pub mod storage;
pub mod services;
pub mod state;
pub mod commands;
pub mod models;

use tauri::{Builder, Manager, Emitter, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{Code, Modifiers};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // Initialize AppState in %AppData%/Keptr
            let app_data_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let app_data_dir = app_data_dir.parent().expect("failed to get parent dir").join("Keptr");
            
            if !app_data_dir.exists() {
                std::fs::create_dir_all(&app_data_dir).expect("failed to create app data dir");
            }
            
            std::fs::create_dir_all(app_data_dir.join("data")).unwrap_or_default();
            std::fs::create_dir_all(app_data_dir.join("files")).unwrap_or_default();
            std::fs::create_dir_all(app_data_dir.join("backups")).unwrap_or_default();

            crate::security::pepper::init_pepper(&app_data_dir);

            app.manage(state::AppState::new(&app_data_dir.to_string_lossy()));
            
            // Start Security Monitor (Auto-Lock)
            services::monitor::start_monitor(app.handle().clone());

            // Start Background Token Refresh Daemon
            start_token_refresh_daemon(app.handle().clone());
            
            // Apply Screenshot Protection globally on boot
            if let Some(window) = app.get_webview_window("main") {
                crate::security::anti_forensic::enable_screenshot_protection(&window).ok();
            }
            crate::security::anti_forensic::prevent_crash_dumps();

            // Register Global Autotype Hotkey
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Shortcut, GlobalShortcutExt, ShortcutState};
                let ctrl_shift_a = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyA);
                let super_shift_a = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyA);
                
                let app_handle_clone1 = app.handle().clone();
                let _ = app.handle().global_shortcut().on_shortcut(ctrl_shift_a, move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        show_spotlight(&app_handle_clone1);
                    }
                });
                
                let app_handle_clone2 = app.handle().clone();
                let _ = app.handle().global_shortcut().on_shortcut(super_shift_a, move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        show_spotlight(&app_handle_clone2);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::unlock_vault,
            commands::auth::lock_vault,
            commands::auth::is_unlocked,
            commands::auth::vault_exists,
            commands::auth::setup_vault,
            commands::auth::change_password,
            commands::auth::select_vault,
            commands::auth::logout,
            commands::auth::set_current_user,
            commands::auth::get_auth_state,
            commands::cloud_auth::cloud_signup,
            commands::cloud_auth::cloud_signin,
            commands::cloud_auth::cloud_logout,
            commands::cloud_auth::get_cloud_auth_state,
            commands::cloud_auth::cloud_oauth_signin,
            commands::cloud_auth::delete_account,
            commands::vault::create_vault_item,
            commands::vault::get_vault_items,
            commands::vault::update_vault_item,
            commands::vault::delete_vault_item,
            commands::vault::pull_sync_items,
            commands::vault::list_vaults,
            commands::vault::create_vault,
            commands::vault::delete_vault,
            commands::files::import_file,
            commands::files::export_file,
            commands::files::create_full_backup,
            commands::files::import_backup,
            commands::security_key::register_hardware_key,
            commands::security_key::login_with_hardware_key,
            commands::security_key::has_hardware_key,
            commands::backup::create_backup_shares,
            commands::backup::recover_vault,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::share::share_item_e2e,
            commands::share::fetch_inbox,
            commands::share::accept_shared_item,
            commands::share::delete_shared_item,
            commands::share::verify_recipient_email,
            commands::autotype::perform_autotype,
            commands::cloud_auth::refresh_cloud_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_spotlight(handle: &tauri::AppHandle) {
    // If the spotlight window already exists, just show + focus it
    if let Some(window) = handle.get_webview_window("spotlight") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = handle.emit("spotlight-focus", ());
        return;
    }

    // Create a new spotlight window on-demand
    let url = WebviewUrl::App("spotlight.html".into());
    let builder = WebviewWindowBuilder::new(handle, "spotlight", url)
        .title("Keptr Quick Access")
        .inner_size(700.0, 420.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .center()
        .skip_taskbar(true)
        .visible(true)
        .focused(true);

    match builder.build() {
        Ok(_) => {},
        Err(e) => eprintln!("Failed to create spotlight window: {:?}", e),
    }
}

/// Background daemon that silently refreshes the Supabase JWT every 50 minutes.
/// This ensures cloud sync, file backup, and sharing operations never fail
/// due to expired tokens. The daemon only runs when a refresh token is available.
fn start_token_refresh_daemon(app_handle: tauri::AppHandle) {
    use tokio::time::{sleep, Duration};

    const REFRESH_INTERVAL_SECS: u64 = 50 * 60; // 50 minutes (JWT expires at 60)

    tauri::async_runtime::spawn(async move {
        // Initial delay — let the app fully boot and any stored session load
        sleep(Duration::from_secs(10)).await;

        loop {
            {
                let state = app_handle.state::<state::AppState>();
                let has_refresh = state.cloud_refresh_token.lock().unwrap().is_some();

                if has_refresh {
                    // Perform the refresh using the same logic as the Tauri command
                    let refresh_token = state.cloud_refresh_token.lock().unwrap().clone();
                    if let Some(rt) = refresh_token {
                        match services::cloud_auth::CloudAuthService::new() {
                            Ok(service) => {
                                match service.refresh_session(&rt).await {
                                    Ok(resp) => {
                                        *state.cloud_session_token.lock().unwrap() = Some(resp.access_token.clone());
                                        *state.cloud_refresh_token.lock().unwrap() = Some(resp.refresh_token.clone());

                                        // Persist to identity.json
                                        let identity_path = state.app_dir.join("identity.json");
                                        if let Ok(content) = std::fs::read_to_string(&identity_path) {
                                            if let Ok(mut ident) = serde_json::from_str::<serde_json::Value>(&content) {
                                                ident["session_token"] = serde_json::json!(resp.access_token);
                                                ident["refresh_token"] = serde_json::json!(resp.refresh_token);
                                                if let Ok(json) = serde_json::to_string(&ident) {
                                                    let _ = std::fs::write(&identity_path, json);
                                                }
                                            }
                                        }
                                        println!("[Token Refresh] Session refreshed successfully.");
                                    }
                                    Err(e) => {
                                        eprintln!("[Token Refresh] Failed to refresh session: {}", e);
                                        // Don't clear tokens — let the user re-authenticate manually
                                    }
                                }
                            }
                            Err(e) => eprintln!("[Token Refresh] Service init failed: {}", e),
                        }
                    }
                }
            }

            sleep(Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
        }
    });
}

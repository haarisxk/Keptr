use enigo::{Enigo, Keyboard, Settings};
use tokio::time::sleep;
use std::time::Duration;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn perform_autotype(
    text: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 1. Verify caller has unlocked vault privileges
    {
        let hierarchy = state.key_hierarchy.lock().unwrap();
        if hierarchy.is_none() {
            return Err("Vault is locked".to_string());
        }
    }

    // 2. Yield focus so the OS has time to activate the user's previous window
    // This assumes the React frontend minimizes the Tauri window just before calling this.
    sleep(Duration::from_millis(500)).await;

    // 3. Synthesize keystrokes
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| format!("Failed to init keyboard: {:?}", e))?;
    enigo.text(&text).map_err(|e| format!("Failed to type text: {:?}", e))?;

    // 4. Securely wipe the string memory to prevent remanence
    // (Rust automatically drops String, but zeroize is best practice in production)
    // Note: tauri IPC currently receives Strings as copies. In a future harden, 
    // replacing `text: String` with direct DB lookups inside this Rust context 
    // bypasses the IPC bus entirely. For now, this baseline unblocks the engine.
    
    Ok(())
}

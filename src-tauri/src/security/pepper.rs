use std::fs;
use std::path::Path;
use std::sync::OnceLock;

/// The application pepper, loaded at runtime.
/// This ensures no secret is hardcoded in the binary.
static PEPPER: OnceLock<Vec<u8>> = OnceLock::new();

/// Initializes the pepper with the correct application directory.
/// Must be called during Tauri setup.
pub fn init_pepper(app_dir: &Path) {
    let pepper_path = app_dir.join("pepper.key");
    
    // Migration: If pepper.key doesn't exist in app_dir, but exists in current working directory, copy it.
    let legacy_path = Path::new("pepper.key");
    if !pepper_path.exists() && legacy_path.exists() {
        let _ = fs::copy(legacy_path, &pepper_path);
    }
    
    let key = if pepper_path.exists() {
        // Load existing
        fs::read(&pepper_path).expect("Failed to read pepper.key")
    } else {
        // Generate new High-Entropy 256-bit pepper
        let mut key = [0u8; 32];
        getrandom::getrandom(&mut key).expect("RNG failure");
        fs::write(&pepper_path, &key).expect("Failed to write pepper.key");
        key.to_vec()
    };
    
    // Ignore error if already initialized
    let _ = PEPPER.set(key);
}

/// Returns the application pepper.
/// Panics if pepper cannot be loaded (should be initialized at startup).
pub fn get_pepper() -> &'static [u8] {
    PEPPER.get().expect("Pepper not initialized! Call init_pepper first.")
}

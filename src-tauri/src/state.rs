use std::sync::Mutex;
use std::time::Instant;
use std::path::PathBuf;
use crate::security::KeyHierarchy;
use crate::storage::StorageService;

use crate::services::vault_registry::VaultRegistry;

pub struct AppState {
    pub db: StorageService,
    // The key hierarchy (encryption + auth keys) is held in memory when unlocked.
    // Wrapped in Mutex for thread safety across Tauri commands.
    // KeyHierarchy (ZeroizeOnDrop) ensures derived keys are wiped when dropped.
    pub key_hierarchy: Mutex<Option<KeyHierarchy>>,
    pub last_activity: Mutex<Instant>,
    pub app_dir: PathBuf,
    
    // Multi-Vault Support
    pub vault_registry: Mutex<VaultRegistry>,
    pub current_user: Mutex<Option<String>>, // Email of logged-in user
    pub current_vault_id: Mutex<Option<String>>,
    
    // Cloud Authentication
    pub cloud_session_token: Mutex<Option<String>>,
    pub cloud_refresh_token: Mutex<Option<String>>,
    pub cloud_user_id: Mutex<Option<String>>,
}

impl AppState {
    pub fn new(app_dir: &str) -> Self {
        let path = PathBuf::from(app_dir);
        let registry = VaultRegistry::new(&path);
        let db = StorageService::new(app_dir);

        let (session, refresh, uid, email) = crate::commands::cloud_auth::load_identity(&path);
        
        Self {
            db,
            key_hierarchy: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
            app_dir: path,
            vault_registry: Mutex::new(registry),
            current_user: Mutex::new(email),
            current_vault_id: Mutex::new(None),
            cloud_session_token: Mutex::new(session),
            cloud_refresh_token: Mutex::new(refresh),
            cloud_user_id: Mutex::new(uid),
        }
    }
}

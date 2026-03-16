pub mod auth;
pub mod vault;
pub mod files;
pub mod security_key;
pub mod backup;
pub mod settings;
pub mod cloud_auth;
pub mod share;
pub mod autotype;

// Re-export all commands for registration in lib.rs
pub use auth::{vault_exists, setup_vault, unlock_vault, lock_vault, is_unlocked, change_password, select_vault, logout, set_current_user, get_auth_state};
pub use vault::{create_vault_item, get_vault_items, update_vault_item, delete_vault_item, list_vaults, create_vault, delete_vault};
pub use files::{import_file, export_file, create_full_backup, import_backup};
pub use security_key::{register_hardware_key, login_with_hardware_key, has_hardware_key};
pub use backup::{create_backup_shares, recover_vault};
pub use settings::{get_settings, update_settings};
pub use share::{share_item_e2e, fetch_inbox, accept_shared_item, verify_recipient_email};
pub use autotype::{perform_autotype};

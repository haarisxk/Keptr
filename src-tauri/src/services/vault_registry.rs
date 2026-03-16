use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultMetadata {
    pub id: String,         // UUID string
    pub name: String,
    pub path: String,       // Relative path to app_dir/data/
    pub owner: Option<String>, // User email/ID, or None for local
    pub created_at: String,
}

pub struct VaultRegistry {
    app_dir: PathBuf,
    vaults: Vec<VaultMetadata>,
}

impl VaultRegistry {
    pub fn new(app_dir: &Path) -> Self {
        let mut registry = Self {
            app_dir: app_dir.to_path_buf(),
            vaults: Vec::new(),
        };
        registry.load_or_migrate();
        registry
    }

    fn registry_path(&self) -> PathBuf {
        self.app_dir.join("vaults.json")
    }

    fn load_or_migrate(&mut self) {
        let path = self.registry_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(vaults) = serde_json::from_str(&content) {
                    self.vaults = vaults;
                    return;
                }
            }
        }

        // Migration: Check for legacy vault.kore
        let legacy_path = self.app_dir.join("data").join("vault.kore");
        if legacy_path.exists() {
            let metadata = VaultMetadata {
                id: Uuid::new_v4().to_string(),
                name: "Default Vault".to_string(),
                path: "vault.kore".to_string(),
                owner: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            self.vaults.push(metadata);
            self.save().ok(); // Try save
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.vaults)
            .map_err(|e| e.to_string())?;
        fs::write(self.registry_path(), json)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list(&self, owner: Option<&str>) -> Vec<VaultMetadata> {
        self.vaults.iter()
            .filter(|v| v.owner.as_deref() == owner)
            .cloned()
            .collect()
    }
    
    // Returns ALL vaults (for internal maintenance or if we change visible logic)
    pub fn get_all(&self) -> Vec<VaultMetadata> {
        self.vaults.clone()
    }

    pub fn add(&mut self, name: &str, owner: Option<&str>) -> Result<VaultMetadata, String> {
        // Ensure data dir exists
        let data_dir = self.app_dir.join("data");
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        }

        let id = Uuid::new_v4().to_string();
        // Filename: vault_<uuid>.kore
        let filename = format!("vault_{}.kore", id);
        
        let metadata = VaultMetadata {
            id: id.clone(),
            name: name.to_string(),
            path: filename,
            owner: owner.map(|s| s.to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.vaults.push(metadata.clone());
        self.save()?;
        Ok(metadata)
    }

    pub fn get(&self, id: &str) -> Option<VaultMetadata> {
        self.vaults.iter().find(|v| v.id == id).cloned()
    }

    pub fn update_owner(&mut self, id: &str, new_owner: Option<String>) -> Result<(), String> {
        if let Some(vault) = self.vaults.iter_mut().find(|v| v.id == id) {
            vault.owner = new_owner;
            self.save()?;
            Ok(())
        } else {
            Err("Vault not found".to_string())
        }
    }

    pub fn remove(&mut self, id: &str) -> Result<(), String> {
        if let Some(index) = self.vaults.iter().position(|v| v.id == id) {
            let vault = &self.vaults[index];
            let path = self.app_dir.join("data").join(&vault.path);
            
            // Delete file
            if path.exists() {
                fs::remove_file(&path).map_err(|e| e.to_string())?;
                // Attempt to remove wal/shm
                 let _ = fs::remove_file(format!("{}-wal", path.display()));
                 let _ = fs::remove_file(format!("{}-shm", path.display()));
            }

            self.vaults.remove(index);
            self.save()?;
            Ok(())
        } else {
            Err("Vault not found".to_string())
        }
    }
}

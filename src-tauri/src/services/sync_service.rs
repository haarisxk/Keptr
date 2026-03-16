use postgrest::Postgrest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub vault_id: String,
    pub item_id: String,
    pub encrypted_payload: String,
    pub nonce: String,
    pub last_modified: i64,
    pub user_id: String,
}

/// The name of the private Supabase Storage bucket for encrypted file blobs.
const STORAGE_BUCKET: &str = "encrypted-files";

pub struct SyncService {
    client: Postgrest,
    /// Base Supabase project URL (e.g. `https://xxxx.supabase.co`)
    storage_base_url: String,
    /// The `anon` or `service_role` key used in the `apikey` header.
    api_key: String,
    /// The user's JWT bearer token for authenticated requests.
    auth_token: String,
}

impl SyncService {
    pub fn new(jwt: Option<String>) -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let mut supabase_url = env::var("SUPABASE_URL").unwrap_or_default();
        let mut supabase_key = env::var("SUPABASE_ANON_KEY").unwrap_or_default();
        
        if supabase_url.is_empty() {
             supabase_url = option_env!("SUPABASE_URL").unwrap_or("").to_string();
        }
        if supabase_key.is_empty() {
             supabase_key = option_env!("SUPABASE_ANON_KEY").unwrap_or("").to_string();
        }

        if supabase_url.is_empty() || supabase_key.is_empty() {
             println!("[SyncService Warning]: Supabase URL or Key not found in .env or build context.");
        }

        let auth_token = jwt.unwrap_or_else(|| supabase_key.clone());

        let rest_url = format!("{}/rest/v1", supabase_url.trim_end_matches('/'));
        let client = Postgrest::new(rest_url)
            .insert_header("apikey", &supabase_key)
            .insert_header("Authorization", format!("Bearer {}", auth_token));

        Ok(Self {
            client,
            storage_base_url: supabase_url.trim_end_matches('/').to_string(),
            api_key: supabase_key,
            auth_token,
        })
    }

    // ─────────────────────────────────────────────────────────────
    //  Supabase Storage API — Encrypted File Cloud Backup
    // ─────────────────────────────────────────────────────────────

    /// Builds the canonical storage object path for a file.
    /// Format: `{user_id}/{vault_id}/{file_name}`
    fn storage_object_path(user_id: &str, vault_id: &str, file_name: &str) -> String {
        format!("{}/{}/{}", user_id, vault_id, file_name)
    }

    /// Uploads an already-encrypted `.kaps` file blob to Supabase Storage.
    /// Uses upsert semantics (overwrites if the object already exists).
    pub async fn upload_file(
        &self,
        user_id: &str,
        vault_id: &str,
        file_name: &str,
        encrypted_bytes: Vec<u8>,
    ) -> Result<(), String> {
        let object_path = Self::storage_object_path(user_id, vault_id, file_name);
        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.storage_base_url, STORAGE_BUCKET, object_path
        );

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/octet-stream")
            // x-upsert: true allows overwriting existing objects
            .header("x-upsert", "true")
            .body(encrypted_bytes)
            .send()
            .await
            .map_err(|e| format!("Storage upload network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Storage upload failed ({}): {}", status, err_text));
        }

        Ok(())
    }

    /// Downloads an encrypted `.kaps` file blob from Supabase Storage.
    /// Returns the raw bytes of the encrypted file.
    pub async fn download_file(
        &self,
        user_id: &str,
        vault_id: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, String> {
        let object_path = Self::storage_object_path(user_id, vault_id, file_name);
        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.storage_base_url, STORAGE_BUCKET, object_path
        );

        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await
            .map_err(|e| format!("Storage download network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Storage download failed ({}): {}", status, err_text));
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("Failed to read storage response body: {}", e))
    }

    /// Deletes an encrypted `.kaps` file blob from Supabase Storage.
    pub async fn delete_file(
        &self,
        user_id: &str,
        vault_id: &str,
        file_name: &str,
    ) -> Result<(), String> {
        let object_path = Self::storage_object_path(user_id, vault_id, file_name);
        let url = format!(
            "{}/storage/v1/object/{}/{}",
            self.storage_base_url, STORAGE_BUCKET, object_path
        );

        let client = reqwest::Client::new();
        let resp = client
            .delete(&url)
            .header("apikey", &self.api_key)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await
            .map_err(|e| format!("Storage delete network error: {}", e))?;

        // Supabase returns 200 even for non-existent objects, which is fine
        if !resp.status().is_success() {
            let status = resp.status();
            let err_text = resp.text().await.unwrap_or_default();
            eprintln!("[Cloud Storage] Delete warning ({}): {}", status, err_text);
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────────
    //  Postgrest API — Vault Item Sync (existing)
    // ─────────────────────────────────────────────────────────────

    /// Pushes an encrypted SQLite record to the Cloud Locker.
    pub async fn push_item(&self, payload: SyncPayload) -> Result<(), String> {
        let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

        let resp = self.client.from("sync_vaults")
            .upsert(json)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to sync item: {}", err_text));
        }

        Ok(())
    }

    /// Publishes the X25519 Public Key to Supabase for E2EE sharing.
    pub async fn publish_public_key(&self, user_id: &str, public_key: &str) -> Result<(), String> {
        let payload = serde_json::json!({
            "user_id": user_id,
            "public_key": public_key
        });
        
        let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

        let resp = self.client.from("public_keys")
            .on_conflict("user_id")
            .upsert(json)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to publish public key: {}", err_text));
        }

        Ok(())
    }

    /// Deletes a specific item permanently from the Cloud Locker.
    pub async fn delete_item(&self, vault_id: &str, item_id: &str, user_id: &str) -> Result<(), String> {
        let resp = self.client.from("sync_vaults")
            .eq("vault_id", vault_id)
            .eq("item_id", item_id)
            .eq("user_id", user_id)
            .delete()
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to delete item from cloud: {}", err_text));
        }

        Ok(())
    }

    /// Pulls all updated blocks for a specific vault from the Cloud Locker since the last sync time.
    pub async fn pull_updates(&self, vault_id: &str, since_timestamp: i64) -> Result<Vec<SyncPayload>, String> {
        let resp = self.client.from("sync_vaults")
            .eq("vault_id", vault_id)
            .gte("last_modified", since_timestamp.to_string())
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
             let err_text = resp.text().await.unwrap_or_default();
             return Err(format!("Failed to fetch updates: {}", err_text));
        }

        let text = resp.text().await.map_err(|e| e.to_string())?;
        let updates: Vec<SyncPayload> = serde_json::from_str(&text).map_err(|e| e.to_string())?;

        Ok(updates)
    }

    /// Wipes all cloud-synced blocks for the specified user from Supabase and deletes the user account itself.
    pub async fn delete_all_user_data(&self, user_id: &str) -> Result<(), String> {
        let resp = self.client.from("sync_vaults")
            .eq("user_id", user_id)
            .delete()
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to delete user data from cloud: {}", err_text));
        }

        let _ = self.client.rpc("delete_user", "{}")
            .execute()
            .await;

        Ok(())
    }
}

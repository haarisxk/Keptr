use postgrest::Postgrest;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct SharedPayload {
    pub id: Option<String>,
    pub sender_id: String,
    pub recipient_id: String,
    pub vault_id: String,
    pub item_id: String,
    pub encrypted_payload: String,
    pub nonce: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKeyLookup {
    pub user_id: String,
    pub public_key: String,
}

pub struct E2EService {
    client: Postgrest,
}

impl E2EService {
    pub fn new(jwt: Option<String>) -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let supabase_url = env::var("SUPABASE_URL").unwrap_or_else(|_| option_env!("SUPABASE_URL").unwrap_or("").to_string());
        let supabase_key = env::var("SUPABASE_ANON_KEY").unwrap_or_else(|_| option_env!("SUPABASE_ANON_KEY").unwrap_or("").to_string());

        let auth_token = jwt.unwrap_or_else(|| supabase_key.clone());
        let rest_url = format!("{}/rest/v1", supabase_url.trim_end_matches('/'));
        
        let client = Postgrest::new(rest_url)
            .insert_header("apikey", &supabase_key)
            .insert_header("Authorization", format!("Bearer {}", auth_token));

        Ok(Self { client })
    }

    /// Looks up a recipient's user_id and X25519 public key by their email address.
    /// This relies on a custom RPC function `get_public_key_by_email` deployed to Supabase.
    pub async fn fetch_public_key(&self, email: &str) -> Result<PublicKeyLookup, String> {
        let payload = serde_json::json!({ "target_email": email });
        let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

        let resp = self.client.rpc("get_public_key_by_email", &json)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to locate recipient (Make sure they are registered and have logged in at least once): {}", err_text));
        }

        let text = resp.text().await.map_err(|e| e.to_string())?;
        
        let mut lookups: Vec<PublicKeyLookup> = if let Ok(arr) = serde_json::from_str(&text) {
            arr
        } else if let Ok(single) = serde_json::from_str::<PublicKeyLookup>(&text) {
            vec![single]
        } else {
            return Err("Recipient not found or has not generated a public key yet.".to_string());
        };

        if lookups.is_empty() {
             return Err("Recipient not found or has not generated a public key yet.".to_string());
        }

        Ok(lookups.remove(0))
    }

    /// Fetches a sender's public key directly from the `public_keys` table using their UUID.
    pub async fn fetch_public_key_by_id(&self, user_id: &str) -> Result<String, String> {
        let resp = self.client.from("public_keys")
            .eq("user_id", user_id)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to retrieve sender's public key: {}", err_text));
        }

        let text = resp.text().await.map_err(|e| e.to_string())?;
        
        let mut lookups: Vec<PublicKeyLookup> = if let Ok(arr) = serde_json::from_str(&text) {
            arr
        } else if let Ok(single) = serde_json::from_str::<PublicKeyLookup>(&text) {
            vec![single]
        } else {
            return Err("Sender's public key is no longer available.".to_string());
        };

        if lookups.is_empty() {
             return Err("Sender's public key is no longer available.".to_string());
        }

        Ok(lookups.remove(0).public_key)
    }

    /// Dispatches an encrypted package to the Supabase Cloud.
    pub async fn send_package(&self, payload: SharedPayload) -> Result<(), String> {
        let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

        let resp = self.client.from("shared_items")
            .insert(json)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Failed to send package: {}", err_text));
        }

        Ok(())
    }

    /// Checks the Supabase inbox for incoming shared items addressed to this user.
    pub async fn check_inbox(&self, user_id: &str) -> Result<Vec<SharedPayload>, String> {
        let resp = self.client.from("shared_items")
            .eq("recipient_id", user_id)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
             let err_text = resp.text().await.unwrap_or_default();
             return Err(format!("Failed to check inbox: {}", err_text));
        }

        let text = resp.text().await.map_err(|e| e.to_string())?;
        let items: Vec<SharedPayload> = serde_json::from_str(&text).map_err(|e| e.to_string())?;

        Ok(items)
    }

    /// Resolves a user's UUID to their email address using the `get_email_by_user_id` RPC.
    pub async fn resolve_sender_email(&self, user_id: &str) -> Result<String, String> {
        let payload = serde_json::json!({ "target_user_id": user_id });
        let json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;

        let resp = self.client.rpc("get_email_by_user_id", &json)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Ok(user_id.to_string()); // Fallback to raw UUID if resolution fails
        }

        let text = resp.text().await.map_err(|e| e.to_string())?;
        
        // The RPC returns a plain JSON string, e.g. `"sender@email.com"`
        if let Ok(email) = serde_json::from_str::<String>(&text) {
            if !email.is_empty() {
                return Ok(email);
            }
        }
        
        Ok(user_id.to_string())
    }

    /// Permanently deletes an incoming shared package after it has been securely downloaded.
    pub async fn delete_package(&self, package_id: &str) -> Result<(), String> {
        let resp = self.client.from("shared_items")
            .eq("id", package_id)
            .delete()
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
             let err_text = resp.text().await.unwrap_or_default();
             return Err(format!("Failed to delete synchronized package: {}", err_text));
        }

        Ok(())
    }
}

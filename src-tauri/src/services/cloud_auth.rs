use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct AuthRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub user: UserDto,
}

#[derive(Deserialize, Debug)]
pub struct UserDto {
    pub id: String,
    pub aud: String,
    pub email: String,
}

pub struct CloudAuthService {
    client: Client,
    url: String,
    api_key: String,
}

impl CloudAuthService {
    pub fn new() -> Result<Self, String> {
        dotenvy::dotenv().ok();
        let mut url = env::var("SUPABASE_URL").unwrap_or_default();
        let mut key = env::var("SUPABASE_ANON_KEY").unwrap_or_default();

        if url.is_empty() { url = option_env!("SUPABASE_URL").unwrap_or("").to_string(); }
        if key.is_empty() { key = option_env!("SUPABASE_ANON_KEY").unwrap_or("").to_string(); }

        if url.is_empty() || key.is_empty() {
             return Err("Missing Supabase configuration".into());
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            client,
            url,
            api_key: key,
        })
    }

    pub async fn sign_up(&self, email: &str, password: &str) -> Result<AuthResponse, String> {
        let endpoint = format!("{}/auth/v1/signup", self.url);
        let resp = self.client.post(&endpoint)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&AuthRequest { email, password })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
             let err_text = resp.text().await.unwrap_or_default();
             return Err(format!("Sign up failed: {}", err_text));
        }

        resp.json::<AuthResponse>().await.map_err(|e| e.to_string())
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthResponse, String> {
        let endpoint = format!("{}/auth/v1/token?grant_type=password", self.url);
        let resp = self.client.post(&endpoint)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&AuthRequest { email, password })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
             let err_text = resp.text().await.unwrap_or_default();
             return Err(format!("Sign in failed: {}", err_text));
        }

        resp.json::<AuthResponse>().await.map_err(|e| e.to_string())
    }

    /// Refreshes an expired session using a long-lived refresh token.
    /// Returns a fresh `AuthResponse` with a new `access_token` and rotated `refresh_token`.
    pub async fn refresh_session(&self, refresh_token: &str) -> Result<AuthResponse, String> {
        let endpoint = format!("{}/auth/v1/token?grant_type=refresh_token", self.url);

        let body = serde_json::json!({
            "refresh_token": refresh_token
        });

        let resp = self.client.post(&endpoint)
            .header("apikey", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Token refresh network error: {}", e))?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(format!("Token refresh failed: {}", err_text));
        }

        resp.json::<AuthResponse>().await.map_err(|e| format!("Token refresh parse error: {}", e))
    }
}

use serde::{Deserialize, Serialize};
use tiny_http::{Server, Response, Method, Header};
use tauri::{Window, Manager};
use tauri_plugin_shell::ShellExt;
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct TokenPayload {
    access_token: String,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudAuthResponse {
    pub success: bool,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub error: Option<String>,
}

pub async fn start_supabase_oauth(window: Window, provider: &str) -> Result<CloudAuthResponse, String> {
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
         return Err("Supabase URL or Key not found in .env or build context.".to_string());
    }

    let port = 14222;
    let server = Server::http(format!("127.0.0.1:{}", port)).map_err(|e| e.to_string())?;
    let redirect_uri = format!("http://127.0.0.1:{}/auth", port);

    let auth_url = format!(
        "{}/auth/v1/authorize?provider={}&redirect_to={}",
        supabase_url.trim_end_matches('/'),
        provider,
        redirect_uri
    );

    let shell = window.app_handle().shell();
    if let Err(e) = shell.open(&auth_url, None) {
          return Err(format!("Failed to open browser: {}", e));
    }

    let tokens = tauri::async_runtime::spawn_blocking(move || {
        loop {
            if let Ok(mut request) = server.recv() {
                match (request.method(), request.url()) {
                    (&Method::Get, "/auth") => {
                        let html = r#"<!DOCTYPE html><html><head><title>Keptr Auth</title></head><body style="background:#09090b;color:#fafafa;font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;"><div style="text-align:center;"><p id="status">Securing your session, please wait...</p></div><script>const hash=window.location.hash.substring(1);const params=new URLSearchParams(hash);const accessToken=params.get('access_token');if(accessToken){fetch('/token',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({access_token:accessToken,refresh_token:params.get('refresh_token')})}).then(()=>{document.getElementById('status').innerHTML="<h2>Authorization Successful!</h2><p>You can securely close this window and return to Keptr.</p>";setTimeout(()=>window.close(),2000);}).catch(e=>{document.getElementById('status').innerHTML="Error sending token: "+e;});}else{document.getElementById('status').innerHTML="Verification Failed. No access token found.";}</script></body></html>"#;
                        let response = Response::from_string(html).with_header(
                            Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap()
                        );
                        let _ = request.respond(response);
                    },
                    (&Method::Post, "/token") => {
                        let mut content = String::new();
                        let _ = request.as_reader().read_to_string(&mut content);
                        
                        let response = Response::from_string("OK").with_header(
                            Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap()
                        );
                        let _ = request.respond(response);

                        if let Ok(payload) = serde_json::from_str::<TokenPayload>(&content) {
                            return Some((payload.access_token, payload.refresh_token));
                        }
                    },
                    _ => {
                        let response = Response::from_string("Not Found").with_status_code(404);
                        let _ = request.respond(response);
                    }
                }
            }
        }
    }).await.map_err(|e| e.to_string())?.ok_or("Failed to receive tokens")?;

    let (access_token, refresh_token_opt) = tokens;

    // Now use the access token to get user info
    let client = reqwest::Client::new();
    let user_url = format!("{}/auth/v1/user", supabase_url.trim_end_matches('/'));
    
    let user_resp = client.get(&user_url)
        .header("apikey", &supabase_key)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !user_resp.status().is_success() {
        return Err(format!("Failed to retrieve user info: {}", user_resp.status()));
    }

    let user_json: serde_json::Value = user_resp.json().await.map_err(|e| e.to_string())?;
    let email = user_json.get("email").and_then(|s| s.as_str()).map(|s| s.to_string());
    let id = user_json.get("id").and_then(|s| s.as_str()).ok_or("No user ID found")?.to_string();

    Ok(CloudAuthResponse {
        success: true,
        access_token: Some(access_token),
        refresh_token: refresh_token_opt,
        user_id: Some(id),
        email,
        error: None,
    })
}

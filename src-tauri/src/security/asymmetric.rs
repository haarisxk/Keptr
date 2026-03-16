use x25519_dalek::{PublicKey, StaticSecret};
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generates a new X25519 Keypair for Asymmetric End-to-End Encryption.
/// Returns (Private Key Base64, Public Key Base64).
pub fn generate_x25519_keypair() -> (String, String) {
    let secret_key = StaticSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&secret_key);

    let sec_b64 = URL_SAFE_NO_PAD.encode(secret_key.to_bytes());
    let pub_b64 = URL_SAFE_NO_PAD.encode(public_key.as_bytes());

    (sec_b64, pub_b64)
}

/// Computes the Diffie-Hellman shared secret from a local Private Key and remote Public Key.
pub fn compute_shared_secret(private_b64: &str, public_b64: &str) -> Result<[u8; 32], String> {
    let sec_bytes = URL_SAFE_NO_PAD.decode(private_b64).map_err(|e| format!("Invalid private key encoding: {}", e))?;
    let pub_bytes = URL_SAFE_NO_PAD.decode(public_b64).map_err(|e| format!("Invalid public key encoding: {}", e))?;

    if sec_bytes.len() != 32 || pub_bytes.len() != 32 {
        return Err("Invalid key length".to_string());
    }

    let mut sec_arr = [0u8; 32];
    sec_arr.copy_from_slice(&sec_bytes);
    
    let mut pub_arr = [0u8; 32];
    pub_arr.copy_from_slice(&pub_bytes);

    let secret_key = StaticSecret::from(sec_arr);
    let public_key = PublicKey::from(pub_arr);

    let shared_secret = secret_key.diffie_hellman(&public_key);
    
    Ok(*shared_secret.as_bytes())
}

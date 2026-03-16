use x25519_dalek::{PublicKey, StaticSecret};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

fn main() {
    // 1. Let's create a dummy Sender Private Key
    let sender_secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
    let sender_public = PublicKey::from(&sender_secret);
    let sender_pub_b64 = URL_SAFE_NO_PAD.encode(sender_public.as_bytes());

    // 2. Let's create a dummy Recipient Private Key
    let rec_secret = StaticSecret::random_from_rng(rand::rngs::OsRng);
    let rec_public = PublicKey::from(&rec_secret);
    let rec_pub_b64 = URL_SAFE_NO_PAD.encode(rec_public.as_bytes());

    // 3. Let's test the compute fn
    let sender_shared = compute_shared_secret(&URL_SAFE_NO_PAD.encode(sender_secret.to_bytes()), &rec_pub_b64).unwrap();
    let rec_shared = compute_shared_secret(&URL_SAFE_NO_PAD.encode(rec_secret.to_bytes()), &sender_pub_b64).unwrap();

    println!("Sender Shared: {:?}", sender_shared);
    println!("Rec Shared: {:?}", rec_shared);
    println!("Match: {}", sender_shared == rec_shared);
}

pub fn compute_shared_secret(private_b64: &str, public_b64: &str) -> Result<[u8; 32], String> {
    let sec_bytes = URL_SAFE_NO_PAD.decode(private_b64).map_err(|e| format!("Invalid private key encoding: {}", e))?;
    let pub_bytes = URL_SAFE_NO_PAD.decode(public_b64).map_err(|e| format!("Invalid public key encoding: {}", e))?;

    let mut sec_arr = [0u8; 32];
    sec_arr.copy_from_slice(&sec_bytes);
    
    let mut pub_arr = [0u8; 32];
    pub_arr.copy_from_slice(&pub_bytes);

    let secret_key = StaticSecret::from(sec_arr);
    let public_key = PublicKey::from(pub_arr);

    let shared_secret = secret_key.diffie_hellman(&public_key);
    
    Ok(*shared_secret.as_bytes())
}

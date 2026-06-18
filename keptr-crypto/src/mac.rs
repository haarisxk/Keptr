use hmac::{Hmac, Mac};
use sha2::Sha512;
use crate::memory::SecretBytes;

type HmacSha512 = Hmac<Sha512>;

/// Computes HMAC-SHA-512 over the given data.
/// Returns a 64-byte array (512 bits).
pub fn compute_hmac(key: &SecretBytes, data: &[u8]) -> Result<[u8; 64], String> {
    let mut mac = HmacSha512::new_from_slice(key.as_bytes())
        .map_err(|e| format!("HMAC key setup failed: {}", e))?;
    mac.update(data);
    
    let result = mac.finalize();
    let bytes = result.into_bytes();
    
    let mut output = [0u8; 64];
    output.copy_from_slice(&bytes);
    Ok(output)
}

/// Verifies HMAC-SHA-512.
pub fn verify_hmac(key: &SecretBytes, data: &[u8], expected_mac: &[u8]) -> Result<(), String> {
    let mut mac = HmacSha512::new_from_slice(key.as_bytes())
        .map_err(|e| format!("HMAC key setup failed: {}", e))?;
    mac.update(data);
    
    mac.verify_slice(expected_mac)
        .map_err(|_| "HMAC verification failed. Data integrity compromised.".to_string())
}

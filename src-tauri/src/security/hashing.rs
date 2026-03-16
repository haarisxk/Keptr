use sha2::{Sha256, Sha512, Digest};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

/// Computes the SHA-256 hash of the given data (32 bytes).
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes the SHA-512 hash of the given data (64 bytes).
pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes an HMAC-SHA256 tag for the given data using the provided key.
///
/// Returns a 32-byte authentication tag.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC-SHA256 accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// Verifies an HMAC-SHA256 tag against expected data.
///
/// Uses constant-time comparison to prevent timing attacks.
pub fn verify_hmac_sha256(key: &[u8], data: &[u8], tag: &[u8; 32]) -> bool {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC-SHA256 accepts any key length");
    mac.update(data);
    mac.verify_slice(tag).is_ok()
}

/// Computes an HMAC-SHA512 tag for the given data using the provided key.
///
/// Returns a 64-byte authentication tag.
pub fn hmac_sha512(key: &[u8], data: &[u8]) -> [u8; 64] {
    let mut mac = HmacSha512::new_from_slice(key)
        .expect("HMAC-SHA512 accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// Verifies an HMAC-SHA512 tag against expected data.
///
/// Uses constant-time comparison to prevent timing attacks.
pub fn verify_hmac_sha512(key: &[u8], data: &[u8], tag: &[u8; 64]) -> bool {
    let mut mac = HmacSha512::new_from_slice(key)
        .expect("HMAC-SHA512 accepts any key length");
    mac.update(data);
    mac.verify_slice(tag).is_ok()
}

//! Key Derivation Hierarchy
//! 
//! Implements a tiered key system for defense-in-depth:
//! L0: Master Key (Argon2id + Salt + Pepper)
//! L1: Encryption Key (HKDF-SHA512), Auth Key (HKDF-SHA512)
//! L2: Per-Item Keys (HKDF-SHA512)

use hkdf::Hkdf;
use sha2::Sha512;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};
use super::{SecureKey, CryptoError};
use super::argon2::{self, Argon2Params};
use super::secret::SecretString;
use super::pepper;

/// Holds the derived L1 keys necessary for vault operations.
/// 
/// The raw Master Key is zeroized immediately after deriving these keys.
/// `encryption_key` is used to derive per-item keys.
/// `auth_key` is used strictly for authentication proofs (canary).
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct KeyHierarchy {
    pub encryption_key: SecureKey,
    pub auth_key: SecureKey,
}

impl KeyHierarchy {
    /// Use the default high-security parameters for deriving keys.
    pub fn derive(password: &SecretString, salt: &str) -> Result<Self, CryptoError> {
        Self::derive_with_params(password, salt, &Argon2Params::high_security())
    }

    /// Derives the full key hierarchy from the user password.
    /// 
    /// 1. Mixes in application pepper.
    /// 2. Derives L0 Master Key using Argon2id.
    /// 3. Derives L1 Encryption & Auth Keys using HKDF-SHA512.
    pub fn derive_with_params(
        password: &SecretString, 
        salt: &str, 
        params: &Argon2Params
    ) -> Result<Self, CryptoError> {
        let pepper = pepper::get_pepper();
        
        let master_key = argon2::derive_key_with_pepper(password, salt, pepper, params)?;
        Self::derive_from_master_key(&master_key)
    }

    /// Derives L1 keys directly from the L0 Master Key.
    /// Used when the Master Key is unwrapped via Hardware Key or other mechanisms.
    pub fn derive_from_master_key(master_key: &SecureKey) -> Result<Self, CryptoError> {
        // Dereference SecureKey (Zeroizing<[u8; 32]>) to slice for Hkdf
        let hkdf = Hkdf::<Sha512>::new(None, &master_key[..]);

        let mut encryption_key_bytes = [0u8; 32];
        hkdf.expand(b"keptr-encryption-key", &mut encryption_key_bytes)
            .map_err(|_| CryptoError::KeyDerivationFailed("HKDF expansion failed for encryption key".to_string()))?;
        
        let mut auth_key_bytes = [0u8; 32];
        hkdf.expand(b"keptr-auth-key", &mut auth_key_bytes)
            .map_err(|_| CryptoError::KeyDerivationFailed("HKDF expansion failed for auth key".to_string()))?;

        Ok(Self {
            encryption_key: Zeroizing::new(encryption_key_bytes),
            auth_key: Zeroizing::new(auth_key_bytes),
        })
    }
    
    /// Derives an L2 per-item encryption key.
    /// 
    /// Uses HKDF-SHA512(encryption_key, info="keptr-item:{id}", salt=item_salt).
    pub fn derive_item_key(&self, item_id: &uuid::Uuid, item_salt: &[u8]) -> Result<SecureKey, CryptoError> {
        let hkdf = Hkdf::<Sha512>::new(Some(item_salt), &self.encryption_key[..]);
        
        let info = format!("keptr-item:{}", item_id);
        let mut item_key_bytes = [0u8; 32];
        
        hkdf.expand(info.as_bytes(), &mut item_key_bytes)
             .map_err(|_| CryptoError::KeyDerivationFailed("HKDF expansion failed for item key".to_string()))?;
             
        Ok(Zeroizing::new(item_key_bytes))
    }

    /// Derives an L2 per-file encryption key.
    pub fn derive_file_key(&self, file_id: &str, file_salt: &[u8]) -> Result<SecureKey, CryptoError> {
        let hkdf = Hkdf::<Sha512>::new(Some(file_salt), &self.encryption_key[..]);
        
        let info = format!("keptr-file:{}", file_id);
        let mut file_key_bytes = [0u8; 32];
        
        hkdf.expand(info.as_bytes(), &mut file_key_bytes)
             .map_err(|_| CryptoError::KeyDerivationFailed("HKDF expansion failed for file key".to_string()))?;
             
        Ok(Zeroizing::new(file_key_bytes))
    }
    
    /// Derives an L2 backup encryption key.
    pub fn derive_backup_key(&self, backup_id: &str) -> Result<SecureKey, CryptoError> {
        // Backups might not have a salt if single file? 
        // We'll use a fixed salt context or random if provided.
        // For simplicity, let's use the backup_id as info and no salt (HKDF default salt is zeros).
        let hkdf = Hkdf::<Sha512>::new(None, &self.encryption_key[..]);
        
        let info = format!("keptr-backup:{}", backup_id);
        let mut key_bytes = [0u8; 32];
        
        hkdf.expand(info.as_bytes(), &mut key_bytes)
             .map_err(|_| CryptoError::KeyDerivationFailed("HKDF expansion failed for backup key".to_string()))?;
             
        Ok(Zeroizing::new(key_bytes))
    }
}

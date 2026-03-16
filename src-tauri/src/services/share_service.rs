use crate::security::{SecretString, SecureKey};
use crate::security::argon2::{self, Argon2Params};
use crate::security::shamir::{self, ShamirConfig, SecretShare};
use zeroize::Zeroizing;

pub struct ShareService;

impl ShareService {
    /// Generates k-of-n shares for the master key derived from the password.
    /// 
    /// Returns a list of strings, each containing a share in a portable format:
    /// `share-<index>-<threshold>-<hex_data>`
    pub fn generate_shares(
        password: &SecretString,
        salt: &str,
        threshold: u8,
        total_shares: u8,
    ) -> Result<Vec<SecretString>, String> {
        // 1. Derive Master Key
        let pepper = crate::security::pepper::get_pepper();
        let params = Argon2Params::high_security();
        
        let master_key = argon2::derive_key_with_pepper(password, salt, pepper, &params)
            .map_err(|e: crate::security::CryptoError| e.to_string())?;

        // 2. Configure Shamir
        let config = ShamirConfig {
            threshold,
            total_shares,
        };
        config.validate().map_err(|e: crate::security::CryptoError| e.to_string())?;

        // 3. Split Master Key
        let shares = shamir::split_secret(&master_key[..], &config)
            .map_err(|e: crate::security::CryptoError| e.to_string())?;

        // 4. Encode Shares
        let mut encoded_shares = Vec::new();
        for share in shares {
            let hex_data = hex::encode(&share.data);
            // Format: version-index-threshold-data
            // We use v1 prefix
            let s = format!("v1-{}-{}-{}", share.index, share.threshold, hex_data);
            encoded_shares.push(SecretString::new(s));
        }

        Ok(encoded_shares)
    }

    /// Reconstructs the master key from a list of share strings.
    pub fn recover_master_key(share_strings: &[String]) -> Result<SecureKey, String> {
        let mut shares = Vec::new();

        for s in share_strings {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 4 || parts[0] != "v1" {
                return Err("Invalid share format".to_string());
            }

            let index = parts[1].parse::<u8>().map_err(|_| "Invalid share index")?;
            let threshold = parts[2].parse::<u8>().map_err(|_| "Invalid share threshold")?;
            let data = hex::decode(parts[3]).map_err(|_| "Invalid share data")?;

            shares.push(SecretShare::new(index, data, threshold));
        }

        let master_key_bytes = shamir::combine_shares(&shares)
            .map_err(|e: crate::security::CryptoError| e.to_string())?;

        if master_key_bytes.len() != 32 {
            return Err("Recovered key has invalid length".to_string());
        }

        // Convert Vec<u8> to [u8; 32]
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&master_key_bytes);
        
        Ok(Zeroizing::new(key_array))
    }
}

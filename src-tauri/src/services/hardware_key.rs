use serde::{Deserialize, Serialize};
use ctap_hid_fido2::{Cfg, FidoKeyHid};
use ctap_hid_fido2::fidokey::make_credential::{Extension as Mext};
use ctap_hid_fido2::fidokey::get_assertion::{Extension as Gext};
use crate::security::CryptoService;
use std::convert::TryFrom;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HardwareCredential {
    pub id: String,        // Credential ID (Hex)
    pub salt: String,      // Salt used for HDK derivation
}

pub struct HardwareKeyService;

impl HardwareKeyService {
    /// Registers a FIDO2 device using `hmac-secret` extension.
    pub fn register_device() -> Result<HardwareCredential, String> {
        let devices = ctap_hid_fido2::get_fidokey_devices();
        if devices.is_empty() {
            return Err("No supported hardware key detected. Please plug it in and try again.".to_string());
        }
        
        // Use the first device
        let device_info = &devices[0];
        let cfg = Cfg::init();
        let key = FidoKeyHid::new(std::slice::from_ref(&device_info.param), &cfg)
            .map_err(|e| format!("Failed to connect to hardware key: {}", e))?;

        // make_credential arguments
        let challenge = CryptoService::generate_csprng_data(32);
        let rp_id = "keptr.app";
        
        // Request hmac-secret extension
        let extensions = vec![Mext::HmacSecret(Some(true))];

        let att = key.make_credential_with_extensions(
            rp_id,
            &challenge,
            None, // No PIN for now (simplify UI flow, assume User Verification via touch/bio if configured on key)
            Some(&extensions),
        ).map_err(|e| format!("Make Credential failed: {}", e))?;
        
        let cred_id_hex = hex::encode(&att.credential_descriptor.id);
        
        // Generate a random salt (32 bytes) that will be used as the input for HMAC during auth
        let salt = CryptoService::generate_salt();
        
        Ok(HardwareCredential {
            id: cred_id_hex,
            salt,
        })
    }

    /// Authenticates and derives the Hardware Key using `hmac-secret`.
    pub fn authenticate_and_derive(
        credential_id_hex: &str,
        salt_hex: &str,
    ) -> Result<Vec<u8>, String> {
        let devices = ctap_hid_fido2::get_fidokey_devices();
        if devices.is_empty() {
             return Err("No hardware key detected. Please plug it in and try again.".to_string());
        }
        let device_info = &devices[0];
        let cfg = Cfg::init();
        let key = FidoKeyHid::new(std::slice::from_ref(&device_info.param), &cfg)
             .map_err(|e| format!("Failed to connect to hardware key: {}", e))?;
        
        let cid = hex::decode(credential_id_hex).map_err(|_| "Invalid credential ID format")?;
        let salt_bytes = hex::decode(salt_hex).map_err(|_| "Invalid salt format")?;
        let salt_array: [u8; 32] = TryFrom::try_from(salt_bytes.as_slice())
            .map_err(|_| "Invalid salt length")?;

        let rp_id = "keptr.app";
        let challenge = CryptoService::generate_csprng_data(32); // Random challenge for liveness
        
        // Request hmac-secret processing on the salt
        let extensions = vec![Gext::HmacSecret(Some(salt_array))];

        let assertion = key.get_assertion_with_extensios( // Note: correct library typo name
            rp_id,
            &challenge,
            &[cid],
            None, // PIN
            Some(&extensions),
        ).map_err(|e| format!("Failed to authenticate key (GetAssertion): {}", e))?;

        // Extract the HMAC output from extensions
        for ext in assertion.extensions {
            if let Gext::HmacSecret(Some(output)) = ext {
                // This output IS the hardware derived key (32 bytes)
                return Ok(output.to_vec());
            }
        }

        Err("Authenticator did not return hmac-secret output. Ensure you are using the correct FIDO2 key.".to_string())
    }
}

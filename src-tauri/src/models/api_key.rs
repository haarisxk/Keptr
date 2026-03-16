use serde::{Deserialize, Serialize};
use crate::security::secret::SecretString;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ApiKeyData {
    pub service_name: Option<String>,
    pub key_id: Option<String>,
    pub api_secret: Option<SecretString>,
    pub environment: Option<String>, // e.g. "Production", "Staging"
}

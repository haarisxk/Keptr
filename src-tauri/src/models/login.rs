use serde::{Deserialize, Serialize};
use crate::security::secret::SecretString;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LoginData {
    pub username: Option<String>,
    pub password: Option<SecretString>,
    pub url: Option<String>,
    pub totp: Option<SecretString>,
}

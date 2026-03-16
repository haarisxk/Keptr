use serde::{Deserialize, Serialize};
use crate::security::secret::SecretString;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LicenseData {
    pub product_name: Option<String>,
    pub license_key: Option<SecretString>,
    pub purchase_date: Option<String>,
}

use serde::{Deserialize, Serialize};
use crate::security::secret::SecretString;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CardData {
    pub card_holder: Option<String>,
    pub card_number: Option<SecretString>,
    pub expiry_date: Option<String>, // MM/YY
    pub cvv: Option<SecretString>,
    pub billing_address: Option<String>,
}

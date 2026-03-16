use serde::{Deserialize, Serialize};
use crate::security::secret::SecretString;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BankData {
    pub bank_name: Option<String>,
    pub account_number: Option<SecretString>,
    pub routing_number: Option<String>,
    pub swift_code: Option<String>,
    pub iban: Option<String>,
}

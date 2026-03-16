use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{LoginData, CardData, BankData, LicenseData, ApiKeyData, NoteData, FileData};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VaultData {
    Login(LoginData),
    Card(CardData),
    Bank(BankData),
    License(LicenseData),
    ApiKey(ApiKeyData),
    Note(NoteData),
    File(FileData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultItem {
    pub id: Uuid,
    pub title: String,
    
    #[serde(flatten)]
    pub data: VaultData,

    pub created_at: String,
    pub updated_at: String,
    
    #[serde(default)]
    pub favorite: bool,
    pub deleted_at: Option<String>,
}

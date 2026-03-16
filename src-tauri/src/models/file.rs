use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct FileData {
    pub file_path: Option<String>, // Original path or virtual path
    pub file_size: Option<i64>,
    pub file_extension: Option<String>,
}

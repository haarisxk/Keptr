use crate::security::{CryptoService, EncryptedData, KeyHierarchy, CryptoError};
use std::fs::{self, File};
use std::io::{self, Read, Write, Cursor};
use std::path::{Path, PathBuf};
use zip::write::FileOptions;

#[derive(Debug)]
pub enum FileError {
    IoError(io::Error),
    CryptoError(CryptoError),
    ZipError(zip::result::ZipError),
    InvalidExtension(String),
    CorruptedFile(String),
}

impl From<io::Error> for FileError {
    fn from(e: io::Error) -> Self { FileError::IoError(e) }
}
impl From<CryptoError> for FileError {
    fn from(e: CryptoError) -> Self { FileError::CryptoError(e) }
}
impl From<zip::result::ZipError> for FileError {
    fn from(e: zip::result::ZipError) -> Self { FileError::ZipError(e) }
}

pub struct FileService;

impl FileService {
    pub const EXT_DATA: &'static str = "kore";
    pub const EXT_FILE: &'static str = "kaps";
    pub const EXT_BACKUP: &'static str = "kept";
    const SALT_LEN: usize = 32;
    const MAGIC_V2: [u8; 4] = *b"KORE";
    const VERSION_V2: u8 = 2; // Using 2 to denote V2

    /// Encrypts a raw file and saves it as a .kaps file.
    /// Format V2: [MAGIC(4)][VER(1)][SALT(32)][ID_LEN(2, LE)][ID_BYTES][EncryptedData]
    pub fn save_attachment(
        original_path: &Path,
        dest_dir: &Path,
        key_hierarchy: &KeyHierarchy,
    ) -> Result<PathBuf, FileError> {
        let mut file = File::open(original_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let stem = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");

        Self::save_attachment_data(stem, &buffer, dest_dir, key_hierarchy)
    }

    /// Internal helper to encrypt bytes and save to disk
    fn save_attachment_data(
        stem: &str,
        data: &[u8],
        dest_dir: &Path,
        key_hierarchy: &KeyHierarchy,
    ) -> Result<PathBuf, FileError> {
        // 1. Generate Salt
        let file_salt = CryptoService::generate_csprng_data(Self::SALT_LEN);
        
        // Use the stem as the stable Key ID.
        // It is stored in the file, so checking logic verifies this ID regardless of filename.
        let key_id = stem;
        
        // 2. Derive Key using ID
        let file_key = key_hierarchy.derive_file_key(key_id, &file_salt)
            .map_err(FileError::from)?;
            
        // 3. Encrypt (AAD rule: use Key ID as AAD)
        let aad = key_id.as_bytes();
        let encrypted = CryptoService::encrypt_xchacha20_aad(data, &file_key, aad)?;
        
        let encrypted_bytes = serde_json::to_vec(&encrypted)
            .map_err(|e| FileError::IoError(io::Error::new(io::ErrorKind::Other, e)))?;

        let new_filename = format!("{}.{}", stem, Self::EXT_FILE);
        let dest_path = dest_dir.join(new_filename);

        let mut out_file = File::create(&dest_path)?;
        
        // Write V2 Header
        out_file.write_all(&Self::MAGIC_V2)?;         // 4 bytes
        out_file.write_all(&[Self::VERSION_V2])?;     // 1 byte
        out_file.write_all(&file_salt)?;              // 32 bytes
        
        let id_bytes = key_id.as_bytes();
        let id_len = id_bytes.len() as u16;
        out_file.write_all(&id_len.to_le_bytes())?;   // 2 bytes
        out_file.write_all(id_bytes)?;                // Variable bytes
        
        // Write Encrypted Payload
        out_file.write_all(&encrypted_bytes)?;

        Ok(dest_path)
    }

    /// Rotates keys for all encrypted files in the directory.
    pub fn rotate_files(
        files_dir: &Path,
        old_hierarchy: &KeyHierarchy,
        new_hierarchy: &KeyHierarchy,
    ) -> Result<(), FileError> {
        if !files_dir.exists() { return Ok(()); }
        
        for entry in fs::read_dir(files_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some(Self::EXT_FILE) {
                // 1. Load and Decrypt with OLD key
                // Note: load_attachment verifies salt and integrity
                let plaintext = match Self::load_attachment(&path, old_hierarchy) {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Failed to decrypt file during rotation: {:?} - {:?}", path, e);
                        continue;
                    }
                };
                
                let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
                
                // 2. Encrypt and Save with NEW key (overwriting existing file)
                // save_attachment_data will overwrite and UPGRADE to V2 format.
                Self::save_attachment_data(stem, &plaintext, files_dir, new_hierarchy)?;
            }
        }
        Ok(())
    }

    /// Decrypts a .kaps file and returns the raw bytes.
    pub fn load_attachment(kaps_path: &Path, key_hierarchy: &KeyHierarchy) -> Result<Vec<u8>, FileError> {
        if kaps_path.extension().and_then(|s| s.to_str()) != Some(Self::EXT_FILE) {
            return Err(FileError::InvalidExtension(
                "File must have .kaps extension".to_string(),
            ));
        }

        let mut file = File::open(kaps_path)?;
        let mut total_buffer = Vec::new();
        file.read_to_end(&mut total_buffer)?;
        
        if total_buffer.len() < 4 {
             return Err(FileError::CorruptedFile("File too short".to_string()));
        }

        // Check for V2 Magic
        if total_buffer.starts_with(&Self::MAGIC_V2) {
             Self::load_attachment_v2(kaps_path, &total_buffer, key_hierarchy)
        } else {
             Self::load_attachment_legacy(kaps_path, &total_buffer, key_hierarchy)
        }
    }
    
    fn load_attachment_v2(_path: &Path, data: &[u8], key_hierarchy: &KeyHierarchy) -> Result<Vec<u8>, FileError> {
        let mut cursor = Cursor::new(data);
        
        // Skip Magic (4) + Version (1)
        cursor.set_position(5);
        
        // Read Salt (32)
        let mut salt = [0u8; 32];
        cursor.read_exact(&mut salt).map_err(FileError::IoError)?;
        
        // Read ID Len (2)
        let mut id_len_bytes = [0u8; 2];
        cursor.read_exact(&mut id_len_bytes).map_err(FileError::IoError)?;
        let id_len = u16::from_le_bytes(id_len_bytes) as usize;
        
        // Read ID
        let mut id_bytes = vec![0u8; id_len];
        cursor.read_exact(&mut id_bytes).map_err(FileError::IoError)?;
        let key_id = String::from_utf8(id_bytes)
            .map_err(|_| FileError::CorruptedFile("Invalid UTF-8 in Key ID".to_string()))?;
            
        // Read Encrypted Data (Remaining)
        let pos = cursor.position() as usize;
        let encrypted_bytes = &data[pos..];
        
        // Derive Key using stored ID
        let file_key = key_hierarchy.derive_file_key(&key_id, &salt)
            .map_err(FileError::from)?;
            
        let encrypted: EncryptedData = serde_json::from_slice(encrypted_bytes)
            .map_err(|e| FileError::IoError(io::Error::new(io::ErrorKind::Other, e)))?;
            
        // Decrypt using stored ID as AAD
        match CryptoService::decrypt_xchacha20_aad(&encrypted, &file_key, key_id.as_bytes()) {
             Ok(data) => Ok(data),
             Err(e) => Err(FileError::CryptoError(CryptoError::DecryptionFailed(
                 format!("V2 Decryption failed: {}", e)
             )))
        }
    }

    fn load_attachment_legacy(kaps_path: &Path, total_buffer: &[u8], key_hierarchy: &KeyHierarchy) -> Result<Vec<u8>, FileError> {
        if total_buffer.len() < Self::SALT_LEN {
            return Err(FileError::CorruptedFile("File too short to contain salt".to_string()));
        }

        // Split Salt and Data
        let (salt, data) = total_buffer.split_at(Self::SALT_LEN);
        
        let stem = kaps_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
            
        // Derive Key
        let file_key = key_hierarchy.derive_file_key(stem, salt)
            .map_err(FileError::from)?;

        let encrypted: EncryptedData = serde_json::from_slice(data)
            .map_err(|e| FileError::IoError(io::Error::new(io::ErrorKind::Other, e)))?;

        // Decrypt with AAD
        let aad = stem.as_bytes();
        match CryptoService::decrypt_xchacha20_aad(&encrypted, &file_key, aad) {
            Ok(decrypted) => Ok(decrypted),
            Err(original_err) => {
                // FALLBACK: Try lowercase stem (to handle case-insensitive filesystem issues)
                let lower_stem = stem.to_lowercase();
                if lower_stem != stem {
                    if let Ok(lower_key) = key_hierarchy.derive_file_key(&lower_stem, salt) {
                        if let Ok(decrypted) = CryptoService::decrypt_xchacha20_aad(&encrypted, &lower_key, lower_stem.as_bytes()) {
                            return Ok(decrypted);
                        }
                    }
                }

                let debug_info = format!("Stem: '{}', AAD Len: {}, Path: {:?}", stem, aad.len(), kaps_path);
                Err(FileError::CryptoError(CryptoError::DecryptionFailed(format!("{} - Info: {}", original_err, debug_info))))
            }
        }
    }

    /// Creates a .kept backup bundle using V2 format.
    pub fn create_backup(
        vault_path: &Path,
        attachments_dir: &Path,
        backup_dest_path: &Path,
        key_hierarchy: &KeyHierarchy,
    ) -> Result<(), FileError> {
        let mut zip_buffer = Vec::new();

        {
            let mut zip_cursor = Cursor::new(&mut zip_buffer);
            let mut zip = zip::ZipWriter::new(&mut zip_cursor);

            let options = FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o600);

            if vault_path.exists() {
                zip.start_file("vault.kore", options)?;
                let mut f = File::open(vault_path)?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)?;
                zip.write_all(&buf)?;
            }

            if attachments_dir.exists() {
                for entry in fs::read_dir(attachments_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some(Self::EXT_FILE) {
                        let name = path.file_name().unwrap().to_str().unwrap();
                        zip.start_file(format!("attachments/{}", name), options)?;
                        let mut f = File::open(&path)?;
                        let mut buf = Vec::new();
                        f.read_to_end(&mut buf)?;
                        zip.write_all(&buf)?;
                    }
                }
            }

            zip.finish()?;
        }

        // Encryption logic for backup (V2 format)
        let backup_salt = CryptoService::generate_csprng_data(Self::SALT_LEN);
        let key_id = "keptr_backup"; 
        
        let backup_key = key_hierarchy.derive_file_key(key_id, &backup_salt)
             .map_err(FileError::from)?;
        
        let aad = key_id.as_bytes();
        let bundle_encrypted = CryptoService::encrypt_xchacha20_aad(&zip_buffer, &backup_key, aad)?;
        
        let encrypted_bytes = serde_json::to_vec(&bundle_encrypted)
            .map_err(|e| FileError::IoError(io::Error::new(io::ErrorKind::Other, e)))?;

        let mut out_file = File::create(backup_dest_path)?;
        
        // Write V2 Header
        out_file.write_all(b"KEPT")?;                 // 4 bytes Magic
        out_file.write_all(&[Self::VERSION_V2])?;     // 1 byte
        out_file.write_all(&backup_salt)?;            // 32 bytes
        
        let id_bytes = key_id.as_bytes();
        let id_len = id_bytes.len() as u16;
        out_file.write_all(&id_len.to_le_bytes())?;   // 2 bytes
        out_file.write_all(id_bytes)?;                // Variable bytes
        
        out_file.write_all(&encrypted_bytes)?;

        Ok(())
    }

    /// Restores a .kept backup (V2 format or Legacy).
    pub fn restore_backup(kept_path: &Path, key_hierarchy: &KeyHierarchy) -> Result<Vec<u8>, FileError> {
        if kept_path.extension().and_then(|s| s.to_str()) != Some(Self::EXT_BACKUP) {
            return Err(FileError::InvalidExtension("File must have .kept extension".to_string()));
        }

        let mut file = File::open(kept_path)?;
        let mut total_buffer = Vec::new();
        file.read_to_end(&mut total_buffer)?;
        
        if total_buffer.len() < 4 {
             return Err(FileError::CorruptedFile("Backup file too short".to_string()));
        }
        
        if total_buffer.starts_with(b"KEPT") {
            // -- V2 Backup Format --
            let mut cursor = Cursor::new(&total_buffer);
            cursor.set_position(5); // Skip Magic (4) + Version (1)
            
            let mut salt = [0u8; 32];
            cursor.read_exact(&mut salt).map_err(FileError::IoError)?;
            
            let mut id_len_bytes = [0u8; 2];
            cursor.read_exact(&mut id_len_bytes).map_err(FileError::IoError)?;
            let id_len = u16::from_le_bytes(id_len_bytes) as usize;
            
            let mut id_bytes = vec![0u8; id_len];
            cursor.read_exact(&mut id_bytes).map_err(FileError::IoError)?;
            let key_id = String::from_utf8(id_bytes)
                .map_err(|_| FileError::CorruptedFile("Invalid UTF-8 in Key ID".to_string()))?;
                
            let pos = cursor.position() as usize;
            let encrypted_bytes = &total_buffer[pos..];
            
            let backup_key = key_hierarchy.derive_file_key(&key_id, &salt).map_err(FileError::from)?;
            
            let encrypted: EncryptedData = serde_json::from_slice(encrypted_bytes)
                .map_err(|e| FileError::IoError(io::Error::new(io::ErrorKind::Other, e)))?;
                
            match CryptoService::decrypt_xchacha20_aad(&encrypted, &backup_key, key_id.as_bytes()) {
                 Ok(data) => Ok(data),
                 Err(e) => Err(FileError::CryptoError(CryptoError::DecryptionFailed(format!("V2 Backup Decryption failed: {}", e))))
            }
        } else {
            // -- Legacy Backup Format --
            if total_buffer.len() < Self::SALT_LEN {
                 return Err(FileError::CorruptedFile("Backup file too short".to_string()));
            }
            
            let (salt, data) = total_buffer.split_at(Self::SALT_LEN);
            let stem = kept_path.file_stem().and_then(|s| s.to_str()).unwrap_or("backup");
            
            let backup_key = key_hierarchy.derive_file_key(stem, salt).map_err(FileError::from)?;
            
            let encrypted: EncryptedData = serde_json::from_slice(data)
                .map_err(|e| FileError::IoError(io::Error::new(io::ErrorKind::Other, e)))?;

            let aad = stem.as_bytes();
            CryptoService::decrypt_xchacha20_aad(&encrypted, &backup_key, aad).map_err(FileError::from)
        }
    }
}

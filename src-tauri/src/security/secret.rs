use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::{Zeroize, Zeroizing};

/// A string type that protects sensitive data in memory.
///
/// Wraps `zeroize::Zeroizing<String>` for automatic zeroization on drop.
/// Debug and Display print `[REDACTED]` to prevent log leakage.
/// Serialization exposes the value only for frontend transport or encryption.
#[derive(Clone)]
pub struct SecretString(Zeroizing<String>);

impl Default for SecretString {
    fn default() -> Self {
        Self(Zeroizing::new(String::new()))
    }
}

impl SecretString {
    pub fn new(s: String) -> Self {
        Self(Zeroizing::new(s))
    }

    /// Access the underlying string value. Use sparingly.
    pub fn exposed(&self) -> &str {
        &self.0
    }
}

impl Zeroize for SecretString {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl std::fmt::Display for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl Serialize for SecretString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.exposed().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SecretString::new(s))
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        SecretString::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        SecretString::new(s.to_string())
    }
}

use zeroize::{Zeroize, ZeroizeOnDrop};

/// A secure wrapper for a sensitive key or password.
/// It automatically zeros the memory when dropped.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

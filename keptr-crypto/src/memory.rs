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

    pub fn generate_random(len: usize) -> Self {
        use rand::RngCore;
        let mut data = vec![0u8; len];
        rand::rngs::OsRng.fill_bytes(&mut data);
        Self(data)
    }
}

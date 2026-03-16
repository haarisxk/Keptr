//! Shamir's Secret Sharing Scheme (SSSS)
//!
//! Splits a secret into n shares where any k (threshold) shares can
//! reconstruct the original, but k-1 shares reveal nothing.
//! Uses finite field arithmetic over GF(256).

use super::CryptoError;
use super::csprng;

/// Configuration for secret sharing.
#[derive(Debug, Clone)]
pub struct ShamirConfig {
    /// Minimum shares required to reconstruct (threshold).
    pub threshold: u8,
    /// Total number of shares to generate.
    pub total_shares: u8,
}

impl Default for ShamirConfig {
    fn default() -> Self {
        Self {
            threshold: 3,
            total_shares: 5,
        }
    }
}

impl ShamirConfig {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), CryptoError> {
        if self.threshold < 2 {
            return Err(CryptoError::ShamirError("Threshold must be at least 2".to_string()));
        }
        if self.threshold > self.total_shares {
            return Err(CryptoError::ShamirError("Threshold cannot exceed total shares".to_string()));
        }
        if self.total_shares < 2 {
            return Err(CryptoError::ShamirError("Must generate at least 2 shares".to_string()));
        }
        Ok(())
    }
}

/// A single share of a split secret.
#[derive(Debug, Clone)]
pub struct SecretShare {
    /// Share index (1-indexed, used as the X coordinate).
    pub index: u8,
    /// Share data (Y coordinates for each byte of the secret).
    pub data: Vec<u8>,
    /// Threshold required to reconstruct.
    pub threshold: u8,
}

impl SecretShare {
    /// Creates a new share.
    pub fn new(index: u8, data: Vec<u8>, threshold: u8) -> Self {
        Self { index, data, threshold }
    }

    /// Serializes the share to a portable byte format.
    ///
    /// Format: `[index (1)][threshold (1)][length (1)][data (N)]`
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(3 + self.data.len());
        bytes.push(self.index);
        bytes.push(self.threshold);
        bytes.push(self.data.len() as u8);
        bytes.extend_from_slice(&self.data);
        bytes
    }

    /// Deserializes a share from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() < 3 {
            return Err(CryptoError::ShamirError("Share too short".to_string()));
        }

        let index = bytes[0];
        let threshold = bytes[1];
        let len = bytes[2] as usize;

        if bytes.len() != 3 + len {
            return Err(CryptoError::ShamirError("Share length mismatch".to_string()));
        }

        Ok(Self {
            index,
            threshold,
            data: bytes[3..].to_vec(),
        })
    }
}

/// GF(256) finite field arithmetic using the AES irreducible polynomial
/// x^8 + x^4 + x^3 + x + 1 (0x11B).
mod gf256 {
    /// Multiplication in GF(256).
    pub fn mul(a: u8, b: u8) -> u8 {
        let mut result = 0u8;
        let mut a = a;
        let mut b = b;

        while b != 0 {
            if b & 1 != 0 {
                result ^= a;
            }
            let high_bit = a & 0x80;
            a <<= 1;
            if high_bit != 0 {
                a ^= 0x1B;
            }
            b >>= 1;
        }

        result
    }

    /// Multiplicative inverse in GF(256) using Fermat's little theorem.
    /// In GF(2^8), a^(2^8 - 1) = 1, so a^(-1) = a^(254).
    pub fn inv(a: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        // a^254 via binary exponentiation
        // 254 = 0b11111110
        let mut result = 1u8;
        let mut base = a;
        let mut exp = 254u32;
        while exp > 0 {
            if exp & 1 == 1 {
                result = mul(result, base);
            }
            base = mul(base, base);
            exp >>= 1;
        }
        result
    }

    /// Division in GF(256).
    pub fn div(a: u8, b: u8) -> u8 {
        assert!(b != 0, "Division by zero in GF(256)");
        mul(a, inv(b))
    }
}

/// Evaluates a polynomial at a given point in GF(256).
fn evaluate_polynomial(coefficients: &[u8], x: u8) -> u8 {
    let mut result = 0u8;
    let mut x_power = 1u8;

    for &coeff in coefficients {
        result ^= gf256::mul(coeff, x_power);
        x_power = gf256::mul(x_power, x);
    }

    result
}

/// Splits a secret into shares using Shamir's Secret Sharing.
///
/// Any `threshold` shares can reconstruct the original; fewer reveal nothing.
pub fn split_secret(secret: &[u8], config: &ShamirConfig) -> Result<Vec<SecretShare>, CryptoError> {
    config.validate()?;

    let mut shares: Vec<Vec<u8>> = vec![
        Vec::with_capacity(secret.len());
        config.total_shares as usize
    ];

    for &secret_byte in secret {
        let mut coefficients = vec![0u8; config.threshold as usize];
        coefficients[0] = secret_byte;

        // Random coefficients for higher-degree terms
        let random_coeffs = csprng::generate((config.threshold - 1) as usize);
        coefficients[1..].copy_from_slice(&random_coeffs);

        for i in 0..config.total_shares {
            let x = i + 1;
            let y = evaluate_polynomial(&coefficients, x);
            shares[i as usize].push(y);
        }
    }

    let result: Vec<SecretShare> = shares
        .into_iter()
        .enumerate()
        .map(|(i, data)| SecretShare::new((i + 1) as u8, data, config.threshold))
        .collect();

    Ok(result)
}

/// Lagrange interpolation in GF(256) to recover the constant term.
fn lagrange_interpolate(points: &[(u8, u8)]) -> u8 {
    let mut result = 0u8;

    for i in 0..points.len() {
        let (xi, yi) = points[i];
        let mut term = yi;

        for j in 0..points.len() {
            if i != j {
                let (xj, _) = points[j];
                let numerator = xj;
                let denominator = xi ^ xj;
                term = gf256::mul(term, gf256::div(numerator, denominator));
            }
        }

        result ^= term;
    }

    result
}

/// Reconstructs a secret from shares using Lagrange interpolation.
///
/// Requires at least `threshold` shares.
pub fn combine_shares(shares: &[SecretShare]) -> Result<Vec<u8>, CryptoError> {
    if shares.is_empty() {
        return Err(CryptoError::ShamirError("No shares provided".to_string()));
    }

    let threshold = shares[0].threshold;

    if shares.len() < threshold as usize {
        return Err(CryptoError::ShamirError(
            format!("Need at least {} shares, got {}", threshold, shares.len()),
        ));
    }

    // Validate consistency
    let share_len = shares[0].data.len();
    for share in shares {
        if share.data.len() != share_len {
            return Err(CryptoError::ShamirError("Share lengths don't match".to_string()));
        }
        if share.threshold != threshold {
            return Err(CryptoError::ShamirError("Share thresholds don't match".to_string()));
        }
    }

    // Check for duplicate indices
    let mut seen = std::collections::HashSet::new();
    for share in shares {
        if !seen.insert(share.index) {
            return Err(CryptoError::ShamirError("Duplicate share indices".to_string()));
        }
    }

    let shares_to_use = &shares[..threshold as usize];

    let mut result = Vec::with_capacity(share_len);
    for byte_idx in 0..share_len {
        let points: Vec<(u8, u8)> = shares_to_use
            .iter()
            .map(|s| (s.index, s.data[byte_idx]))
            .collect();

        result.push(lagrange_interpolate(&points));
    }

    Ok(result)
}

/// Verifies that a set of shares reconstructs to the expected secret.
pub fn verify_shares(shares: &[SecretShare], expected: &[u8]) -> Result<bool, CryptoError> {
    if shares.is_empty() {
        return Ok(false);
    }

    let threshold = shares[0].threshold;
    if shares.len() < threshold as usize {
        return Ok(false);
    }

    let reconstructed = combine_shares(shares)?;
    Ok(reconstructed == expected)
}

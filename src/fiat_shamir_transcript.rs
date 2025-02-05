use ark_ff::{PrimeField, BigInteger};
use sha3::{Digest, Keccak256};

/// Implements a Fiat-Shamir transcript using the Keccak256 hash function.
pub struct FiatShamirTranscript {
    hasher: Keccak256,
}

impl FiatShamirTranscript {
    /// Initializes a new transcript instance.
    pub fn new() -> Self {
        Self {
            hasher: Keccak256::new(),
        }
    }

    /// Appends new data to the transcript.
    pub fn absorb(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Generates a challenge from the current transcript state.
    fn generate_challenge(&mut self) -> [u8; 32] {
        let mut result = [0; 32];
        result.copy_from_slice(&self.hasher.finalize_reset());
        self.hasher.update(result);
        result
    }

    /// Samples a field element from the transcript.
    pub fn sample_field_element<F: PrimeField>(&mut self) -> F {
        let challenge = self.generate_challenge();
        F::from_be_bytes_mod_order(&challenge)
    }
}
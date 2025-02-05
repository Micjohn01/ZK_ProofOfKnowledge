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

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;

    #[test]
    fn test_transcript_initialization() {
        let _transcript = FiatShamirTranscript::new();
    }

    #[test]
    fn test_transcript_absorb() {
        let mut transcript = FiatShamirTranscript::new();
        transcript.absorb(b"test data");
    }

    #[test]
    fn test_generate_challenge() {
        let mut transcript = FiatShamirTranscript::new();
        transcript.absorb(b"some input");
        let challenge = transcript.generate_challenge();
        assert_eq!(challenge.len(), 32);
    }

    #[test]
    fn test_sample_field_element() {
        let mut transcript = FiatShamirTranscript::new();
        transcript.absorb(b"field element test");
        let element: Fr = transcript.sample_field_element();
        assert!(element.into_bigint().to_bytes_be().len() > 0);
    }
}

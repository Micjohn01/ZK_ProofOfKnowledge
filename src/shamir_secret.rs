use ark_ff::{Field, PrimeField};
use ark_std::{rand::Rng, UniformRand};
use std::collections::HashMap;

// A specific field type is use by using a 256-bit prime field from arkworks.
use ark_bn254::Fr as FieldElement;

/// This is a function to generate shares of a secret
fn generate_shares<F: Field + UniformRand>(
    secret: F,
    threshold: usize,
    num_shares: usize,
) -> HashMap<F, F> {
    let mut rng = ark_std::test_rng();
    let mut coefficients: Vec<F> = vec![secret]; // This gives the coefficients of the polynomial

    // Random coefficients for the polynomial
    for _ in 1..threshold {
        coefficients.push(F::rand(&mut rng));
    }
 
    let mut shares: HashMap<F, F> = HashMap::new();
    for i in 1..=num_shares {
        let x: F = F::from(i as u64); // Let's convert explicitly
        let mut y = F::zero();
        for (j, coeff) in coefficients.iter().enumerate() {
            y += *coeff * x.pow(&[j as u64]); // Horner's method for polynomial evaluation
        }
        shares.insert(x, y);
    }

    shares
}

/// This is a function to reconstruct the secret from shares
fn reconstruct_secret<F: PrimeField>(shares: &HashMap<F, F>) -> F {
    let mut secret: F = F::zero();

    for (&x_i, &y_i) in shares.iter() {
        let mut numerator = F::one();
        let mut denominator = F::one();

        for (&x_j, _) in shares.iter() {
            if x_i != x_j {
                numerator *= -x_j;
                denominator *= x_i - x_j;
            }
        }

        secret += y_i * (numerator * denominator.inverse().unwrap());
    }

    secret
}

// fn main() {
//     // We are defining the secret as a field element
//     let secret = FieldElement::from(42u64); // Explicit type for clarity
//     let threshold = 3;
//     let num_shares = 5;

//     // Generate shares
//     let shares = generate_shares(secret, threshold, num_shares);
//     println!("Shares: {:?}", shares);

//     // Taking a subset of the shares to reconstruct the secret
//     let subset_of_shares: HashMap<_, _> = shares.iter().take(threshold).map(|(&x, &y)| (x, y)).collect();

//     // Reconstructing the secret
//     let reconstructed_secret = reconstruct_secret(&subset_of_shares);
//     println!("Reconstructed Secret: {}", reconstructed_secret);

//     // Asserting that the original secret matches the reconstructed secret
//     assert_eq!(secret, reconstructed_secret);
// }

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr as FieldElement;
    use std::collections::HashMap;

    /// Helper function to generate a subset of shares
    fn subset_of_shares<F: Field>(
        shares: &HashMap<F, F>,
        threshold: usize,
    ) -> HashMap<F, F> {
        shares.iter().take(threshold).map(|(&x, &y)| (x, y)).collect()
    }

    #[test]
    fn test_generate_and_reconstruct_secret() {
        // Defininng a secret and parameters
        let secret = FieldElement::from(123u64);
        let threshold = 3;
        let num_shares = 5;

        // Generating share for the secret
        let shares = generate_shares(secret, threshold, num_shares);

        // Ensuring that the correct number of shares were generated
        assert_eq!(shares.len(), num_shares);

        // Using only `threshold` shares to reconstruct the secret
        let subset = subset_of_shares(&shares, threshold);
        let reconstructed_secret = reconstruct_secret(&subset);

        // Check if the reconstructed secret matches the original secret
        assert_eq!(secret, reconstructed_secret);
    }

    #[test]
    fn test_reconstruction_with_exact_threshold() {
        let secret = FieldElement::from(42u64);
        let threshold = 4;
        let num_shares = 7;

        let shares = generate_shares(secret, threshold, num_shares);
        let subset = subset_of_shares(&shares, threshold);

        let reconstructed_secret = reconstruct_secret(&subset);

        assert_eq!(secret, reconstructed_secret);
    }

    #[test]
    fn test_reconstruction_with_all_shares() {
        let secret = FieldElement::from(314u64);
        let threshold = 3;
        let num_shares = 6;

        let shares = generate_shares(secret, threshold, num_shares);
        let reconstructed_secret = reconstruct_secret(&shares);

        assert_eq!(secret, reconstructed_secret);
    }

    #[test]
    fn test_reconstruction_with_too_few_shares() {
        let secret = FieldElement::from(999u64);
        let threshold = 5;
        let num_shares = 8;

        let shares = generate_shares(secret, threshold, num_shares);
        let subset = subset_of_shares(&shares, threshold - 1); // One less than required

        // Reconstructing with fewer than `threshold` shares should fail (or return incorrect value)
        let reconstructed_secret = reconstruct_secret(&subset);

        // The secret should not match, as there are not enough shares
        assert_ne!(secret, reconstructed_secret);
    }

    #[test]
    fn test_different_secret_values() {
        // Test with different secrets and ensure reconstruction works
        let secrets = vec![1u64, 123u64, 1024u64, 98765u64];

        for secret_value in secrets {
            let secret = FieldElement::from(secret_value);
            let threshold = 3;
            let num_shares = 5;

            let shares = generate_shares(secret, threshold, num_shares);
            let subset = subset_of_shares(&shares, threshold);
            let reconstructed_secret = reconstruct_secret(&subset);

            assert_eq!(secret, reconstructed_secret);
        }
    }

}

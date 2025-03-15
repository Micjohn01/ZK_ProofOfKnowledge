use crate::{sum_all_polys::PolynomialMetadata, univariate_polynomial::SparseUnivariatePolynomial};
use crate::sum_all_polys::{PolynomialArithmetic, SumPolynomial};
use crate::fiat_shamir_transcript::FiatShamirTranscript;
use ark_ff::{PrimeField, BigInteger};

#[derive(Clone, Debug)]
pub struct SumcheckProver<F: PrimeField> {
    pub claimed_sum: F,
    pub round_univariate_polynomials: Vec<SparseUnivariatePolynomial<F>>,
    pub random_challenges: Vec<F>,
}

#[derive(Clone, Debug)]
pub struct SumcheckVerifier<F: PrimeField> {
    pub is_proof_valid: bool,
    pub random_challenges: Vec<F>,
    pub last_claimed_sum: F,
}

/// Serializes a univariate polynomial into a byte vector.
pub fn univariate_to_bytes<F: PrimeField>(univariate_poly: &SparseUnivariatePolynomial<F>) -> Vec<u8> {
    univariate_poly
        .terms
        .iter()
        .flat_map(|(power, coeff)| {
            let mut bytes = power.to_be_bytes().to_vec();
            bytes.extend(coeff.into_bigint().to_bytes_be());
            bytes
        })
        .collect()
}

/// Serializes a field element into a byte vector (big-endian).
pub fn field_element_to_bytes<F: PrimeField>(field_element: F) -> Vec<u8> {
    field_element.into_bigint().to_bytes_be()
}

pub fn prove<F: PrimeField>(
    sum_polynomial: SumPolynomial<F>,
    claimed_sum: F,
    transcript: &mut FiatShamirTranscript,
) -> SumcheckProver<F> {
    let num_vars = sum_polynomial.number_of_variables();
    let mut round_univariate_polynomials = Vec::new();
    let mut random_challenges = Vec::new();
    let mut current_polynomial = sum_polynomial;

    // Append the claimed sum to the transcript
    transcript.absorb(&field_element_to_bytes(claimed_sum));

    for _round in 0..num_vars {
        // Generate the univariate polynomial for this round
        let univariate_poly = generate_round_univariate(&current_polynomial);
        round_univariate_polynomials.push(univariate_poly.clone());

        // Append the polynomial to the transcript
        let poly_bytes = univariate_to_bytes(&univariate_poly);
        transcript.absorb(&poly_bytes);

        // Generate a random challenge from the transcript
        let random_challenge = transcript.sample_field_element();
        random_challenges.push(random_challenge);

        // Partially evaluate the polynomial at the random challenge
        current_polynomial = current_polynomial.partial_evaluate(0, random_challenge);
    }

    SumcheckProver {
        claimed_sum,
        round_univariate_polynomials,
        random_challenges,
    }
}

pub fn verify<F: PrimeField>(
    proof: &SumcheckProver<F>,
    transcript: &mut FiatShamirTranscript,
) -> SumcheckVerifier<F> {
    let mut current_sum = proof.claimed_sum;
    let mut random_challenges = Vec::new();

    // Append the claimed sum to the transcript
    transcript.absorb(&field_element_to_bytes(current_sum));

    for (i, round_poly) in proof.round_univariate_polynomials.iter().enumerate() {
        // Verify that g_i(0) + g_i(1) == current_sum
        let eval_at_zero = round_poly.evaluate(F::zero());
        let eval_at_one = round_poly.evaluate(F::one());
        if eval_at_zero + eval_at_one != current_sum {
            return SumcheckVerifier {
                is_proof_valid: false,
                random_challenges,
                last_claimed_sum: current_sum,
            };
        }

        // Append the polynomial to the transcript
        let poly_bytes = univariate_to_bytes(round_poly);
        transcript.absorb(&poly_bytes);

        // Generate a random challenge from the transcript
        let random_challenge = transcript.sample_field_element();
        random_challenges.push(random_challenge);

        // Update the current sum for the next round
        current_sum = round_poly.evaluate(random_challenge);
    }

    SumcheckVerifier {
        is_proof_valid: true,
        random_challenges,
        last_claimed_sum: current_sum,
    }
}

fn generate_round_univariate<F: PrimeField>(current_polynomial: &SumPolynomial<F>) -> SparseUnivariatePolynomial<F> {
    let degree = current_polynomial.degree();
    let num_evaluations = degree + 1;

    let mut evaluations = Vec::new();
    for i in 0..num_evaluations {
        let value = F::from(i as u64);
        let partial_eval_sum_poly = current_polynomial.partial_evaluate(0, value);
        let evaluation = partial_eval_sum_poly.evaluate(&[]); // Evaluate at all zeros
        evaluations.push((i, evaluation));
    }

    SparseUnivariatePolynomial::new(evaluations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::{Field, Zero, One};
    use crate::product_poly::ProductPolynomial;
    use crate::multi_poly::MultiPoly;

    // Helper function to create a simple SumPolynomial for testing
    fn create_test_sum_polynomial<F: PrimeField>() -> SumPolynomial<F> {
        // Example: f(x, y) = x^2 + 2xy + y
        // Sum over {0,1}^2: f(0,0) = 0, f(0,1) = 1, f(1,0) = 1, f(1,1) = 4
        // Total sum = 0 + 1 + 1 + 4 = 6
        let poly1 = MultiPoly::new(2, vec![F::one(), F::zero(), F::zero(), F::one()]); // x^2
        let poly2 = MultiPoly::new(2, vec![F::zero(), F::from(2u64), F::zero(), F::zero()]); // 2xy
        let poly3 = MultiPoly::new(2, vec![F::zero(), F::one(), F::zero(), F::zero()]); // y
        let prod_poly1 = ProductPolynomial::new(vec![poly1]);
        let prod_poly2 = ProductPolynomial::new(vec![poly2]);
        let prod_poly3 = ProductPolynomial::new(vec![poly3]);
        SumPolynomial::new(vec![prod_poly1, prod_poly2, prod_poly3])
    }

    // Helper function to compute the expected sum over all boolean inputs
    fn compute_expected_sum<F: PrimeField>(poly: &SumPolynomial<F>) -> F {
        let mut sum = F::zero();
        for x in 0..2 {
            for y in 0..2 {
                let inputs = vec![F::from(x), F::from(y)];
                sum += poly.evaluate(&inputs);
            }
        }
        sum
    }

    #[test]
    fn test_prove_and_verify_correct_sum() {
        let mut transcript = FiatShamirTranscript::new();
        let sum_poly = create_test_sum_polynomial::<Fr>();
        let claimed_sum = compute_expected_sum(&sum_poly); // Should be 6

        // Generate proof
        let proof = prove(sum_poly.clone(), claimed_sum, &mut transcript);

        // Reset transcript for verification
        let mut transcript = FiatShamirTranscript::new();
        let verifier_result = verify(&proof, &mut transcript);

        assert!(verifier_result.is_proof_valid, "Verification should pass with correct sum");
        assert_eq!(proof.claimed_sum, claimed_sum, "Claimed sum should match expected sum");
        assert_eq!(
            proof.random_challenges.len(),
            sum_poly.number_of_variables(),
            "Number of challenges should match number of variables"
        );
        assert_eq!(
            proof.random_challenges, verifier_result.random_challenges,
            "Random challenges should match between prover and verifier"
        );
    }

    
}



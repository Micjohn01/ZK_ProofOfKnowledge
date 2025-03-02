// use ark_ff::{BigInteger, PrimeField};

// use crate::multi_poly::*;
// use crate::fiat_shamir_transcript::*;


// /// Represents a proof for the sumcheck protocol.
// pub struct SumcheckProof<F: PrimeField> {
//     claimed_sum: F,
//     round_polynomials: Vec<[F; 2]>,
// }

// /// Generates a proof for a claimed sum of a multilinear polynomial.
// pub fn generate_proof<F: PrimeField>(poly: &MultiPoly<F>, claimed_sum: F) -> SumcheckProof<F> {
//     let mut round_polynomials = Vec::new();
//     let mut transcript = FiatShamirTranscript::new();

//     transcript.absorb(&poly.coefficients.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>());
//     transcript.absorb(&claimed_sum.into_bigint().to_bytes_be());

//     let mut poly = poly.clone();
//     for _ in 0..poly.num_vars {
//         let round_poly: [F; 2] = [
//             poly.partial_eval(0, &F::zero()).coefficients.iter().sum(),
//             poly.partial_eval(0, &F::one()).coefficients.iter().sum(),
//         ];

//         transcript.absorb(&round_poly.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>());
//         round_polynomials.push(round_poly);
//         let challenge = transcript.sample_field_element();
//         let _poly = &poly.partial_eval(0, &challenge);
//     }

//     SumcheckProof { claimed_sum, round_polynomials }
// }

// /// Verifies a sumcheck proof.
// pub fn verify_proof<F: PrimeField>(poly: &MultiPoly<F>, proof: &SumcheckProof<F>) -> bool {
//     if proof.round_polynomials.len() != poly.num_vars {
//         return false;
//     }
    
//     let mut transcript = FiatShamirTranscript::new();
//     transcript.absorb(&poly.coefficients.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>());
//     transcript.absorb(&proof.claimed_sum.into_bigint().to_bytes_be());
    
//     let mut claimed_sum = proof.claimed_sum;
//     let mut challenges = vec![];

//     for round_poly in &proof.round_polynomials {
//         if claimed_sum != round_poly.iter().sum() {
//             return false;
//         }
//         transcript.absorb(&round_poly.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>());
//         let challenge = transcript.sample_field_element();
//         claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
//         challenges.push(challenge);
//     }

//     claimed_sum == poly.evaluate(&challenges)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use ark_bn254::Fr;

//     fn to_field(values: Vec<u64>) -> Vec<Fr> {
//         values.into_iter().map(Fr::from).collect()
//     }

//     #[test]
//     fn test_generate_proof() {
//         let poly = MultiPoly::new(2, to_field(vec![2, 4, 6, 8]));
//         let claimed_sum = poly.coefficients.iter().sum();
//         let proof = generate_proof(&poly, claimed_sum);
//         assert_eq!(proof.round_polynomials.len(), 2);
//     }

//     #[test]
//     fn test_verify_valid_proof() {
//         let poly = MultiPoly::new(2, to_field(vec![2, 4, 6, 8]));
//         let claimed_sum = poly.coefficients.iter().sum();
//         let proof = generate_proof(&poly, claimed_sum);
//         assert!(verify_proof(&poly, &proof));
//     }

//     #[test]
//     fn test_verify_invalid_proof() {
//         let poly = MultiPoly::new(2, to_field(vec![2, 4, 6, 8]));
//         let mut proof = generate_proof(&poly, Fr::from(100)); // Incorrect sum
//         assert!(!verify_proof(&poly, &proof));

//         proof.round_polynomials[0] = [Fr::from(1), Fr::from(1)]; // Corrupting proof
//         assert!(!verify_proof(&poly, &proof));
//     }
// }
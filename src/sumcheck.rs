use ark_ff::{BigInteger, PrimeField};
use ark_std::rand::Rng;
use ark_std::test_rng;

use crate::updated_multilinear::*;
use crate::transcript::*;
struct Proof<F: PrimeField> {
    claimed_sum: F,
    round_polys: Vec<[F; 2]>
}

fn prove<F: PrimeField>(poly: &MultilinearPolynomial<F>, claimed_sum: F) -> Proof<F> {
    let mut round_polys =  vec![];

    // public
    // poly
    // claimed_sum

    let mut transcript = Transcript::new();
    // append takes &[u8]
    // [&[u8], &[u8], ...]
    // [[1,2], [3,4]]
    // [1,2,3,4]
    transcript.append(poly.evals.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>().as_slice(),);
    transcript.append(claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut poly = poly.clone();

    for _ in 0..poly.n_vars {
        let round_poly: [F; 2] = [poly.partial_evaluate(0, &F::zero()).evals.iter().sum(), poly.partial_evaluate(0, &F::one()).evals.iter().sum()];

    transcript.append(round_poly.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>().as_slice());

    round_polys.push(round_poly);

    let challenge = transcript.sample_field_element();
    
    poly = poly.partial_evaluate(0, &challenge);
    }
    
    Proof {
        claimed_sum,
        round_polys
    }
}

fn verify<F: PrimeField>(poly: &MultilinearPolynomial<F>, proof: &Proof<F>) -> bool {
    if proof.round_polys.len() != poly.n_vars {
        return false;
    }

    let mut challenges = vec![];

    let mut transcript = Transcript::new();

    transcript.append(poly.evals.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>().as_slice(),);
    transcript.append(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

    let mut claimed_sum = proof.claimed_sum;

    for round_poly in &proof.round_polys {
        if claimed_sum != round_poly.iter().sum(){
            return false;
        }

        transcript.append(round_poly.iter().flat_map(|f| f.into_bigint().to_bytes_be()).collect::<Vec<_>>().as_slice(),);


        let challenge = transcript.sample_field_element();

        claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]); 

        challenges.push(challenge);
    }
    
    if claimed_sum != poly.evaluate(&challenges) {
        return false;
    }

    true
}

fn get_large_poly() -> MultilinearPolynomial<Fr> {
    let n_vars = 20;
    let size = 1 << n_vars;
    let mut rng = test_rng();

    let poly: Vec<Fr> = (0..size).map(|_| Fr::from(rng.gen_range(0..20))).collect();

    MultilinearPolynomial::new ( evals, n_vars )
} 

// fn main(){
    
// }

#[cfg(test)]

mod test {
    use crate::updated_multilinear::MultilinearPolynomial;
    // use ark_bn254::Fr;
    use crate::sumcheck::{prove, verify};
    use field_tracker::{print_summary, Ft};

    type Fr = Ft!(ark_bn254::Fr);

    #[test]
    fn test_sumcheck() {
        let poly = 
        MultilinearPolynomial::new(3, vec![Fr::from(0), Fr::from(0), Fr::from(0), Fr::from(3), Fr::from(0), Fr::from(0), Fr::from(2), Fr::from(5)]);

        let proof = prove(&poly, Fr::from(10));

        print_summary!();
        // verify 
    }
}
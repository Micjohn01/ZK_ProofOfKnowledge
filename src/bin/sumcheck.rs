use ark_ff::PrimeField;
use crate::updated_multilinear::MultilinearPolynomial;
use crate::transcript::Transcript;

struct Proof<F: PrimeField> {
    claimed_sum: F,
    round_polys: Vec<[F; 2]>
}

fn prove<F: PrimeField>(poly: &MultilinearPolynomial<F>, claimed_sum: F) -> Proof<F> {
    let mut round_polys =  vec![];
    todo!()
}

fn verify<F: PrimeField>(poly: &MultilinearPolynomial<F>, proof: &Proof<F>) -> bool {
    todo!()
}

fn main(){
    
}
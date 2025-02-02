use ark_ff::PrimeField;

struct MultilinearPolynomial<F: PrimeField> {
    evals: Vec<F>,
}

impl<F: PrimeField> MultilinearPolynomial<F> {
    fn new(n_vars: usize, evaluations: Vec<F>) -> Self {
        if evaluations.len() != << n_vars {
            panic!() 
        }
    }
    fn evaluate(&self, assignments: &[F]) -> F {
        todo!()
    }
}


fn main (){
    
}
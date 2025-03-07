

use std::result;

use ark_ff::PrimeField;
use crate::new_circuit::*;

#[derive(Clone)]
pub(crate) struct MultilinearPolynomial<F: PrimeField> {
    pub(crate)evals: Vec<F>,
    pub(crate)n_vars: usize,
}

impl<F: PrimeField> MultilinearPolynomial<F> {
    pub(crate) fn new(n_vars: usize, evaluations: Vec<F>) -> Self {
        if evaluations.len() != 1 << n_vars {
            panic!("You are doing it wrong"); 
        }

        Self {
            evals: evaluations,
            n_vars,
        }
    }

    // f(a,b,c)
    // f(3,2,4) -> [3,2,4] = These are the assignments
    pub fn evaluate(&self, assignments: &[F]) -> F {
        if assignments.len() != self.n_vars {
            panic! ("You are still doing it all wrong");
        }

        let mut polynomial = self.clone();

        for val in assignments {
            polynomial = polynomial.partial_evaluate( 0, val);
        }

        polynomial.evals[0]
        
    }

    pub fn partial_evaluate(&self, index: usize, value:&F) -> Self{
        // Use index to generate  pairing
        // linear interpolate and evaluate

        // 00 - (000, 001) - (0, 4)
        // 01 - (001, 101) - (1, 5)
        // 10 - (010, 110) - (2, 6)
        // 11 - (011, 111) - (3, 7)

        let mut result = vec![];
        // What does the pairs need?
        // index <- 0 -> a
        // /len of hypercube
        for (a, b) in pairs(index, self.n_vars).into_iter()  {
            let a = self.evals[a];
            let b = self.evals[b];
            result.push(a + *value * (b - a));
        }

        Self::new(self.n_vars - 1, result)
        
    }
}
    // example
    // 3 vars
    // target_hypercube = 3 - 1 = 2
    // 0..2^2 => 0..4
    // This will generate the hypercube for the lower dimension
    // 0 - 00
    // 1 - 01
    // 2 - 10
    // 3 - 11......The index here is starting from the front.
    pub fn pairs(index: usize, n_vars: usize) -> Vec<(usize, usize)> { 
        let mut result =  vec![];
        let target_hypercube =  n_vars - 1;
        for val in 0..(1 << target_hypercube) {
            let inverted_index =  n_vars - index - 1;
            let insert_zero = insert_bit(val, inverted_index);
            let insert_one = insert_zero | (1 << inverted_index);
            result.push((insert_zero, insert_one));
        }
        result

    }

    // always inserts 0
    // 3 insert 0 at index 1 insert_bit(3,1)
    // 11 -> 101
    fn insert_bit(value: usize, index: usize) -> usize {
        // high bit
        // 1011
        // right shift twice 101 10

        // insert at 0
        // 11 -> 110
        // insert at 2
        // 11 -> 011

        // 1011 & 0011
        // 1 << 2 =  100
        // 100 - 1 = 11
        // index is the same thing as length of the low
        let high = value >> index;
        let mask =  (1 << index) - 1;
        let low = value & mask;

        // high | new_bit | low

        high << index + 1 | low
    }





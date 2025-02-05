use ark_ff::PrimeField;

pub struct MultiPoly<F: PrimeField> {
   pub coefficients: Vec<F>,
    pub num_vars: usize,
}

impl<F: PrimeField> MultiPoly<F> {
    /// Constructs a new multilinear polynomial.
    pub fn new(num_vars: usize, coefficients: Vec<F>) -> Self {
        assert_eq!(coefficients.len(), 1 << num_vars, "Invalid number of coefficients");
        Self { coefficients, num_vars }
    }

    /// Evaluates the polynomial at a given assignment.
    pub fn evaluate(&self, assignment: &[F]) -> F {
        assert_eq!(assignment.len(), self.num_vars, "Incorrect number of assignments");
        let mut poly: &MultiPoly<F> = self.clone();
        for val in assignment {
            poly =&poly.partial_eval(0, val);
        }
        poly.coefficients[0]
    }

    /// Partially evaluates the polynomial at a given variable index.
    pub fn partial_eval(&self, index: usize, value: &F) -> Self {
        let mut result = vec![];
        for (a, b) in Self::generate_pairs(index, self.num_vars).into_iter() {
            let a = self.coefficients[a];
            let b = self.coefficients[b];
            result.push(a + *value * (b - a));
        }
        Self::new(self.num_vars - 1, result)
    }

    /// Generates index pairs for partial evaluation.
    pub fn generate_pairs(index: usize, num_vars: usize) -> Vec<(usize, usize)> {
        let mut result = vec![];
        let target_hypercube = num_vars - 1;
        for val in 0..(1 << target_hypercube) {
            let inverted_index = num_vars - index - 1;
            let zero_inserted = Self::insert_zero_bit(val, inverted_index);
            let one_inserted = zero_inserted | (1 << inverted_index);
            result.push((zero_inserted, one_inserted));
        }
        result
    }

    /// Inserts a zero bit at a specified position.
    fn insert_zero_bit(value: usize, index: usize) -> usize {
        let high = value >> index;
        let mask = (1 << index) - 1;
        let low = value & mask;
        (high << (index + 1)) | low
    }
}
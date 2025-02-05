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
            let _poly = &poly.partial_eval(0, val);
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

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::BigInteger;

    fn to_field(values: Vec<u64>) -> Vec<Fr> {
        values.into_iter().map(Fr::from).collect()
    }

    #[test]
    fn test_polynomial_construction() {
        let poly = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        assert_eq!(poly.coefficients.len(), 4);
    }

    #[test]
    fn test_evaluate() {
        let poly = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        let result = poly.evaluate(&to_field(vec![0, 1]));
        assert!(result.into_bigint().to_bytes_be().len() > 0);
    }

    #[test]
    fn test_partial_eval() {
        let poly = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        let new_poly = poly.partial_eval(0, &Fr::from(1));
        assert_eq!(new_poly.coefficients.len(), 2);
    }

    #[test]
    fn test_generate_pairs() {
        let pairs = MultiPoly::<Fr>::generate_pairs(0, 2);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (0, 2));
    }

    #[test]
    fn test_insert_zero_bit() {
        assert_eq!(MultiPoly::<Fr>::insert_zero_bit(3, 1), 5);
        assert_eq!(MultiPoly::<Fr>::insert_zero_bit(3, 2), 3);
    }

    #[test]
    fn test_evaluation_with_sample_polynomials() {
        // Polynomial 1: f(a, b) = 2 + 4a + 6b + 8ab (multilinear)
        let poly1 = MultiPoly::new(2, to_field(vec![2, 4, 6, 8]));
        let eval1 = poly1.evaluate(&to_field(vec![1, 0]));
        assert_eq!(eval1, Fr::from(4));

        // Polynomial 2: f(a, b, c) = 1 + 3a + 5b + 7ab + 9c + 11ac + 13bc + 15abc (multilinear)
        let poly2 = MultiPoly::new(3, to_field(vec![1, 3, 5, 7, 9, 11, 13, 15]));
        let eval2 = poly2.evaluate(&to_field(vec![1, 1, 0]));
        assert_eq!(eval2, Fr::from(7));
    }
}
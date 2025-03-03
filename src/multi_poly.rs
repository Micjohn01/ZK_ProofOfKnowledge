use ark_ff::{BigInteger, PrimeField};
use crate::new_circuit;

#[derive(Clone, Debug, PartialEq)]
pub struct MultiPoly<F: PrimeField> {
    pub evaluated_values: Vec<F>,
    pub num_vars: usize,
}

impl<F: PrimeField> MultiPoly<F> {
    pub fn new(num_vars: usize, evaluated_values: Vec<F>) -> Self {
        assert_eq!(
            evaluated_values.len(),
            1 << num_vars,
            "Invalid number of evaluated values"
        );
        Self {
            evaluated_values,
            num_vars,
        }
    }

    pub fn evaluate(&self, assignment: &[F]) -> F {
        assert_eq!(
            assignment.len(),
            self.num_vars,
            "Incorrect number of assignments"
        );
        let mut poly = self.evaluated_values.clone();
        for (i, &val) in assignment.iter().enumerate() {
            poly = Self::partial_evaluate(&poly, i, val).evaluated_values;
        }
        poly[0]
    }

    pub fn partial_evaluate(poly: &[F], index: usize, value: F) -> Self {
        let num_vars = poly.len().ilog2() as usize;
        let mut result = Vec::with_capacity(1 << (num_vars - 1));
        for (a, b) in Self::generate_pairs(index, num_vars) {
            let a_val = poly[a];
            let b_val = poly[b];
            result.push(a_val + value * (b_val - a_val));
        }
        Self::new(num_vars - 1, result)
    }

    fn generate_pairs(index: usize, num_vars: usize) -> Vec<(usize, usize)> {
        let target_size = 1 << (num_vars - 1);
        let mut result = Vec::with_capacity(target_size);
        let inverted_index = num_vars - 1 - index;
        let bit_mask = 1 << inverted_index;
        for val in 0..target_size {
            let zero = Self::insert_zero_bit(val, inverted_index);
            let one = zero | bit_mask;
            result.push((zero, one));
        }
        result
    }

    fn insert_zero_bit(value: usize, index: usize) -> usize {
        let high = value >> index;
        let mask = (1 << index) - 1;
        let low = value & mask;
        (high << (index + 1)) | low
    }

    pub fn scalar_mul(&self, scalar: F) -> Self {
        let scaled = self.evaluated_values.iter().map(|&val| scalar * val).collect();
        Self::new(self.num_vars, scaled)
    }

    pub fn number_of_variables(&self) -> usize {
        self.num_vars
    }

    pub fn add_polynomials(first_poly: &Self, second_poly: &Self) -> Self {
        assert_eq!(
            first_poly.evaluated_values.len(),
            second_poly.evaluated_values.len(),
            "Polynomials must have same number of evaluations"
        );
        let sum = first_poly
            .evaluated_values
            .iter()
            .zip(&second_poly.evaluated_values)
            .map(|(&a, &b)| a + b)
            .collect();
        Self::new(first_poly.num_vars, sum)
    }

    pub fn tensor_add_poly(&self, other: &Self) -> Self {
        let new_num_vars = self.num_vars + other.num_vars;
        let mut result = Vec::with_capacity(1 << new_num_vars);
        for &w_b in &self.evaluated_values {
            for &w_c in &other.evaluated_values {
                result.push(w_b + w_c);
            }
        }
        Self::new(new_num_vars, result)
    }

    pub fn tensor_mul_poly(&self, other: &Self) -> Self {
        let new_num_vars = self.num_vars + other.num_vars;
        let mut result = Vec::with_capacity(1 << new_num_vars);
        for &w_b in &self.evaluated_values {
            for &w_c in &other.evaluated_values {
                result.push(w_b * w_c);
            }
        }
        Self::new(new_num_vars, result)
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for value in &self.evaluated_values {
            bytes.extend(value.into_bigint().to_bytes_be());
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;

    fn to_field(values: Vec<u64>) -> Vec<Fr> {
        values.into_iter().map(Fr::from).collect()
    }

    #[test]
    fn test_polynomial_construction() {
        let poly = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        assert_eq!(poly.evaluated_values.len(), 4);
        assert_eq!(poly.num_vars, 2);
    }

    #[test]
    fn test_evaluate() {
        let poly = MultiPoly::new(2, to_field(vec![0, 0, 3, 8]));
        let result = poly.evaluate(&to_field(vec![6, 2]));
        assert_eq!(result, Fr::from(78));
    }

    #[test]
    fn test_partial_eval() {
        let poly = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        let new_poly = MultiPoly::partial_evaluate(&poly.evaluated_values, 0, Fr::from(1));
        assert_eq!(new_poly.evaluated_values.len(), 2);
        assert_eq!(new_poly.num_vars, 1);
    }

    #[test]
    fn test_generate_pairs() {
        let pairs = MultiPoly::<Fr>::generate_pairs(0, 2);
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (0, 2));
        assert_eq!(pairs[1], (1, 3));
    }

    #[test]
    fn test_insert_zero_bit() {
        assert_eq!(MultiPoly::<Fr>::insert_zero_bit(3, 1), 5);
        assert_eq!(MultiPoly::<Fr>::insert_zero_bit(3, 2), 3);
    }

    #[test]
    fn test_evaluation_with_sample_polynomials() {
        let poly1 = MultiPoly::new(2, to_field(vec![2, 4, 6, 8]));
        let eval1 = poly1.evaluate(&to_field(vec![1, 0]));
        assert_eq!(eval1, Fr::from(4));

        let poly2 = MultiPoly::new(3, to_field(vec![1, 3, 5, 7, 9, 11, 13, 15]));
        let eval2 = poly2.evaluate(&to_field(vec![1, 1, 0]));
        assert_eq!(eval2, Fr::from(7));
    }

    #[test]
    fn test_scalar_mul() {
        let poly = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        let scaled = poly.scalar_mul(Fr::from(2));
        assert_eq!(scaled.evaluated_values, to_field(vec![2, 4, 6, 8]));
        assert_eq!(scaled.num_vars, 2);
    }

    #[test]
    fn test_number_of_variables() {
        let poly = MultiPoly::new(3, to_field(vec![1, 2, 3, 4, 5, 6, 7, 8]));
        assert_eq!(poly.number_of_variables(), 3);
    }

    #[test]
    fn test_add_polynomials() {
        let poly1 = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        let poly2 = MultiPoly::new(2, to_field(vec![2, 3, 4, 5]));
        let sum = MultiPoly::add_polynomials(&poly1, &poly2);
        assert_eq!(sum.evaluated_values, to_field(vec![3, 5, 7, 9]));
        assert_eq!(sum.num_vars, 2);
    }

    #[test]
    fn test_tensor_add_poly() {
        let poly1 = MultiPoly::new(1, to_field(vec![1, 2]));
        let poly2 = MultiPoly::new(1, to_field(vec![3, 4]));
        let result = poly1.tensor_add_poly(&poly2);
        assert_eq!(result.evaluated_values, to_field(vec![4, 5, 5, 6]));
        assert_eq!(result.num_vars, 2);
    }

    #[test]
    fn test_tensor_mul_poly() {
        let poly1 = MultiPoly::new(1, to_field(vec![2, 3]));
        let poly2 = MultiPoly::new(1, to_field(vec![4, 5]));
        let result = poly1.tensor_mul_poly(&poly2);
        assert_eq!(result.evaluated_values, to_field(vec![8, 10, 12, 15]));
        assert_eq!(result.num_vars, 2);
    }

    #[test]
    fn test_convert_to_bytes() {
        let poly = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        let bytes = poly.convert_to_bytes();
        assert_eq!(bytes.len(), 128);
        assert_eq!(bytes[31], 1);
        assert_eq!(bytes[63], 2);
    }

    #[test]
    #[should_panic(expected = "Polynomials must have same number of evaluations")]
    fn test_add_polynomials_different_lengths() {
        let poly1 = MultiPoly::new(2, to_field(vec![1, 2, 3, 4]));
        let poly2 = MultiPoly::new(1, to_field(vec![1, 2]));
        MultiPoly::add_polynomials(&poly1, &poly2);
    }
}
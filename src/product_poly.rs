use ark_ff::{BigInteger, PrimeField};
use crate::multi_poly::MultiPoly;

#[derive(Clone, Debug, PartialEq)]
pub struct ProductPolynomial<F: PrimeField> {
    pub polynomials: Vec<MultiPoly<F>>,
}

impl<F: PrimeField> ProductPolynomial<F> {
    pub fn new(polynomials: Vec<MultiPoly<F>>) -> Self {
        let num_vars = polynomials[0].number_of_variables();
        assert!(
            polynomials.iter().all(|poly| poly.number_of_variables() == num_vars),
            "All polynomials must have the same number of variables"
        );
        Self { polynomials }
    }

    pub fn evaluate(&self, values: &[F]) -> F {
        self.polynomials.iter().fold(F::one(), |acc, poly| acc * poly.evaluate(values))
    }

    pub fn partial_evaluate(&self, evaluating_variable: usize, value: F) -> Self {
        let evaluated_polynomials = self.polynomials.iter()
            .map(|poly| MultiPoly::partial_evaluate(&poly.evaluated_values, evaluating_variable, value))
            .collect();
        Self { polynomials: evaluated_polynomials }
    }

    pub fn multiply_polys_element_wise(&self) -> MultiPoly<F> {
        assert!(self.polynomials.len() > 1, "More than one polynomial required for multiplication");
        let mut result_values = self.polynomials[0].evaluated_values.clone();
        for poly in self.polynomials.iter().skip(1) {
            for (i, &val) in poly.evaluated_values.iter().enumerate() {
                result_values[i] *= val;
            }
        }
        MultiPoly::new(self.polynomials[0].num_vars, result_values)
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for poly in &self.polynomials {
            bytes.extend(poly.evaluated_values.iter().flat_map(|v| v.into_bigint().to_bytes_be()));
        }
        bytes
    }

    pub fn degree(&self) -> usize {
        self.polynomials.len()
    }
}


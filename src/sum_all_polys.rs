use ark_ff::PrimeField;
use crate::product_poly::ProductPolynomial;
use crate::multi_poly::MultiPoly;


#[derive(Debug, PartialEq, Clone)]
pub struct SumPolynomial<F: PrimeField> {
    product_polynomials: Vec<ProductPolynomial<F>>,
}

// Trait for polynomial arithmetic operations
pub trait PolynomialArithmetic<F: PrimeField> {
    fn evaluate(&self, values: &[F]) -> F;
    fn partial_evaluate(&self, var_idx: usize, value: F) -> Self;
    fn element_wise_polynomials_addition(&self) -> MultiPoly<F>;
}

impl<F: PrimeField> PolynomialArithmetic<F> for SumPolynomial<F> {
    fn evaluate(&self, values: &[F]) -> F {
        self.product_polynomials.iter()
            .fold(F::zero(), |acc, prod_poly| acc + prod_poly.evaluate(values))
    }

    fn partial_evaluate(&self, var_idx: usize, value: F) -> Self {
        let evaluated_products = self.product_polynomials.iter()
            .map(|prod_poly| prod_poly.partial_evaluate(var_idx, value))
            .collect();
        SumPolynomial { product_polynomials: evaluated_products }
    }

    fn element_wise_polynomials_addition(&self) -> MultiPoly<F> {
        assert!(self.product_polynomials.len() > 1);
        let num_vars = self.product_polynomials[0].polynomials[0].num_vars;
        let len = self.product_polynomials[0].polynomials[0].evaluated_values.len();
        let mut result = vec![F::zero(); len];
        self.product_polynomials.iter().for_each(|prod_poly| {
            let prod = prod_poly.multiply_polys_element_wise();
            result.iter_mut().zip(&prod.evaluated_values).for_each(|(r, &v)| *r += v);
        });
        MultiPoly::new(num_vars, result)
    }
}

// Trait for serialization
pub trait PolynomialSerialization<F: PrimeField> {
    fn convert_to_bytes(&self) -> Vec<u8>;
}

impl<F: PrimeField> PolynomialSerialization<F> for SumPolynomial<F> {
    fn convert_to_bytes(&self) -> Vec<u8> {
        let total_len = self.product_polynomials.iter()
            .map(|p| p.polynomials.iter().map(|poly| poly.evaluated_values.len() * 32).sum::<usize>())
            .sum();
        let mut bytes = Vec::with_capacity(total_len);
        self.product_polynomials.iter().for_each(|prod_poly| {
            bytes.extend(prod_poly.convert_to_bytes());
        });
        bytes
    }
}

// Trait for metadata access
pub trait PolynomialMetadata {
    fn degree(&self) -> usize;
    fn number_of_variables(&self) -> usize;
}

impl<F: PrimeField> PolynomialMetadata for SumPolynomial<F> {
    fn degree(&self) -> usize {
        self.product_polynomials[0].degree()
    }

    fn number_of_variables(&self) -> usize {
        self.product_polynomials[0].polynomials[0].number_of_variables()
    }
}

// Construction method as a standalone function or in impl block
impl<F: PrimeField> SumPolynomial<F> {
    pub fn new(product_polynomials: Vec<ProductPolynomial<F>>) -> Self {
        let num_vars = product_polynomials[0].polynomials[0].number_of_variables();
        assert!(product_polynomials.iter().all(|p| p.polynomials.iter().all(|poly| poly.number_of_variables() == num_vars)));
        Self { product_polynomials }
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
    #[should_panic(expected = "different number of variables")]
    fn test_new_with_different_polynomial_lengths() {
        let poly1 = MultiPoly::new(1, to_field(vec![0, 2]));
        let poly2 = MultiPoly::new(2, to_field(vec![0, 0, 0, 3]));
        let product_poly1 = ProductPolynomial::new(vec![poly1]);
        let product_poly2 = ProductPolynomial::new(vec![poly2]);
        SumPolynomial::new(vec![product_poly1, product_poly2]);
    }

    #[test]
    fn test_evaluate_sum_poly() {
        let polynomial1a = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let polynomial1b = MultiPoly::new(2, to_field(vec![0, 0, 0, 3]));
        let product_poly1 = ProductPolynomial::new(vec![polynomial1a, polynomial1b]);
        let poly2a = MultiPoly::new(2, to_field(vec![0, 0, 0, 1]));
        let poly2b = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let product_poly2 = ProductPolynomial::new(vec![poly2a, poly2b]);
        let sum_poly = SumPolynomial::new(vec![product_poly1, product_poly2]);
        let values = to_field(vec![1, 2]);
        assert_eq!(sum_poly.evaluate(&values), Fr::from(32));
    }

    #[test]
    fn test_partial_evaluate_sum_poly() {
        let polynomial1a = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let polynomial1b = MultiPoly::new(2, to_field(vec![0, 0, 0, 3]));
        let product_poly1 = ProductPolynomial::new(vec![polynomial1a, polynomial1b]);
        let poly2a = MultiPoly::new(2, to_field(vec![0, 0, 0, 1]));
        let poly2b = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let product_poly2 = ProductPolynomial::new(vec![poly2a, poly2b]);
        let sum_poly = SumPolynomial::new(vec![product_poly1, product_poly2]);
        let result = sum_poly.partial_evaluate(0, Fr::from(2));
        let expect_poly1a = MultiPoly::new(1, to_field(vec![0, 4]));
        let expect_poly1b = MultiPoly::new(1, to_field(vec![0, 6]));
        let expect_product1 = ProductPolynomial::new(vec![expect_poly1a, expect_poly1b]);
        let expect_poly2a = MultiPoly::new(1, to_field(vec![0, 2]));
        let expect_poly2b = MultiPoly::new(1, to_field(vec![0, 4]));
        let expect_product2 = ProductPolynomial::new(vec![expect_poly2a, expect_poly2b]);
        let expected_sum = SumPolynomial::new(vec![expect_product1, expect_product2]);
        assert_eq!(result.product_polynomials, expected_sum.product_polynomials);
    }

    #[test]
    fn test_add_polynomials_element_wise() {
        let polynomial1a = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let polynomial1b = MultiPoly::new(2, to_field(vec![0, 0, 0, 3]));
        let product_poly1 = ProductPolynomial::new(vec![polynomial1a, polynomial1b]);
        let poly2a = MultiPoly::new(2, to_field(vec![0, 0, 0, 1]));
        let poly2b = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let product_poly2 = ProductPolynomial::new(vec![poly2a, poly2b]);
        let sum_poly = SumPolynomial::new(vec![product_poly1, product_poly2]);
        let result = sum_poly.element_wise_polynomials_addition();
        let expected = MultiPoly::new(2, to_field(vec![0, 0, 0, 8]));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_degree_sum_poly() {
        let polynomial1a = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let polynomial1b = MultiPoly::new(2, to_field(vec![0, 0, 0, 3]));
        let product_poly1 = ProductPolynomial::new(vec![polynomial1a, polynomial1b]);
        let poly2a = MultiPoly::new(2, to_field(vec![0, 0, 0, 1]));
        let poly2b = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let product_poly2 = ProductPolynomial::new(vec![poly2a, poly2b]);
        let sum_poly = SumPolynomial::new(vec![product_poly1, product_poly2]);
        assert_eq!(sum_poly.degree(), 2);
    }

    #[test]
    fn test_number_of_variables() {
        let polynomial1a = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let polynomial1b = MultiPoly::new(2, to_field(vec![0, 0, 0, 3]));
        let product_poly1 = ProductPolynomial::new(vec![polynomial1a, polynomial1b]);
        let poly2a = MultiPoly::new(2, to_field(vec![0, 0, 0, 1]));
        let poly2b = MultiPoly::new(2, to_field(vec![0, 0, 0, 2]));
        let product_poly2 = ProductPolynomial::new(vec![poly2a, poly2b]);
        let sum_poly = SumPolynomial::new(vec![product_poly1, product_poly2]);
        assert_eq!(sum_poly.number_of_variables(), 2);
    }

    
}

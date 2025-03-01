use ark_ff::PrimeField;

#[derive(Clone, Debug)]
struct SparseUnivariatePolynomial<F: PrimeField> {
    terms: Vec<(usize, F)>,
}

impl<F: PrimeField> SparseUnivariatePolynomial<F> {
    fn new(terms: Vec<(usize, F)>) -> Self {
        let mut sorted_terms = terms;
        sorted_terms.sort_by(|a, b| a.0.cmp(&b.0));
        let mut combined: Vec<(usize, F)> = Vec::new();
        for (power, coeff) in sorted_terms {
            if coeff.is_zero() { continue; }
            match combined.last_mut() {
                Some(&mut (last_pow, ref mut last_coeff)) if last_pow == power => *last_coeff += coeff,
                _ => combined.push((power, coeff)),
            }
        }
        SparseUnivariatePolynomial { terms: combined }
    }

    fn degree(&self) -> Option<usize> {
        self.terms.last().map(|(power, _)| *power)
    }

    fn evaluate(&self, x: F) -> F {
        self.terms.iter()
            .fold(F::zero(), |acc, (power, coeff)| {
                acc + *coeff * x.pow([*power as u64])
            })
    }

    fn lagrange_interpolation(points: &[(F, F)]) -> Self {
        let mut result = Self::new(vec![(0, F::zero())]);
        
        for (i, &(xi, yi)) in points.iter().enumerate() {
            let mut basis = vec![(0, F::one())];
            
            for (j, &(xj, _)) in points.iter().enumerate() {
                if i != j {
                    let denominator = xi - xj;
                    let term = vec![(0, -xj / denominator), (1, F::one() / denominator)];
                    basis = Self::multiply(&basis, &term);
                }
            }
            
            result = Self::add(&result, &Self::new(Self::scalar_mult(yi, &basis)));
        }
        
        result
    }

    fn multiply(a: &[(usize, F)], b: &[(usize, F)]) -> Vec<(usize, F)> {
        let mut result = Vec::new();
        for (pow_a, coeff_a) in a {
            for (pow_b, coeff_b) in b {
                let new_pow = pow_a + pow_b;
                let new_coeff = *coeff_a * *coeff_b;
                if !new_coeff.is_zero() {
                    result.push((new_pow, new_coeff));
                }
            }
        }
        Self::new(result).terms
    }

    fn scalar_mult(scalar: F, terms: &[(usize, F)]) -> Vec<(usize, F)> {
        terms.iter()
            .map(|(pow, coeff)| (*pow, scalar * coeff))
            .filter(|(_, coeff)| !coeff.is_zero())
            .collect()
    }

    fn add(a: &Self, b: &Self) -> Self {
        let mut combined = a.terms.clone();
        combined.extend(b.terms.iter().cloned());
        Self::new(combined)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    // Helper function to create a test polynomial: 5 + 2x^2
    fn test_poly() -> SparseUnivariatePolynomial<Fq> {
        SparseUnivariatePolynomial::new(vec![(0, Fq::from(5)), (2, Fq::from(2))])
    }

    #[test]
    fn test_creation_and_degree() {
        let poly = test_poly();
        assert_eq!(poly.degree(), Some(2));
        assert_eq!(poly.terms.len(), 2);
        assert_eq!(poly.terms, vec![(0, Fq::from(5)), (2, Fq::from(2))]);

        // Test zero coefficient removal and duplicate power combination
        let poly_with_zero = SparseUnivariatePolynomial::new(vec![
            (0, Fq::from(1)),
            (1, Fq::from(0)),  // Should be removed
            (2, Fq::from(2)),
            (2, Fq::from(3)),  // Should combine with previous 2
        ]);
        assert_eq!(poly_with_zero.terms.len(), 2);
        assert_eq!(poly_with_zero.terms, vec![(0, Fq::from(1)), (2, Fq::from(5))]);

        // Test zero polynomial
        let zero_poly = SparseUnivariatePolynomial::new(vec![(0, Fq::from(0))]);
        assert_eq!(zero_poly.degree(), None);
        assert_eq!(zero_poly.terms.len(), 0);
    }

    #[test]
    fn test_evaluation() {
        let poly = test_poly();
        let x = Fq::from(2);
        // 5 + 2*(2^2) = 5 + 2*4 = 13
        assert_eq!(poly.evaluate(x), Fq::from(13));
        
        let x = Fq::from(0);
        // 5 + 2*(0^2) = 5
        assert_eq!(poly.evaluate(x), Fq::from(5));

        // Test zero polynomial
        let zero = SparseUnivariatePolynomial::new(vec![]);
        assert_eq!(zero.evaluate(x), Fq::from(0));
    }

    #[test]
    fn test_multiply() {
        let p1 = vec![(0, Fq::from(5)), (2, Fq::from(2))];    // 5 + 2x^2
        let p2 = vec![(0, Fq::from(6)), (1, Fq::from(2))];    // 6 + 2x
        let result = SparseUnivariatePolynomial::multiply(&p1, &p2);
        
        // Expected: 5*(6 + 2x) + 2x^2*(6 + 2x) = (30 + 10x) + (12x^2 + 4x^3)
        //         = 30 + 10x + 12x^2 + 4x^3
        let expected = vec![
            (0, Fq::from(30)),
            (1, Fq::from(10)),
            (2, Fq::from(12)),
            (3, Fq::from(4))
        ];
        assert_eq!(result, expected);

        // Test multiplication with zero
        let zero = vec![];
        let result = SparseUnivariatePolynomial::multiply(&p1, &zero);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_add() {
        let p1 = SparseUnivariatePolynomial::new(vec![(0, Fq::from(5)), (2, Fq::from(2))]); // 5 + 2x^2
        let p2 = SparseUnivariatePolynomial::new(vec![(1, Fq::from(3)), (2, Fq::from(1))]); // 3x + x^2
        let sum = SparseUnivariatePolynomial::add(&p1, &p2);
        
        // Expected: 5 + 3x + (2+1)x^2 = 5 + 3x + 3x^2
        let expected = vec![
            (0, Fq::from(5)),
            (1, Fq::from(3)),
            (2, Fq::from(3))
        ];
        assert_eq!(sum.terms, expected);

        // Test adding zero polynomial
        let zero = SparseUnivariatePolynomial::new(vec![]);
        let sum_with_zero = SparseUnivariatePolynomial::add(&p1, &zero);
        assert_eq!(sum_with_zero.terms, p1.terms);
    }

    #[test]
    fn test_scalar_mult() {
        let terms = vec![(0, Fq::from(5)), (2, Fq::from(2))]; // 5 + 2x^2
        let scalar = Fq::from(3);
        let result = SparseUnivariatePolynomial::scalar_mult(scalar, &terms);
        
        // Expected: 3*(5 + 2x^2) = 15 + 6x^2
        let expected = vec![
            (0, Fq::from(15)),
            (2, Fq::from(6))
        ];
        assert_eq!(result, expected);

        // Test with zero scalar
        let zero_result = SparseUnivariatePolynomial::scalar_mult(Fq::from(0), &terms);
        assert_eq!(zero_result, vec![]);
    }

    #[test]
    fn test_lagrange_interpolation() {
        let points = vec![
            (Fq::from(0), Fq::from(2)),
            (Fq::from(1), Fq::from(4)),
            (Fq::from(2), Fq::from(10))
        ];
        
        let poly = SparseUnivariatePolynomial::lagrange_interpolation(&points);
        
        // Verify interpolation points
        for (x, y) in &points {
            assert_eq!(poly.evaluate(*x), *y, "Failed at x = {:?}, expected y = {:?}", x, y);
        }

        // Expected polynomial: 2 + 2x^2 (verified from dense code and manual calculation)
        let expected_terms = vec![
            (0, Fq::from(2)),
            (2, Fq::from(2))
        ];
        assert_eq!(poly.terms, expected_terms);

        // Test single point
        let single_point = vec![(Fq::from(1), Fq::from(5))];
        let single_poly = SparseUnivariatePolynomial::lagrange_interpolation(&single_point);
        assert_eq!(single_poly.evaluate(Fq::from(1)), Fq::from(5));
        assert_eq!(single_poly.terms, vec![(0, Fq::from(5))]); // Constant polynomial
    }
}
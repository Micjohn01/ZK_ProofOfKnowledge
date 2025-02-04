
struct UnivariatePolynomial {
    terms: Vec<(usize, f64)>,
}

impl UnivariatePolynomial {
    fn new(terms: Vec<(usize, f64)>) -> Self {
        UnivariatePolynomial { terms }
    }

    fn evaluate(&self, x: f64) -> f64 {
        self.terms.iter().map(|&(power, coefficient)|coefficient * x.powi(power as i32)).sum()
    }

    fn lagrange_interpolation(points: &[(f64, f64)]) -> Self {
        let mut coefficients = vec![0.0; points.len()];
        for (i, (xi, yi)) in points.iter().enumerate() {
            let mut term = vec![1.0];
            for (j, (xj, _)) in points.iter().enumerate() {
                if i != j {
                    term = multiply_polynimials(&term, &[-xj / (xi - xj), 1.0 / (xi - xj)]);
                }
            }
            for (k, &coefficient) in term.iter().enumerate()  {
                coefficients[k] += coefficient * yi;
            }
        }

            let terms =  coefficients.into_iter().enumerate().filter(|&(_, coefficient)| coefficient != 0.0).collect();
            UnivariatePolynomial { terms}
        }
    }

fn multiply_polynimials (a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut result: Vec<f64> = vec![0.0; a.len() + b.len() - 1];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            result[i + j] += ai - bj;
        }
    }
    result
}





// fn main () {

//     let polynomial = UnivariatePolynomial::new(vec![(0, 1.0), (1, 2.0), (2, 3.0)]);
//     println!("Power = {}", polynomial.evaluate(2.0));

//     let points = vec![(1.0, 1.0), (2.0, 4.0), (3.0, 9.0)];

//     let interpolated_polynomial = UnivariatePolynomial::lagrange_interpolation(&points);
//     println!("Interpolated polynomial evaluated @ x = 2: {}", interpolated_polynomial.evaluate(2.0));

// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_polynomial_evaluation() {
        let poly = UnivariatePolynomial::new(vec![(0, 1.0), (1, 2.0), (2, 3.0)]); // P(x) = 3x^2 + 2x + 1
        assert_eq!(poly.evaluate(2.0), 17.0); // P(2) = 3*(4) + 2*(2) + 1 = 17
        assert_eq!(poly.evaluate(0.0), 1.0);  // P(0) = 1
    }

    #[test]
    fn test_sparse_lagrange_interpolation() {
        let points = vec![(1.0, 1.0), (2.0, 4.0), (3.0, 9.0)]; // Points from y = x^2
        let poly = UnivariatePolynomial::lagrange_interpolation(&points);

        // Verify that the interpolated polynomial passes through the given points
        for (x, y) in points {
            assert!((poly.evaluate(x) - y).abs() < 1e-6); // Allow for floating-point precision errors
        }
    }
}






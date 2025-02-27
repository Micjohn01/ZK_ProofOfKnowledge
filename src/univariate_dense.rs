struct Polynomial {
    coefficients: Vec<f64>,
}

impl Polynomial {
    fn new(coefficients: Vec<f64>) -> Self {
        Polynomial { coefficients }
    }

    fn evaluate(&self, x: f64) -> f64 {
        self.coefficients
            .iter()
            .rev()
            .fold(0.0, |acc, &coeff| acc * x + coeff)
    }

    fn lagrange_interpolation(points: &[(f64, f64)]) -> Self {
        let mut coefficients: Vec<f64> = vec![0.0; points.len()];
        for (i, (xi, yi)) in points.iter().enumerate() {
            let mut term: Vec<f64> = vec![1.0];
            for (j, (xj, _)) in points.iter().enumerate() {
                if i != j {
                    term = multiply_polynomials(&term, &[-xj / (xi - xj), 1.0 / (xi - xj)]);
                }
            }
            for (k, &coeff) in term.iter().enumerate() {
                coefficients[k] += coeff * yi;
            }
        }
        Polynomial { coefficients }
    }
}

fn multiply_polynomials(a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; a.len() + b.len() - 1];
    for (i, &ai) in a.iter().enumerate() {
        for (j, &bj) in b.iter().enumerate() {
            result[i + j] += ai * bj;
        }
    }
    result
}

// fn main() {
//     // Example: Represent P(x) = 3x^2 + 2x + 1
//     let poly = Polynomial::new(vec![1.0, 2.0, 3.0]);
//     println!("P(2) = {}", poly.evaluate(2.0)); // Should print 17.0

//     // Example: Interpolate points (1, 1), (2, 4), (3, 9)
//     let points = vec![(1.0, 1.0), (2.0, 4.0), (3.0, 9.0)];
//     let interpolated_poly = Polynomial::lagrange_interpolation(&points);
//     println!(
//         "Interpolated polynomial evaluated at x = 2: {}",
//         interpolated_poly.evaluate(2.0)
//     ); // Should print 4.0
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polynomial_evaluation() {
        let poly = Polynomial::new(vec![1.0, 2.0, 3.0]); // P(x) = 3x^2 + 2x + 1
        assert_eq!(poly.evaluate(2.0), 17.0); // P(2) = 3*(4) + 2*(2) + 1 = 17
        assert_eq!(poly.evaluate(0.0), 1.0);  // P(0) = 1
    }

    #[test]
    fn test_lagrange_interpolation() {
        let points = vec![(1.0, 1.0), (2.0, 4.0), (3.0, 9.0)]; // Points from y = x^2
        let poly = Polynomial::lagrange_interpolation(&points);

        // Verify that the interpolated polynomial passes through the given points
        for (x, y) in points {
            assert!((poly.evaluate(x) - y).abs() < 1e-6); // Allow for floating-point precision errors
        }
    }

    // #[test]
    // fn test_multiply_polynomials() {
    //     let a = vec![1.0, 2.0]; // P(x) = 2x + 1
    //     let b = vec![3.0, 4.0]; // Q(x) = 4x + 3
    //     let result = multiply_polynomials(&a, &b); // Expected: 8x^2 + 10x + 3
    //     assert_eq!(result, vec![3.0, 10.0, 8.0]);
    // }
}
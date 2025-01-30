use std::collections::HashMap;

struct MultilinearPolynomial {
    coefficients: HashMap<Vec<usize>, f64>,
}

impl MultilinearPolynomial {
    fn new(coefficients: HashMap<Vec<usize>, f64>) -> Self {
        MultilinearPolynomial {coefficients}
    }


fn evaluate(&self, point: &[u8]) -> f64 {
    let mut result: f64 = 0.0;
    for ( vars, &coeff) in &self.coefficients {
        let mut term_value = coeff;
        for &var in vars {
            if point[var] == 0 {
                term_value = 0.0;
                break;
            }
        }
        result += term_value;
    }
    result
}
}

/// Function to generate the Boolean hypercube for `n` variables
fn generate_hypercube(n: usize) -> Vec<Vec<u8>> {
    if n == 0 {
        return vec![vec![]];
    }
    let smaller_hypercube: Vec<Vec<u8>> = generate_hypercube(n - 1);
    let mut hypercube: Vec<Vec<u8>> = Vec::new();
    for mut point in smaller_hypercube {
        let mut point_with_zero: Vec<u8> = point.clone();
        point_with_zero.push(0);
        hypercube.push(point_with_zero);
        point.push(1);
        hypercube.push(point);
    }
    hypercube
}



fn main() {
    // Define the polynomial: 2ab + 3bc + 2da
    let mut coefficients: HashMap<Vec<usize>, f64> = HashMap::new();
    coefficients.insert(vec![0, 1], 2.0); // 2ab (variables a and b)
    coefficients.insert(vec![1, 2], 3.0); // 3bc (variables b and c)
    coefficients.insert(vec![3, 0], 2.0); // 2da (variables d and a)

    let polynomial: MultilinearPolynomial = MultilinearPolynomial::new(coefficients);

    // Generate the Boolean hypercube for 4 variables (a, b, c, d)
    let n = 4;
    let hypercube: Vec<Vec<u8>> = generate_hypercube(n);

    println!("Evaluating polynomial 2ab + 3bc + 2da over the Boolean hypercube:");
    for point in hypercube {
        let value: f64 = polynomial.evaluate(&point);
        println!("Point {:?} -> {}", point, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Boolean hypercube generation
    #[test]
    fn test_generate_hypercube() {
        let n = 2;
        let hypercube = generate_hypercube(n);
        let expected = vec![
            vec![0, 0],
            vec![0, 1],
            vec![1, 0],
            vec![1, 1],
        ];
        assert_eq!(hypercube, expected);
    }

    /// Test polynomial evaluation
    #[test]
    fn test_polynomial_evaluation() {
        let mut coefficients = HashMap::new();
        coefficients.insert(vec![0, 1], 2.0); // 2ab
        coefficients.insert(vec![1, 2], 3.0); // 3bc
        coefficients.insert(vec![3, 0], 2.0); // 2da

        let polynomial = MultilinearPolynomial::new(coefficients);

        // Test evaluation at specific points
        assert_eq!(polynomial.evaluate(&[0, 0, 0, 0]), 0.0); // All variables are 0
        assert_eq!(polynomial.evaluate(&[0, 1, 1, 0]), 3.0); // b=1, c=1
        assert_eq!(polynomial.evaluate(&[1, 0, 0, 1]), 2.0); // a=1, d=1
        assert_eq!(polynomial.evaluate(&[1, 1, 0, 0]), 2.0); // a=1, b=1
        assert_eq!(polynomial.evaluate(&[1, 1, 1, 1]), 7.0); // All variables are 1
    }
}
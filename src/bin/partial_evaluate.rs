use std::collections::HashMap;

/// Represents a multilinear polynomial
struct MultilinearPolynomial {
    coefficients: HashMap<Vec<usize>, f64>, // Keys: variable indices, Values: coefficients
    num_vars: usize, // Track the number of variables dynamically
}

impl MultilinearPolynomial {
    /// Create a new polynomial and infer the number of variables
    fn new(coefficients: HashMap<Vec<usize>, f64>) -> Self {
        // Determine the number of variables from the maximum index in the terms
        let num_vars = coefficients
            .keys()
            .flat_map(|vars| vars.iter().max())
            .max()
            .map(|&m| m + 1) // Convert from 0-based index to count
            .unwrap_or(0); // Handle empty polynomial

        MultilinearPolynomial { coefficients, num_vars }
    }

    /// Evaluate the polynomial at a point on the Boolean hypercube
    fn evaluate(&self, point: &[u8]) -> f64 {
        let mut result = 0.0;
        for (vars, coeff) in &self.coefficients {
            let mut term_active = true;
            for &var in vars {
                if point.get(var).copied().unwrap_or(0) == 0 {
                    term_active = false;
                    break;
                }
            }
            if term_active {
                result += coeff;
            }
        }
        result
    }

    /// Generate the Boolean hypercube for `n` variables
    fn generate_hypercube(&self) -> Vec<Vec<u8>> {
        let n = self.num_vars;
        if n == 0 {
            return vec![vec![]];
        }
        let smaller = self.generate_hypercube(n - 1);
        let mut hypercube = Vec::new();
        for mut point in smaller {
            let mut point_zero = point.clone();
            point_zero.push(0);
            hypercube.push(point_zero);
            point.push(1);
            hypercube.push(point);
        }
        hypercube
    }

    /// Compute linear interpolation for edges of the hypercube
    fn interpolate_edges(&self) -> Vec<(Vec<u8>, Vec<u8>, f64, f64)> {
        let hypercube = self.generate_hypercube();
        let mut edges = Vec::new();

        // Iterate through all pairs of points differing by one variable
        for i in 0..hypercube.len() {
            for j in (i + 1)..hypercube.len() {
                let diff = hypercube[i]
                    .iter()
                    .zip(hypercube[j].iter())
                    .filter(|(a, b)| a != b)
                    .count();
                if diff == 1 {
                    let y1 = self.evaluate(&hypercube[i]);
                    let y2 = self.evaluate(&hypercube[j]);
                    edges.push((hypercube[i].clone(), hypercube[j].clone(), y1, y2));
                }
            }
        }
        edges
    }
}

fn main() {
    // Example: 4-variable polynomial (2ab + 3bc + 4cd)
    let mut coefficients = HashMap::new();
    coefficients.insert(vec![0, 1], 2.0); // 2ab
    coefficients.insert(vec![1, 2], 3.0); // 3bc
    coefficients.insert(vec![2, 3], 4.0); // 4cd

    let polynomial = MultilinearPolynomial::new(coefficients);

    // Generate all edges and their interpolations
    let edges = polynomial.interpolate_edges();

    // Print interpolations for edges of interest
    for (p1, p2, y1, y2) in edges {
        println!(
            "Edge {:?} → {:?}: Linear form = {} + r({} - {}) = {} + {}r",
            p1, p2, y1, y2, y1, y1, y2 - y1
        );
    }
}

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





fn main () {

    let polynomial = UnivariatePolynomial::new(vec![(0, 1.0), (1, 2.0), (2, 3.0)]);
    println!("Power = {}", polynomial.evaluate(2.0));

    let points = vec![(1.0, 1.0), (2.0, 4.0), (3.0, 9.0)];

    let interpolated_polynomial = UnivariatePolynomial::lagrange_interpolation(&points);
    println!("Interpolated polynomial evaluated @ x = 2: {}", interpolated_polynomial.evaluate(2.0));

}






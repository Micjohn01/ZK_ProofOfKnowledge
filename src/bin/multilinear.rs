use std::collections::HashMap;

struct MultilinearPolynomial {
    coefficients: HashMap<Vec<usize>, f64>,
}

impl MultilinearPolynomial {
    fn new(coefficients: HashMap<Vec<usize>, f64>) -> Self {
        MultilinearPolynomial {coefficients}
    }
}



fn main() {

}
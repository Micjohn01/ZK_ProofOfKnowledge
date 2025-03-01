use ark_ff::PrimeField;

#[derive(Clone, Debug, PartialEq)]
pub struct Polynomial<F: PrimeField> {
    pub coeffs: Vec<F>,
}

impl<F: PrimeField> Polynomial<F> {
    pub fn new(input_coeffs: &Vec<F>) -> Self {
        Self {
            coeffs: input_coeffs.clone(),
        }
    }

    pub fn degree(&self) -> u32 {
        self.coeffs.len() as u32 - 1
    }

    pub fn eval(&self, x: F) -> F {
        self.coeffs
            .iter()
            .enumerate()
            .fold(F::zero(), |acc, (exp, &coeff)| acc + coeff * x.pow([exp as u64]))
    }

    pub fn lagrange_interpolate(x_vals: &Vec<F>, y_vals: &Vec<F>) -> Self {
        assert_eq!(x_vals.len(), y_vals.len(), "x_vals and y_vals must have equal length");
        let mut result = vec![F::zero()];
        for (i, &x_i) in x_vals.iter().enumerate() {
            let basis = Self::basis_poly(&y_vals[i], &x_i, x_vals);
            result = Self::add_polys(result, basis);
        }
        Polynomial { coeffs: result }
    }

    fn basis_poly(y_i: &F, x_i: &F, x_set: &Vec<F>) -> Vec<F> {
        let mut num = vec![F::one()];
        for &x_j in x_set {
            if x_j != *x_i {
                num = Self::mul_polys(num, vec![-x_j, F::one()]);
            }
        }
        let basis = Self::new(&num);
        let denom = basis.eval(*x_i);
        Self::scale(*y_i / denom, num)
    }

    fn scale(scalar: F, poly: Vec<F>) -> Vec<F> {
        poly.into_iter().map(|c| scalar * c).collect()
    }

    fn mul_polys(left: Vec<F>, right: Vec<F>) -> Vec<F> {
        let mut product = vec![F::zero(); left.len() + right.len() - 1];
        for (i, &l_coeff) in left.iter().enumerate() {
            for (j, &r_coeff) in right.iter().enumerate() {
                product[i + j] += l_coeff * r_coeff;
            }
        }
        product
    }

    fn add_polys(mut left: Vec<F>, right: Vec<F>) -> Vec<F> {
        let (longer, shorter) = if left.len() > right.len() {
            (left, right)
        } else {
            (right, left)
        };
        left = longer;
        for (i, &r_coeff) in shorter.iter().enumerate() {
            left[i] += r_coeff;
        }
        left
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    fn setup_poly() -> Polynomial<Fq> {
        let coeffs = vec![
            Fq::from(0),
            Fq::from(0),
            Fq::from(2),
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(3),
        ];
        Polynomial::new(&coeffs)
    }

    #[test]
    fn test_degree() {
        let poly = setup_poly();
        assert_eq!(poly.degree(), 7);

        let zero_poly = Polynomial::new(&vec![Fq::from(0)]);
        assert_eq!(zero_poly.degree(), 0);
    }

    #[test]
    fn test_eval() {
        let poly = setup_poly();
        let x = Fq::from(2);
        assert_eq!(poly.eval(x), Fq::from(392));
    }

    #[test]
    fn test_add_polys() {
        let p1 = vec![Fq::from(5), Fq::from(2), Fq::from(5)];
        let p2 = vec![Fq::from(2), Fq::from(1), Fq::from(8), Fq::from(10)];
        assert_eq!(
            Polynomial::add_polys(p1, p2),
            vec![Fq::from(7), Fq::from(3), Fq::from(13), Fq::from(10)]
        );
    }

    #[test]
    fn test_mul_polys() {
        let p1 = vec![Fq::from(5), Fq::from(0), Fq::from(2)];
        let p2 = vec![Fq::from(6), Fq::from(2)];
        assert_eq!(
            Polynomial::mul_polys(p1, p2),
            vec![Fq::from(30), Fq::from(10), Fq::from(12), Fq::from(4)]
        );
    }

    #[test]
    fn test_lagrange_interpolate() {
        let x_vals = vec![Fq::from(0), Fq::from(1), Fq::from(2)];
        let y_vals = vec![Fq::from(2), Fq::from(4), Fq::from(10)];
        let poly = Polynomial::lagrange_interpolate(&x_vals, &y_vals);
        assert_eq!(poly.coeffs, vec![Fq::from(2), Fq::from(0), Fq::from(2)]);
        for (x, y) in x_vals.iter().zip(y_vals.iter()) {
            assert_eq!(poly.eval(*x), *y, "Failed at x = {:?}", x);
        }
    }

    #[test]
    fn test_scale() {
        let poly = vec![Fq::from(5), Fq::from(0), Fq::from(2)];
        let scalar = Fq::from(3);
        let result = Polynomial::scale(scalar, poly);
        assert_eq!(result, vec![Fq::from(15), Fq::from(0), Fq::from(6)]);
    }
}
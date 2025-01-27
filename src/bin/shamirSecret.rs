use rand::Rng;
use std::collections::HashMap;

// Generate a random polynomial of degree `threshold - 1`
fn generate_polynomial(secret: i64, threshold: usize) -> Vec<i64> {
    let mut rng = rand::thread_rng();
    let mut polynomial = vec![secret];
    for _ in 1..threshold {
        polynomial.push(rng.gen_range(1..100)); // Random coefficients
    }
    polynomial
}

// Evaluate the polynomial at a given x
fn evaluate_polynomial(polynomial: &[i64], x: i64) -> i64 {
    polynomial
        .iter()
        .enumerate()
        .fold(0, |acc, (i, &coeff)| acc + coeff * x.pow(i as u32))
}

// Generate shares from the polynomial
fn generate_shares(secret: i64, threshold: usize, num_shares: usize) -> HashMap<i64, i64> {
    let polynomial = generate_polynomial(secret, threshold);
    let mut shares = HashMap::new();
    for x in 1..=num_shares as i64 {
        let y = evaluate_polynomial(&polynomial, x);
        shares.insert(x, y);
    }
    shares
}

// Reconstruct the secret using Lagrange interpolation
fn reconstruct_secret(shares: &HashMap<i64, i64>) -> i64 {
    let mut secret = 0;
    for (x_j, y_j) in shares {
        let mut numerator = 1;
        let mut denominator = 1;
        for (x_i, _) in shares {
            if x_i != x_j {
                numerator *= -x_i;
                denominator *= x_j - x_i;
            }
        }
        secret += y_j * numerator / denominator;
    }
    secret
}

fn main() {
    let secret = 42;
    let threshold = 3;
    let num_shares = 5;

    // Generate shares
    let shares = generate_shares(secret, threshold, num_shares);
    println!("Shares: {:?}", shares);

    // Reconstruct the secret using a subset of shares
    let subset_of_shares: HashMap<i64, i64> = shares.iter().take(threshold).map(|(&x, &y)| (x, y)).collect();
    let reconstructed_secret = reconstruct_secret(&subset_of_shares);
    println!("Reconstructed Secret: {}", reconstructed_secret);
}

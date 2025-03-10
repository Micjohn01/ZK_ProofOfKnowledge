use ark_ff::PrimeField;
use crate::{
    new_circuit::{Circuit, Gate, Layer, Operator},
    gkr_sumcheck::{
        SumcheckProver, SumcheckVerifier,
        prove as sumcheck_prove, verify as sumcheck_verify,
        univariate_to_bytes, field_element_to_bytes
    },
    fiat_shamir_transcript::FiatShamirTranscript,
    sum_all_polys::SumPolynomial,
    product_poly::ProductPolynomial,
    multi_poly::MultiPoly,
};

#[derive(Clone, Debug)]
pub struct GKRProof<F: PrimeField> {
    pub claimed_output: F,
    pub sumcheck_proofs: Vec<SumcheckProver<F>>,
    pub wb_evals: Vec<F>,
    pub wc_evals: Vec<F>,
}

pub fn prove_gkr<F: PrimeField>(
    circuit: &mut Circuit<F>,
    inputs: &[F],
    transcript: &mut FiatShamirTranscript,
) -> GKRProof<F> {
    // Evaluate the circuit
    let output = circuit.evaluate(inputs.to_vec());

    let mut sumcheck_proofs = Vec::new();
    let mut wb_evals = Vec::new();
    let mut wc_evals = Vec::new();
    let mut alpha = F::zero();
    let mut beta = F::zero();
    let mut rb_values = Vec::new();
    let mut rc_values = Vec::new();

    // Layer 0 computation
    let mut w0_polynomial = circuit.w_i_polynomial(0);

    if w0_polynomial.evaluated_values.len() == 1 {
        let mut w0_padded_with_zero = w0_polynomial.evaluated_values.clone();
        w0_padded_with_zero.push(F::zero());
        w0_polynomial = MultiPoly::new(1, w0_padded_with_zero);
    }

    transcript.absorb(&w0_polynomial.convert_to_bytes());
    let random_challenge_a: F = transcript.sample_field_element(); // ra
    let mut claimed_sum = w0_polynomial.evaluate(&vec![random_challenge_a]); // m0

    // Handling subsequent layers
    for layer_index in 0..circuit.layers.len() {
        let (add_i_abc_polynomial, mul_i_abc_polynomial) = circuit.add_i_and_mul_i_mle(layer_index);

        let (add_i_bc, mul_i_bc) = if layer_index == 0 {
            (
                MultiPoly::partial_evaluate(&add_i_abc_polynomial.evaluated_values, 0, random_challenge_a), 
                MultiPoly::partial_evaluate(&mul_i_abc_polynomial.evaluated_values, 0, random_challenge_a)  
            )
        } else {
            compute_new_add_i_mul_i(
                alpha,
                beta,
                add_i_abc_polynomial,
                mul_i_abc_polynomial,
                &rb_values,
                &rc_values
            )
        };

        let wb_poly = circuit.w_i_polynomial(layer_index + 1);
        let wc_poly = wb_poly.clone();

        let fbc_polynomial = compute_fbc_polynomial(add_i_bc, mul_i_bc, &wb_poly, &wc_poly);

        let sumcheck_proof = sumcheck_prove(fbc_polynomial, claimed_sum, transcript);
        sumcheck_proofs.push(sumcheck_proof.clone());

        if layer_index < circuit.layers.len() - 1 {
            let sumcheck_challenges = sumcheck_proof.random_challenges;

            // Evaluate wb and wc to be used by verifier
            let (wb_evaluation, wc_evaluation) = evaluate_wb_wc(&wb_poly, &wc_poly, &sumcheck_challenges);

            wb_evals.push(wb_evaluation);
            wc_evals.push(wc_evaluation);

            // Using the randomness from the sumcheck proof, split into two vec! for rb and rc
            let middle = sumcheck_challenges.len() / 2;
            let (current_rb_values, current_rc_values) = sumcheck_challenges.split_at(middle);
            rb_values = current_rb_values.to_vec();
            rc_values = current_rc_values.to_vec();

            transcript.absorb(&field_element_to_bytes(wb_evaluation));
            alpha = transcript.sample_field_element();

            transcript.absorb(&field_element_to_bytes(wc_evaluation));
            beta = transcript.sample_field_element();

            // Compute claimed sum using linear combination form
            claimed_sum = (alpha * wb_evaluation) + (beta * wc_evaluation);
        }
    }

    GKRProof {
        claimed_output: output[0],
        sumcheck_proofs,
        wb_evals,
        wc_evals,
    }
}

pub fn verify_gkr<F: PrimeField>(
    circuit: &mut Circuit<F>,
    proof: GKRProof<F>,
    inputs: &[F],
    transcript: &mut FiatShamirTranscript,
) -> bool {
    let mut alpha = F::zero();
    let mut beta = F::zero();
    let mut prev_sumcheck_challenges = Vec::new();

    // Layer 0 computation
    let mut w0_polynomial = circuit.w_i_polynomial(0);

    if w0_polynomial.evaluated_values.len() == 1 {
        let mut w0_padded_with_zero = w0_polynomial.evaluated_values.clone();
        w0_padded_with_zero.push(F::zero());
        w0_polynomial = MultiPoly::new(1, w0_padded_with_zero); 
    }

    transcript.absorb(&w0_polynomial.convert_to_bytes());
    let random_challenge_a: F = transcript.sample_field_element();

    let mut claimed_sum = w0_polynomial.evaluate(&vec![random_challenge_a]);

    for layer_index in 0..circuit.layers.len() {
        if claimed_sum != proof.sumcheck_proofs[layer_index].claimed_sum {
            return false;
        }

        // Verify the Sumcheck proof
        let verify_result = sumcheck_verify(&proof.sumcheck_proofs[layer_index], transcript);
        if !verify_result.is_proof_valid {
            return false;
        }

        let sumcheck_challenges = verify_result.random_challenges;

        let (wb_evaluation, wc_evaluation) = if layer_index < circuit.layers.len() - 1 {
            (proof.wb_evals[layer_index], proof.wc_evals[layer_index])
        } else {
            let wb_poly = MultiPoly::new(1, inputs.to_vec());
            let wc_poly = wb_poly.clone();

            evaluate_wb_wc(&wb_poly, &wc_poly, &sumcheck_challenges)
        };

        let expected_claim = if layer_index == 0 {
            compute_verifier_initial_claim(
                circuit,
                layer_index,
                random_challenge_a,
                &sumcheck_challenges,
                wb_evaluation,
                wc_evaluation
            )
        } else {
            compute_verifier_folded_claim(
                circuit,
                layer_index,
                &sumcheck_challenges,
                &prev_sumcheck_challenges,
                wb_evaluation,
                wc_evaluation,
                alpha,
                beta
            )
        };

        if expected_claim != verify_result.last_claimed_sum {
            return false;
        }

        prev_sumcheck_challenges = sumcheck_challenges.to_vec();

        transcript.absorb(&field_element_to_bytes(wb_evaluation));
        alpha = transcript.sample_field_element();

        transcript.absorb(&field_element_to_bytes(wc_evaluation));
        beta = transcript.sample_field_element();

        claimed_sum = (alpha * wb_evaluation) + (beta * wc_evaluation);
    }

    true
}

fn compute_fbc_polynomial<F: PrimeField>(
    add_i_bc: MultiPoly<F>,
    mul_i_bc: MultiPoly<F>,
    w_b_polynomial: &MultiPoly<F>,
    w_c_polynomial: &MultiPoly<F>,
) -> SumPolynomial<F> {
    let add_wbc = MultiPoly::tensor_add_poly(w_b_polynomial, w_c_polynomial);
    let mul_wbc = MultiPoly::tensor_mul_poly(w_b_polynomial, w_c_polynomial);

    let add_i_term = ProductPolynomial::new(vec![add_i_bc, add_wbc]);
    let mul_i_term = ProductPolynomial::new(vec![mul_i_bc, mul_wbc]);

    SumPolynomial::new(vec![add_i_term, mul_i_term])
}

fn compute_new_add_i_mul_i<F: PrimeField>(
    alpha: F,
    beta: F,
    add_i_abc: MultiPoly<F>,
    mul_i_abc: MultiPoly<F>,
    rb_values: &[F],
    rc_values: &[F],
) -> (MultiPoly<F>, MultiPoly<F>) {
    let mut add_rb_bc = MultiPoly::partial_evaluate(&add_i_abc.evaluated_values, 0, rb_values[0]); // Fix: Removed &
    let mut add_rc_bc = MultiPoly::partial_evaluate(&add_i_abc.evaluated_values, 0, rc_values[0]); // Fix: Removed &

    let mut mul_rb_bc = MultiPoly::partial_evaluate(&mul_i_abc.evaluated_values, 0, rb_values[0]); // Fix: Removed &
    let mut mul_rc_bc = MultiPoly::partial_evaluate(&mul_i_abc.evaluated_values, 0, rc_values[0]); // Fix: Removed &

    for rb in rb_values.iter().skip(1) {
        add_rb_bc = MultiPoly::partial_evaluate(&add_rb_bc.evaluated_values, 0, *rb);
        mul_rb_bc = MultiPoly::partial_evaluate(&mul_rb_bc.evaluated_values, 0, *rb);
    }

    for rc in rc_values.iter().skip(1) {
        add_rc_bc = MultiPoly::partial_evaluate(&add_rc_bc.evaluated_values, 0, *rc);
        mul_rc_bc = MultiPoly::partial_evaluate(&mul_rc_bc.evaluated_values, 0, *rc);
    }

    let new_add_i = MultiPoly::add_polynomials(&add_rb_bc.scalar_mul(alpha), &add_rc_bc.scalar_mul(beta));
    let new_mul_i = MultiPoly::add_polynomials(&mul_rb_bc.scalar_mul(alpha), &mul_rc_bc.scalar_mul(beta));

    (new_add_i, new_mul_i)
}

fn evaluate_wb_wc<F: PrimeField>(
    wb_poly: &MultiPoly<F>,
    wc_poly: &MultiPoly<F>,
    sumcheck_challenges: &[F],
) -> (F, F) {
    let middle = sumcheck_challenges.len() / 2;
    let (rb_values, rc_values) = sumcheck_challenges.split_at(middle);

    let wb_poly_evaluated = wb_poly.evaluate(rb_values);
    let wc_poly_evaluated = wc_poly.evaluate(rc_values);

    (wb_poly_evaluated, wc_poly_evaluated)
}

fn compute_verifier_initial_claim<F: PrimeField>(
    circuit: &mut Circuit<F>,
    layer_index: usize,
    initial_random_challenge: F,
    sumcheck_challenges: &[F],
    wb_evaluation: F,
    wc_evaluation: F,
) -> F {
    let (add_i_abc, mul_i_abc) = circuit.add_i_and_mul_i_mle(layer_index);

    let (add_i_bc, mul_i_bc) = (
        MultiPoly::partial_evaluate(&add_i_abc.evaluated_values, 0, initial_random_challenge), 
        MultiPoly::partial_evaluate(&mul_i_abc.evaluated_values, 0, initial_random_challenge)  
    );

    let add_i_r = add_i_bc.evaluate(sumcheck_challenges);
    let mul_i_r = mul_i_bc.evaluate(sumcheck_challenges);

    (add_i_r * (wb_evaluation + wc_evaluation)) + (mul_i_r * (wb_evaluation * wc_evaluation))
}

fn compute_verifier_folded_claim<F: PrimeField>(
    circuit: &mut Circuit<F>,
    layer_index: usize,
    current_sumcheck_challenges: &[F],
    previous_sumcheck_challenges: &[F],
    wb_evaluation: F,
    wc_evaluation: F,
    alpha: F,
    beta: F,
) -> F {
    let (prev_rb, prev_rc) = previous_sumcheck_challenges.split_at(previous_sumcheck_challenges.len() / 2);

    let (add_i_abc, mul_i_abc) = circuit.add_i_and_mul_i_mle(layer_index);

    let (new_add_i, new_mul_i) = compute_new_add_i_mul_i(
        alpha,
        beta,
        add_i_abc,
        mul_i_abc,
        prev_rb,
        prev_rc
    );

    let add_r = new_add_i.evaluate(current_sumcheck_challenges);
    let mul_r = new_mul_i.evaluate(current_sumcheck_challenges);

    (add_r * (wb_evaluation + wc_evaluation)) + (mul_r * (wb_evaluation * wc_evaluation))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_gkr_protocol() {
        // Define a simple circuit
        let gate1 = Gate::new(0, 1, 0, Operator::Add);
        let gate2 = Gate::new(2, 3, 1, Operator::Mul);
        let layer0 = Layer::new(vec![gate1]);
        let layer1 = Layer::new(vec![gate2]);
        let mut circuit = Circuit::new(vec![layer0, layer1]);

        // Define inputs
        let inputs = vec![Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5)];

        // Initialize the transcript
        let mut prover_transcript = FiatShamirTranscript::new();
        let mut verifier_transcript = FiatShamirTranscript::new();

        // Run the prover
        let proof = prove_gkr(&mut circuit, &inputs, &mut prover_transcript);

        // Run the verifier
        let result = verify_gkr(&mut circuit, proof, &inputs, &mut verifier_transcript);

        // Assert that the proof is valid
        assert!(result, "The GKR proof is invalid!");
    }
}
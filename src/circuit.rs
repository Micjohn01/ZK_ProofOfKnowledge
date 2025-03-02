// use ark_ff::PrimeField;
// use updated_multilinear::evaluate::MultilinearPolynomial;
// use std::marker::PhantomData;

// use crate::updated_multilinear;

// #[derive(Debug, Clone, Copy)]
// pub enum Operator {
//     Add,
//     Mul,
// }

// #[derive(Debug)]
// pub struct Gate<F: PrimeField> {
//     op: Operator,
//     left_index: usize,
//     right_index: usize,
// }

// #[derive(Debug)]
// pub struct Layer<F: PrimeField> {
//     gates: Vec<Gate<F>>,
//     w_poly: MultilinearPolynomial<F>, // W_i for this layer
// }

// #[derive(Debug)]
// pub struct Circuit<F: PrimeField> {
//     layers: Vec<Layer<F>>,
//     _phantom: PhantomData<F>,
// }

// impl<F: PrimeField> Circuit<F> {
//     pub fn new(inputs: &[F]) -> Self {
//         let num_vars = (inputs.len() as f64).log2().ceil() as usize;
//         let w_poly = MultilinearPolynomial::new(inputs);
//         let input_layer = Layer {
//             gates: Vec::new(),
//             w_poly,
//         };
//         Circuit {
//             layers: vec![input_layer],
//             _phantom: PhantomData,
//         }
//     }

//     pub fn add_layer(&mut self, operations: &[Operator], input_pairs: &[(usize, usize)]) {
//         let prev_w_poly = &self.layers.last().unwrap().w_poly;
//         let num_vars = (operations.len() as f64).log2().ceil() as usize;
//         let mut outputs = vec![F::zero(); 1 << num_vars];
//         let mut gates = Vec::new();

//         for (i, &op) in operations.iter().enumerate() {
//             let (left_idx, right_idx) = input_pairs[i];
//             let left = prev_w_poly.evaluate(&index_to_binary(left_idx, prev_w_poly.num_vars()));
//             let right = prev_w_poly.evaluate(&index_to_binary(right_idx, prev_w_poly.num_vars()));
//             let output = match op {
//                 Operator::Add => left + right,
//                 Operator::Mul => left * right,
//             };
//             gates.push(Gate {
//                 op,
//                 left_index: left_idx,
//                 right_index: right_idx,
//             });
//             outputs[i] = output;
//         }

//         let w_poly = MultilinearPolynomial::new(&outputs);
//         self.layers.push(Layer { gates, w_poly });
//     }

//     pub fn w_i(&self, layer_id: usize) -> &MultilinearPolynomial<F> {
//         &self.layers[layer_id].w_poly
//     }

//     pub fn evaluate_w_i(&self, layer_id: usize, r: &[bool]) -> F {
//         self.w_i(layer_id).evaluate(r)
//     }

//     pub fn add_i_and_mul_i(&self, layer_id: usize) -> (MultilinearPolynomial<F>, MultilinearPolynomial<F>) {
//         let num_vars = self.w_i(layer_id).num_vars();
//         let size = 1 << num_vars;
//         let mut add_i_vals = vec![F::zero(); size];
//         let mut mul_i_vals = vec![F::zero(); size];

//         for (i, gate) in self.layers[layer_id].gates.iter().enumerate() {
//             match gate.op {
//                 Operator::Add => add_i_vals[i] = F::one(),
//                 Operator::Mul => mul_i_vals[i] = F::one(),
//             }
//         }

//         (
//             MultilinearPolynomial::new(&add_i_vals),
//             MultilinearPolynomial::new(&mul_i_vals),
//         )
//     }

//     pub fn wiring_predicate(&self, layer_id: usize, a: &[bool], b: &[bool], c: &[bool]) -> F {
//         let gate_idx = binary_to_index(a);
//         let gate = &self.layers[layer_id].gates[gate_idx];
//         let w_b = self.evaluate_w_i(layer_id + 1, b);
//         let w_c = self.evaluate_w_i(layer_id + 1, c);
//         match gate.op {
//             Operator::Add => w_b + w_c,
//             Operator::Mul => w_b * w_c,
//         }
//     }

//     pub fn add_count(&self, layer_id: usize) -> usize {
//         self.layers[layer_id].gates.iter().filter(|g| matches!(g.op, Operator::Add)).count()
//     }

//     pub fn mul_count(&self, layer_id: usize) -> usize {
//         self.layers[layer_id].gates.iter().filter(|g| matches!(g.op, Operator::Mul)).count()
//     }

//     pub fn final_outputs(&self) -> Vec<F> {
//         let last_w_poly = &self.layers.last().unwrap().w_poly;
//         let num_outputs = self.layers.last().unwrap().gates.len();
//         if num_outputs == 0 {
//             last_w_poly.values().to_vec()
//         } else {
//             (0..num_outputs)
//                 .map(|i| last_w_poly.evaluate(&index_to_binary(i, last_w_poly.num_vars())))
//                 .collect()
//         }
//     }
// }

// // Helper functions
// fn index_to_binary(index: usize, bits: usize) -> Vec<bool> {
//     (0..bits).rev().map(|i| (index >> i) & 1 == 1).collect()
// }

// fn binary_to_index(binary: &[bool]) -> usize {
//     binary.iter().enumerate().fold(0, |acc, (i, &bit)| {
//         if bit {
//             acc | (1 << (binary.len() - i - 1))
//         } else {
//             acc
//         }
//     })
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use ark_bn254::Fq;

//     #[test]
//     fn test_circuit() {
//         let inputs = vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)];
//         let mut circuit = Circuit::new(&inputs);

//         // Layer 1: [1+2, 3*4]
//         circuit.add_layer(&[Operator::Add, Operator::Mul], &[(0, 1), (2, 3)]);

//         // Layer 2: [3+12]
//         circuit.add_layer(&[Operator::Add], &[(0, 1)]);

//         // Check W_i
//         assert_eq!(circuit.evaluate_w_i(0, &[false, false]), Fq::from(1));
//         assert_eq!(circuit.evaluate_w_i(0, &[true, true]), Fq::from(4));
//         assert_eq!(circuit.evaluate_w_i(1, &[false]), Fq::from(3));
//         assert_eq!(circuit.evaluate_w_i(1, &[true]), Fq::from(12));
//         assert_eq!(circuit.evaluate_w_i(2, &[false]), Fq::from(15));

//         // Check add_i and mul_i
//         let (add_i_1, mul_i_1) = circuit.add_i_and_mul_i(1);
//         assert_eq!(add_i_1.evaluate(&[false]), Fq::from(1));
//         assert_eq!(add_i_1.evaluate(&[true]), Fq::from(0));
//         assert_eq!(mul_i_1.evaluate(&[false]), Fq::from(0));
//         assert_eq!(mul_i_1.evaluate(&[true]), Fq::from(1));

//         // Check wiring predicate
//         let result = circuit.wiring_predicate(0, &[false, false], &[false], &[true]);
//         assert_eq!(result, Fq::from(15)); // 3 + 12

//         // Final output
//         assert_eq!(circuit.final_outputs(), vec![Fq::from(15)]);
//     }
// }
use ark_ff::PrimeField;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
enum Operation {
    Add,
    Mul,
}

#[derive(Debug)]
struct Gate<F: PrimeField> {
    op: Operation,
    left_input: usize,
    right_input: usize,
    output: F,
}

#[derive(Debug)]
struct Layer<F: PrimeField> {
    gates: Vec<Gate<F>>,
    outputs: Vec<F>,
    add_count: usize, // Number of addition gates
    mul_count: usize, // Number of multiplication gates
}

#[derive(Debug)]
struct Circuit<F: PrimeField> {
    layers: Vec<Layer<F>>,
}

impl<F: PrimeField> Circuit<F> {
    // Create a new circuit with input values
    fn new(inputs: &[F]) -> Self {
        Circuit {
            layers: vec![Layer {
                gates: Vec::new(), // Input layer has no gates
                outputs: inputs.to_vec(),
                add_count: 0,
                mul_count: 0,
            }],
        }
    }

    // Add a computation layer with custom operations and input pairs
    fn add_layer(&mut self, operations: &[Operation], input_pairs: &[(usize, usize)]) {
        let prev_outputs = &self.layers.last().unwrap().outputs;
        let mut new_gates = Vec::new();
        let mut new_outputs = Vec::new();
        let mut add_count = 0;
        let mut mul_count = 0;

        for (i, &op) in operations.iter().enumerate() {
            let (left_idx, right_idx) = input_pairs[i];
            let left = prev_outputs[left_idx];
            let right = prev_outputs[right_idx];
            let output = match op {
                Operation::Add => {
                    add_count += 1;
                    left + right
                }
                Operation::Mul => {
                    mul_count += 1;
                    left * right
                }
            };

            new_gates.push(Gate {
                op,
                left_input: left_idx,
                right_input: right_idx,
                output,
            });

            new_outputs.push(output);
        }

        self.layers.push(Layer {
            gates: new_gates,
            outputs: new_outputs,
            add_count,
            mul_count,
        });
    }

    // Get the multilinear polynomial for a specific layer
    fn get_multilinear_polynomial(&self, layer_id: usize) -> HashMap<Vec<bool>, F> {
        let layer = &self.layers[layer_id];
        let num_gates = layer.outputs.len();
        let bits = (num_gates as f64).log2() as usize;

        let mut poly = HashMap::new();
        for (idx, &value) in layer.outputs.iter().enumerate() {
            let binary = index_to_binary(idx, bits);
            poly.insert(binary, value);
        }
        poly
    }

    // Compute the wiring predicate F_i(a, b, c)
    fn wiring_predicate(
        &self,
        layer_id: usize,
        a: &[bool],
        b: &[bool],
        c: &[bool],
    ) -> F {
        let layer = &self.layers[layer_id];
        let next_layer_poly = self.get_multilinear_polynomial(layer_id + 1);

        // Find the gate corresponding to index `a`
        let gate_index = binary_to_index(a);
        let gate = &layer.gates[gate_index];

        // Get values of W_{i+1}(b) and W_{i+1}(c)
        let w_b = next_layer_poly.get(b).unwrap_or(&F::zero());
        let w_c = next_layer_poly.get(c).unwrap_or(&F::zero());

        match gate.op {
            Operation::Add => *w_b + *w_c,
            Operation::Mul => *w_b * *w_c,
        }
    }

    // Get the number of addition gates in a layer
    fn add_count(&self, layer_id: usize) -> usize {
        self.layers[layer_id].add_count
    }

    // Get the number of multiplication gates in a layer
    fn mul_count(&self, layer_id: usize) -> usize {
        self.layers[layer_id].mul_count
    }

    // Get the final output(s) of the circuit
    fn final_outputs(&self) -> &[F] {
        &self.layers.last().unwrap().outputs
    }
}

// Helper functions
fn index_to_binary(index: usize, bits: usize) -> Vec<bool> {
    (0..bits).rev().map(|i| (index >> i) & 1 == 1).collect()
}

fn binary_to_index(binary: &[bool]) -> usize {
    binary.iter().enumerate().fold(0, |acc, (i, &bit)| {
        if bit {
            acc | (1 << (binary.len() - i - 1))
        } else {
            acc
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::Field;
    use ark_ff::One;
    use ark_ff::Zero;
    use ark_test_curves::bls12_381::Fr as F;

    #[test]
    fn test_flexible_outputs_with_final_addition() {
        let inputs = [F::from(1), F::from(2), F::from(3), F::from(4), F::from(5), F::from(6), F::from(7)];
        let mut circuit = Circuit::new(&inputs);
        
        // Layer 1: Custom operations and input pairs
        circuit.add_layer(
            &[Operation::Add, Operation::Mul, Operation::Add, Operation::Mul, Operation::Add],
            &[(0, 1), (2, 3), (4, 5), (1, 6), (0, 2)],
        );
        
        let layer1 = &circuit.layers[1];
        assert_eq!(layer1.outputs, vec![F::from(3), F::from(12), F::from(11), F::from(14), F::from(4)]); // [1+2, 3*4, 5+6, 2*7, 1+3]
        assert_eq!(layer1.add_count, 3); // 3 addition gates
        assert_eq!(layer1.mul_count, 2); // 2 multiplication gates

        // Layer 2: More operations
        circuit.add_layer(
            &[Operation::Mul, Operation::Add, Operation::Mul],
            &[(0, 1), (2, 3), (1, 4)],
        );
        
        let layer2 = &circuit.layers[2];
        assert_eq!(layer2.outputs, vec![F::from(36), F::from(25), F::from(48)]); // [3*12, 11+14, 12*4]
        assert_eq!(layer2.add_count, 1); // 1 addition gate
        assert_eq!(layer2.mul_count, 2); // 2 multiplication gates

        // Layer 3: Final operations
        circuit.add_layer(
            &[Operation::Add, Operation::Mul],
            &[(0, 1), (1, 2)],
        );
        
        let layer3 = &circuit.layers[3];
        assert_eq!(layer3.outputs, vec![F::from(61), F::from(1200)]); // [36+25, 25*48]
        assert_eq!(layer3.add_count, 1); // 1 addition gate
        assert_eq!(layer3.mul_count, 1); // 1 multiplication gate

        // Layer 4: Sum the outputs of Layer 3
        circuit.add_layer(
            &[Operation::Add],
            &[(0, 1)],
        );
        
        let layer4 = &circuit.layers[4];
        assert_eq!(layer4.outputs, vec![F::from(1261)]); // [61 + 1200]
        assert_eq!(layer4.add_count, 1); // 1 addition gate
        assert_eq!(layer4.mul_count, 0); // 0 multiplication gates

        // Final outputs
        assert_eq!(circuit.final_outputs(), &[F::from(1261)]);
    }

    #[test]
    fn test_multilinear_polynomial() {
        let inputs = [F::from(1), F::from(2), F::from(3), F::from(4)];
        let mut circuit = Circuit::new(&inputs);
        circuit.add_layer(
            &[Operation::Add, Operation::Mul],
            &[(0, 1), (2, 3)],
        );

        // Get multilinear polynomial for Layer 1
        let poly = circuit.get_multilinear_polynomial(1);
        assert_eq!(poly[&vec![false]], F::from(3)); // 1 + 2
        assert_eq!(poly[&vec![true]], F::from(12)); // 3 * 4
    }

    #[test]
    fn test_wiring_predicate() {
        let inputs = [F::from(1), F::from(2), F::from(3), F::from(4)];
        let mut circuit = Circuit::new(&inputs);
        circuit.add_layer(
            &[Operation::Add, Operation::Mul],
            &[(0, 1), (2, 3)],
        );

        // Test wiring predicate for Layer 0
        let a = vec![false]; // Gate 0 in Layer 0
        let b = vec![false]; // Gate 0 in Layer 1
        let c = vec![true];  // Gate 1 in Layer 1
        let result = circuit.wiring_predicate(0, &a, &b, &c);
        assert_eq!(result, F::from(15)); // 3 (W_1(b)) + 12 (W_1(c)) = 15
    }
}
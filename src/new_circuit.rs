use ark_ff::PrimeField;
use crate::multi_poly::MultiPoly;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Mul,
}

#[derive(Debug, Clone)]
pub struct Gate {
    pub left_index: usize,
    pub right_index: usize,
    pub output_index: usize,
    pub operator: Operator,
}

impl Gate {
    pub fn new(left_index: usize, right_index: usize, output_index: usize, operator: Operator) -> Self {
        Self {
            left_index,
            right_index,
            output_index,
            operator,
        }
    }

    pub fn evaluate<F: PrimeField>(&self, input: &[F]) -> F {
        let left_value = input[self.left_index];
        let right_value = input[self.right_index];
        match self.operator {
            Operator::Add => left_value + right_value,
            Operator::Mul => left_value * right_value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layer {
    pub gates: Vec<Gate>,
}

impl Layer {
    pub fn new(gates: Vec<Gate>) -> Self {
        Self { gates }
    }

    pub fn evaluate<F: PrimeField>(&self, input: &[F]) -> Vec<F> {
        let max_output_index = self.gates.iter().map(|gate| gate.output_index).max().unwrap_or(0);
        let mut output = vec![F::zero(); max_output_index + 1];

        for gate in &self.gates {
            let result = gate.evaluate(input);
            output[gate.output_index] += result;
        }

        output
    }
}

#[derive(Debug)]
pub struct Circuit<F: PrimeField> {
    pub layers: Vec<Layer>,
    pub layer_evaluations: Vec<Vec<F>>,
    _phantom: PhantomData<F>,
}

impl<F: PrimeField> Circuit<F> {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self {
            layers,
            layer_evaluations: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn evaluate(&mut self, input: Vec<F>) -> Vec<F> {
        let mut current_input = input;
        let mut evaluations = Vec::new();

        for layer in self.layers.iter().rev() {
            evaluations.push(current_input.clone());
            current_input = layer.evaluate(&current_input);
        }

        evaluations.reverse();
        self.layer_evaluations = evaluations;

        self.layer_evaluations[0].clone()
    }

    pub fn w_i_polynomial(&self, layer_index: usize) -> MultiPoly<F> {
        assert!(layer_index < self.layer_evaluations.len(), "Layer index out of bounds");
        let n_vars = self.num_layer_variables(layer_index);
        MultiPoly::new(n_vars, self.layer_evaluations[layer_index].clone())
    }

    pub fn add_i_and_mul_i_mle(&self, layer_index: usize) -> (MultiPoly<F>, MultiPoly<F>) {
        let n_vars = self.num_layer_variables(layer_index);
        let num_combinations = 1 << n_vars;

        let mut add_i_values = vec![F::zero(); num_combinations];
        let mut mul_i_values = vec![F::zero(); num_combinations];

        for gate in &self.layers[layer_index].gates {
            let index = self.gate_to_index(layer_index, gate);
            match gate.operator {
                Operator::Add => add_i_values[index] = F::one(),
                Operator::Mul => mul_i_values[index] = F::one(),
            }
        }

        let add_i_poly = MultiPoly::new(n_vars, add_i_values);
        let mul_i_poly = MultiPoly::new(n_vars, mul_i_values);

        (add_i_poly, mul_i_poly)
    }

    fn num_layer_variables(&self, layer_index: usize) -> usize {
        if layer_index == 0 {
            3
        } else {
            3 * layer_index + 2
        }
    }

    fn gate_to_index(&self, layer_index: usize, gate: &Gate) -> usize {
        let a = format!("{:0>width$b}", gate.output_index, width = layer_index);
        let b = format!("{:0>width$b}", gate.left_index, width = layer_index + 1);
        let c = format!("{:0>width$b}", gate.right_index, width = layer_index + 1);
        let combined = a + &b + &c;
        usize::from_str_radix(&combined, 2).unwrap_or(0)
    }
}
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use ark_bn254::Fq;

//     #[test]
//     fn test_circuit_evaluation() {
//         let input = vec![Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5)];

//         let gate1 = Gate::new(0, 1, 0, Operator::Mul);
//         let gate2 = Gate::new(0, 1, 0, Operator::Add);
//         let gate3 = Gate::new(2, 3, 1, Operator::Mul);

//         let layer0 = Layer::new(vec![gate1]);
//         let layer1 = Layer::new(vec![gate2, gate3]);

//         let mut circuit = Circuit::<Fq>::new(vec![layer0, layer1]);
//         let result = circuit.evaluate(input);

//         let expected_layers_evaluation = vec![
//             vec![Fq::from(100)],
//             vec![Fq::from(5), Fq::from(20)],
//             vec![Fq::from(2), Fq::from(3), Fq::from(4), Fq::from(5)],
//         ];

//         assert_eq!(result[0], Fq::from(100));
//         assert_eq!(circuit.layer_evaluations, expected_layers_evaluation);
//     }

    
// }
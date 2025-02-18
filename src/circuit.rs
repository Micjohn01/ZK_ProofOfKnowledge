enum Operation {
    Add,
    Mul,
}

struct GateId(usize);

struct Gate {
    id: GateId,
    operator: Operation,
    inputs: Vec<GateId>,
    value: Option<u64>,
}

struct Layer {
    gates: Vec<Gate>,
}

// Operation tracking structures
#[derive(Debug, Default)]
struct LayerStats {
    add_count: usize,
    mul_count: usize,
}

struct GateStats {
    is_add: bool,
    is_mul: bool,
}

struct Circuit {
    layers: Vec<Layer>,
}

impl Circuit {
    fn new(input_layer: Vec<u64>) -> Self {
        let input_gates = input_layer
            .into_iter()
            .enumerate()
            .map(|(idx, val)| Gate {
                id: GateId(idx),
                operator: Operation::Add,
                inputs: Vec::new(),
                value: Some(val),
            })
            .collect();

        Circuit {
            layers: vec![Layer { gates: input_gates }],
        }
    }

    fn add_layer(&mut self, operations: Vec<(Operation, Vec<GateId>)>) {
        let mut new_layer = Vec::with_capacity(operations.len());

        for (idx, (operator, inputs)) in operations.into_iter().enumerate() {
            new_layer.push(Gate {
                id: GateId(idx),
                operator,
                inputs,
                value: None,
            });
        }

        self.layers.push(Layer { gates: new_layer });
    }

    fn evaluate(&mut self) {
        for layer_idx in 1..self.layers.len() {
            let (prev_layers, current_layers) = self.layers.split_at_mut(layer_idx);
            let prev_layer = &prev_layers[layer_idx - 1].gates;
            let current_layer = &mut current_layers[0].gates;

            for gate in current_layer.iter_mut() {
                let inputs: Vec<u64> = gate.inputs
                    .iter()
                    .map(|id| prev_layer[id.0].value.unwrap())
                    .collect();

                gate.value = Some(match gate.operator {
                    Operation::Add => inputs.iter().sum(),
                    Operation::Mul => inputs.iter().product(),
                });
            }
        }
    }

    fn output(&self) -> Option<u64> {
        self.layers.last()?.gates.first()?.value
    }

    // Operation counting methods
    fn layer_operations(&self, layer_idx: usize) -> Option<LayerStats> {
        self.layers.get(layer_idx).map(|layer| {
            let mut stats = LayerStats::default();
            for gate in &layer.gates {
                match gate.operator {
                    Operation::Add => stats.add_count += 1,
                    Operation::Mul => stats.mul_count += 1,
                }
            }
            stats
        })
    }

    fn gate_operation(&self, layer_idx: usize, gate_id: GateId) -> Option<GateStats> {
        self.layers.get(layer_idx).and_then(|layer| {
            layer.gates.get(gate_id.0).map(|gate| GateStats {
                is_add: matches!(gate.operator, Operation::Add),
                is_mul: matches!(gate.operator, Operation::Mul),
            })
        })
    }

    fn circuit_operations(&self) -> Vec<LayerStats> {
        self.layers.iter().map(|layer| {
            let mut stats = LayerStats::default();
            for gate in &layer.gates {
                match gate.operator {
                    Operation::Add => stats.add_count += 1,
                    Operation::Mul => stats.mul_count += 1,
                }
            }
            stats
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_counting() {
        let mut circuit = Circuit::new(vec![2, 3]);
        circuit.add_layer(vec![
            (Operation::Add, vec![GateId(0), GateId(1)]),
            (Operation::Mul, vec![GateId(0), GateId(1)])
        ]);
        
        // Test layer operations
        assert_eq!(circuit.layer_operations(0).unwrap().add_count, 2);
        let layer1 = circuit.layer_operations(1).unwrap();
        assert_eq!(layer1.add_count, 1);
        assert_eq!(layer1.mul_count, 1);
        
        // Test gate operations
        let gate0 = circuit.gate_operation(1, GateId(0)).unwrap();
        assert!(gate0.is_add);
        let gate1 = circuit.gate_operation(1, GateId(1)).unwrap();
        assert!(gate1.is_mul);
        
        // Test full circuit stats
        let all_stats = circuit.circuit_operations();
        assert_eq!(all_stats[0].add_count, 2);
        assert_eq!(all_stats[1].mul_count, 1);
    }
}
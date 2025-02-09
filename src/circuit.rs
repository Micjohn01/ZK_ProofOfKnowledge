
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

struct Circuit {
    layers: Vec<Layer>,
}

impl Circuit {
    fn new (input_layer: Vec<u64>) -> Self {
        let input_gates = input_layer.into_iter().enumerate().map(|(idx, val)| Gate {
            id: GateId(idx),
            operator: Operation::Add,
            inputs: Vec::new(),
            value: Some(val),
        }).collect();

        Circuit { layers: vec![Layer { gates: input_gates}], }
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

    self.layers.push(Layer { gates: new_layer});
}

fn evaluate(&mut self) {
    for layer_idx in 1..self.layers.len() {
        let prev_layer = &self.layers[layer_idx - 1].gates;
        let current_layer = &mut self.layers[layer_idx].gates;

        for gate in current_layer.iter_mut() {
            let inputs: Vec<u64> = gate.inputs.iter().map(|id| prev_layer[id.0].value.unwrap()).collect();

            gate.value = Some(match gate.operator{
                Operation::Add => inputs.iter().sum(),
                Operation::Mul => inputs.iter().product(),
            });
        }
    }
}


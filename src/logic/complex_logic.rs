use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::And;
use crate::logic::foundations::{ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW, HIGH};

pub struct VariableBitCPUEnable {
    complex_gate: ComplexGateMembers,
    and_gates: Vec<Rc<RefCell<And>>>,
}

#[allow(dead_code)]
impl VariableBitCPUEnable {
    pub fn new(number_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut one_bit_memory_cells: Vec<Rc<RefCell<And>>> = Vec::new();

        for i in 0..number_bits {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            one_bit_memory_cells.push(
                And::new(2, 1)
            );
        }

        let enable_input_gate = SimpleInput::new(number_bits, "E");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(enable_input_gate.clone());

        let mut one_bit_memory_cell = VariableBitCPUEnable {
            complex_gate: ComplexGateMembers::new(
                number_bits + 1,
                number_bits,
                GateType::VariableCPUEnableType,
                input_gates,
                output_gates,
            ),
            and_gates: one_bit_memory_cells,
        };

        one_bit_memory_cell.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(one_bit_memory_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();

        for i in 0..number_bits {
            self.complex_gate.input_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                self.and_gates[i].clone(),
            );

            e_input_gate.borrow_mut().connect_output_to_next_gate(
                i,
                1,
                self.and_gates[i].clone(),
            );

            self.and_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                output_gates[i].clone(),
            );
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for VariableBitCPUEnable {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
            None,
        )
    }

    fn get_gate_type(&self) -> GateType {
        self.complex_gate.simple_gate.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.complex_gate.simple_gate.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.complex_gate.simple_gate.should_print_output = print_output;
    }

    fn get_tag(&self) -> String {
        self.complex_gate.simple_gate.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.complex_gate.simple_gate.tag = tag.to_string();
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use rand::Rng;
    use crate::test_stuff::run_multi_input_output_logic_gate;
    use super::*;

    #[test]
    fn cpu_enable_initialization() {
        let num_bits = rand::thread_rng().gen_range(1..=16);
        let cpu_enable = VariableBitCPUEnable::new(num_bits);

        let output = cpu_enable.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), num_bits);
        for out in output {
            match out {
                GateOutputState::NotConnected(signal) => {
                    assert_eq!(signal, LOW);
                }
                GateOutputState::Connected(_) => panic!("Final output gate should never be connected.")
            }
        }
    }

    #[test]
    fn cpu_enable_inputs_change() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH, LOW],
                vec![HIGH, HIGH, LOW],
                vec![LOW, HIGH, LOW],
            ],
            vec![
                vec![HIGH, HIGH, LOW],
                vec![LOW, LOW, LOW],
                vec![LOW, HIGH, LOW],
            ],
            HashMap::from(
                [("E",vec![vec![HIGH], vec![LOW], vec![HIGH]])]
            ),
            VariableBitCPUEnable::new(3),
        );
    }
}
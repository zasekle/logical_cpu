use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::complex_logic::VariableBitCPUEnable;
use crate::logic::foundations::{ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW, HIGH};
use crate::logic::memory_gates::VariableBitMemoryCell;

pub struct VariableBitRegister {
    complex_gate: ComplexGateMembers,
    memory: Rc<RefCell<VariableBitMemoryCell>>,
    enable: Rc<RefCell<VariableBitCPUEnable>>,
}

#[allow(dead_code)]
impl VariableBitRegister {
    pub fn new(number_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        for i in 0..number_bits {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);
        }

        let set_input_gate = SimpleInput::new(number_bits, "S");
        let enable_input_gate = SimpleInput::new(number_bits, "E");

        input_gates.push(set_input_gate.clone());
        input_gates.push(enable_input_gate.clone());

        let mut one_bit_memory_cell = VariableBitRegister {
            complex_gate: ComplexGateMembers::new(
                number_bits + 2,
                number_bits,
                GateType::VariableBitRegisterType,
                input_gates,
                output_gates,
            ),
            memory: VariableBitMemoryCell::new(number_bits),
            enable: VariableBitCPUEnable::new(number_bits),
        };

        one_bit_memory_cell.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(one_bit_memory_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let mut memory_gate = self.memory.borrow_mut();
        let mut enable_gate = self.enable.borrow_mut();

        //TODO: delete me
        memory_gate.toggle_output_printing(true);
        enable_gate.toggle_output_printing(true);

        for i in 0..number_bits {
            let mut input_gate = self.complex_gate.input_gates[i].borrow_mut();

            input_gate.connect_output_to_next_gate(
                0,
                i,
                self.memory.clone(),
            );

            memory_gate.connect_output_to_next_gate(
                i,
                i,
                self.enable.clone(),
            );

            enable_gate.connect_output_to_next_gate(
                i,
                0,
                output_gates[i].clone(),
            );
        }

        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();
        let memory_set_index = memory_gate.get_index_from_tag("S");
        s_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            memory_set_index,
            self.memory.clone(),
        );

        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();
        let enable_set_index = enable_gate.get_index_from_tag("E");
        e_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            enable_set_index,
            self.enable.clone(),
        );

        drop(memory_gate);
        drop(enable_gate);

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(true);
    }
}

impl LogicGate for VariableBitRegister {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(current_gate_output_key, next_gate_input_key, next_gate);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(&self.get_tag())
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

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
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
    fn processor_register_initialization() {
        let num_bits = rand::thread_rng().gen_range(1..=16);
        let register = VariableBitRegister::new(num_bits);

        let output = register.borrow_mut().fetch_output_signals().unwrap();

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
    fn processor_register_simple_test() {
        let gate = VariableBitRegister::new(2);
        gate.borrow_mut().toggle_output_printing(true);

        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH]
            ],
            vec![
                vec![HIGH, HIGH],
            ],
            HashMap::from(
                [
                    ("S", vec![HIGH]),
                    ("E", vec![HIGH])
                ],
            ),
            gate,
        );
    }
}
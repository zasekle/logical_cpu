use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::{And, ControlledBuffer, Not};
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
    controlled_buffer: Rc<RefCell<ControlledBuffer>>,
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

        let set_input_gate = SimpleInput::new(1, "S");
        let enable_input_gate = SimpleInput::new(2, "E");

        input_gates.push(set_input_gate.clone());
        input_gates.push(enable_input_gate.clone());

        let mut bit_register = VariableBitRegister {
            complex_gate: ComplexGateMembers::new(
                number_bits + 2,
                number_bits,
                GateType::VariableBitRegisterType,
                input_gates,
                output_gates,
            ),
            memory: VariableBitMemoryCell::new(number_bits),
            enable: VariableBitCPUEnable::new(number_bits),
            controlled_buffer: ControlledBuffer::new(number_bits),
        };

        bit_register.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(bit_register))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let mut memory_gate = self.memory.borrow_mut();
        let mut enable_gate = self.enable.borrow_mut();
        let mut controlled_buffer_gate = self.controlled_buffer.borrow_mut();

        controlled_buffer_gate.toggle_output_printing(true);
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
                i,
                self.controlled_buffer.clone(),
            );

            controlled_buffer_gate.connect_output_to_next_gate(
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

        let enable_controlled_buffer_index = controlled_buffer_gate.get_index_from_tag("E");
        e_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            enable_controlled_buffer_index,
            self.controlled_buffer.clone(),
        );

        drop(memory_gate);
        drop(enable_gate);
        drop(controlled_buffer_gate);

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

pub struct VariableDecoder {
    complex_gate: ComplexGateMembers,
    and_gates: Vec<Rc<RefCell<And>>>,
    not_gates: Vec<Rc<RefCell<Not>>>,
}

#[allow(dead_code)]
impl VariableDecoder {
    pub fn new(number_inputs: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_inputs, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let number_outputs = usize::pow(2, number_inputs as u32);

        for i in 0..number_inputs {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(number_outputs/2 + 1, input_tag.as_str()));
        }

        for i in 0..number_outputs {
            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);
        }

        let mut and_gates = Vec::with_capacity(number_outputs);
        for _ in 0..number_outputs {
            and_gates.push(And::new(number_inputs,1));
        }

        let mut not_gates = Vec::with_capacity(number_inputs);
        for _ in 0..number_inputs {
            not_gates.push(Not::new(number_outputs/2));
        }

        let mut decoder = VariableDecoder {
            complex_gate: ComplexGateMembers::new(
                number_inputs,
                number_outputs,
                GateType::VariableDecoderType,
                input_gates,
                output_gates,
            ),
            and_gates,
            not_gates,
        };

        decoder.build_and_prime_circuit(number_inputs, number_outputs, output_gates_logic);

        Rc::new(RefCell::new(decoder))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_inputs: usize,
        number_outputs: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..number_inputs {
            let mut input_gate = self.complex_gate.input_gates[i].borrow_mut();

            input_gate.connect_output_to_next_gate(
                0,
                0,
                self.not_gates[i].clone()
            );
        }

        let mut input_gate_index = vec![1; number_inputs];
        let mut not_gate_index = vec![0; number_inputs];

        for i in 0..number_outputs {
            let mut and_gate = self.and_gates[i].borrow_mut();

            //This will make a binary number formatted as a String with `number_inputs` digits.
            let binary_number = format!("{:0width$b}", i, width = number_inputs);

            for (j, c) in binary_number.chars().enumerate() {
                if c == '0' { // '0' means connects from output.
                    let next_index = not_gate_index[j];
                    not_gate_index[j] += 1;
                    self.not_gates[j].borrow_mut().connect_output_to_next_gate(
                        next_index,
                        j,
                        self.and_gates[i].clone(),
                    );
                } else { // '1' means connects from input.
                    let next_index = input_gate_index[j];
                    input_gate_index[j] += 1;
                    self.complex_gate.input_gates[j].borrow_mut().connect_output_to_next_gate(
                        next_index,
                        j,
                        self.and_gates[i].clone(),
                    );
                }
            }

            and_gate.connect_output_to_next_gate(
                0,
                0,
                output_gates[i].clone()
            );
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(true);
    }
}

impl LogicGate for VariableDecoder {
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
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH, LOW]
            ],
            vec![
                vec![HIGH, HIGH, LOW],
            ],
            HashMap::from(
                [
                    ("S", vec![HIGH]),
                    ("E", vec![HIGH])
                ],
            ),
            VariableBitRegister::new(3),
        );
    }

    #[test]
    fn processor_register_set_bit_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW, HIGH],
                vec![HIGH, HIGH, LOW],
            ],
            vec![
                vec![HIGH, LOW, HIGH],
                vec![HIGH, LOW, HIGH],
            ],
            HashMap::from(
                [
                    ("S", vec![HIGH, LOW]),
                    ("E", vec![HIGH, HIGH])
                ],
            ),
            VariableBitRegister::new(3),
        );
    }

    #[test]
    fn processor_register_enable_bit_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW, HIGH, HIGH],
                vec![HIGH, HIGH, LOW, LOW],
            ],
            vec![
                vec![HIGH, LOW, HIGH, HIGH],
                vec![LOW, LOW, LOW, LOW],
            ],
            HashMap::from(
                [
                    ("S", vec![HIGH, HIGH]),
                    ("E", vec![HIGH, LOW])
                ],
            ),
            VariableBitRegister::new(4),
        );
    }

    #[test]
    fn decoder_initialization() {
        let num_bits = rand::thread_rng().gen_range(1..=16);
        let register = VariableDecoder::new(num_bits);

        let output = register.borrow_mut().fetch_output_signals().unwrap();

        println!("output_ {:#?}", output);
        assert_eq!(output.len(), usize::pow(2, num_bits as u32));
        for (i, out) in output.into_iter().enumerate() {
            match out {
                GateOutputState::NotConnected(signal) => {
                    if i == 0 {
                        assert_eq!(signal, HIGH);
                    } else {
                        assert_eq!(signal, LOW);
                    }
                }
                GateOutputState::Connected(_) => panic!("Final output gate should never be connected.")
            }
        }
    }

    #[test]
    fn decoder_all_numbers() {
        let number_inputs = rand::thread_rng().gen_range(1..=5);
        let number_outputs = usize::pow(2, number_inputs as u32);
        let decoder = VariableDecoder::new(number_inputs);

        let mut input_vector = Vec::new();
        let mut output_vector = Vec::new();
        for i in 0..number_outputs {
            let binary_input_number = format!("{:0width$b}", i, width = number_inputs);

            let mut i_vector = Vec::with_capacity(number_inputs);
            for c in binary_input_number.chars() {
                if c == '0' {
                    i_vector.push(LOW);
                } else {
                    i_vector.push(HIGH);
                }
            }
            input_vector.push(i_vector);

            let mut o_vector = vec![LOW; number_outputs];
            o_vector[i] = HIGH;

            output_vector.push(o_vector);
        }

        run_multi_input_output_logic_gate(
            input_vector,
            output_vector,
            HashMap::new(),
            decoder.clone()
        );
    }
}
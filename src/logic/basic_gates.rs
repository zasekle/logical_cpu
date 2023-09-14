use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, BasicGateMembers, InputSignalReturn};
use crate::logic::foundations::Signal::{HIGH, LOW};

pub struct Or {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Or {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Or {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::OrType,
                        None,
                    )
                }
            )
        )
    }
}

impl LogicGate for Or {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {

        self.members.connect_output_to_next_gate(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct And {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl And {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                And {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::AndType,
                        None,
                    )
                }
            )
        )
    }
}

impl LogicGate for And {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct Not {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Not {
    pub fn new(output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Not {
                    members: BasicGateMembers::new(
                        1,
                        output_num,
                        GateType::NotType,
                        None,
                    )
                }
            )
        )
    }
}

impl LogicGate for Not {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct Nor {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Nor {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Nor {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::NorType,
                        None,
                    )
                }
            )
        )
    }
}

impl LogicGate for Nor {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct Nand {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Nand {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Nand {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::NandType,
                        None,
                    )
                }
            )
        )
    }
}

impl LogicGate for Nand {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct ControlledBuffer {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl ControlledBuffer {
    pub fn new(input_output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                ControlledBuffer {
                    members: BasicGateMembers::new(
                        input_output_num + 1,
                        input_output_num,
                        GateType::ControlledBufferType,
                        None,
                    )
                }
            )
        )
    }
}

impl LogicGate for ControlledBuffer {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        //When input index 0 is HIGH, allow the signal through, otherwise return NotConnected.
        let enable_index = self.get_index_from_tag("E");
        if self.members.input_signals[enable_index] == HIGH {
            GateLogic::fetch_output_signals_basic_gate(&mut self.members)
        } else {
            let output_states = vec![GateOutputState::NotConnected(LOW); self.members.output_states.len()];
            Ok(output_states)
        }
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        if tag == "E" {
            self.members.input_signals.len() - 1
        } else {
            panic!("Gate {} using tag {} id {} did not exist.", self.get_tag(), tag, self.get_unique_id().id())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::start_clock;
    use crate::test_stuff::check_for_single_element_signal;
    use super::*;

    fn test_simple_gate(
        gate: Rc<RefCell<dyn LogicGate>>,
        first_input: Signal,
        second_input: Option<Signal>,
        output: Signal,
    ) {
        let first_pin_input = AutomaticInput::new(vec![first_input], 1, "");
        let output_gate = SimpleOutput::new("");

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        input_gates.push(first_pin_input.clone());
        output_gates.push(output_gate.clone());

        first_pin_input.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            gate.clone(),
        );

        if let Some(second_input) = second_input {
            let second_pin_input = AutomaticInput::new(vec![second_input], 1, "");

            second_pin_input.borrow_mut().connect_output_to_next_gate(
                0,
                1,
                gate.clone(),
            );

            input_gates.push(second_pin_input.clone());
        }

        gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        start_clock(
            &input_gates,
            &output_gates,
            &mut |_: &Vec<(String, Vec<GateOutputState>)>, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                check_for_single_element_signal(&output_gates, output.clone());
            },
        );
    }

    fn test_controlled_buffer(
        signal: Signal
    ) {
        let output_gate = SimpleOutput::new("OUT");
        let controlled_buffer = ControlledBuffer::new(1);
        let mut mut_controlled_buffer = controlled_buffer.borrow_mut();

        mut_controlled_buffer.connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        mut_controlled_buffer.update_input_signal(
            GateInput {
                input_index: 0,
                signal: signal.clone(),
            }
        );

        let enable_index = mut_controlled_buffer.get_index_from_tag("E");
        mut_controlled_buffer.update_input_signal(
            GateInput {
                input_index: enable_index,
                signal: HIGH,
            }
        );

        let output = mut_controlled_buffer.fetch_output_signals().unwrap();

        for gate_output_state in output {
            match gate_output_state {
                GateOutputState::NotConnected(_) => panic!("Output should be connected when pin is low."),
                GateOutputState::Connected(connected_output) => {
                    assert_eq!(connected_output.throughput.signal, signal);
                    let connected_id = connected_output.gate.borrow_mut().get_unique_id().id();
                    let output_id = output_gate.borrow_mut().get_unique_id().id();
                    assert_eq!(connected_id, output_id);
                }
            }
        }
    }
    #[test]
    fn test_or_gate_low_low() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            LOW,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_or_gate_low_high() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            LOW,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_or_gate_high_low() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            HIGH,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_or_gate_high_high() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            HIGH,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_and_gate_low_low() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            LOW,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_and_gate_low_high() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            LOW,
            Some(HIGH),
            LOW,
        );
    }

    #[test]
    fn test_and_gate_high_low() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            HIGH,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_and_gate_high_high() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            HIGH,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_not_gate_low() {
        let not_gate = Not::new(1);

        test_simple_gate(
            not_gate,
            LOW,
            None,
            HIGH,
        );
    }

    #[test]
    fn test_not_gate_high() {
        let not_gate = Not::new(1);

        test_simple_gate(
            not_gate,
            HIGH,
            None,
            LOW,
        );
    }

    #[test]
    fn test_nor_gate_low_low() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            LOW,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_nor_gate_low_high() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            LOW,
            Some(HIGH),
            LOW,
        );
    }

    #[test]
    fn test_nor_gate_high_low() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            HIGH,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_nor_gate_high_high() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            HIGH,
            Some(HIGH),
            LOW,
        );
    }

    #[test]
    fn test_nand_gate_low_low() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            LOW,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_low_high() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            HIGH,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_high_low() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            LOW,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_high_high() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            HIGH,
            Some(HIGH),
            LOW,
        );
    }

    #[test]
    fn test_controlled_buffer_initialization() {
        let output_gate = SimpleOutput::new("OUT");
        let controlled_buffer = ControlledBuffer::new(1);
        let mut mut_controlled_buffer = controlled_buffer.borrow_mut();
        mut_controlled_buffer.connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        mut_controlled_buffer.update_input_signal(
            GateInput {
                input_index: 0,
                signal: HIGH,
            }
        );

        let output = mut_controlled_buffer.fetch_output_signals().unwrap();

        for gate_output_state in output {
            match gate_output_state {
                GateOutputState::NotConnected(signal) => assert_eq!(signal, LOW),
                GateOutputState::Connected(_) => panic!("Output should not be connected when pin is low.")
            }
        }
    }

    #[test]
    fn test_controlled_buffer_high() {
        test_controlled_buffer(HIGH);
    }

    #[test]
    fn test_controlled_buffer_low() {
        test_controlled_buffer(LOW);
    }
}
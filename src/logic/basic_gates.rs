use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, BasicGateMembers, InputSignalReturn, ConnectedOutput, calculate_input_signals_from_all_inputs, Signal, calculate_input_signal_from_single_inputs};
use crate::logic::foundations::Signal::{HIGH, LOW, NONE};

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

    fn get_tag(&self) -> String {
        self.members.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.members.tag = tag.to_string()
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
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

    fn get_tag(&self) -> String {
        self.members.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.members.tag = tag.to_string()
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
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

    fn get_tag(&self) -> String {
        self.members.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.members.tag = tag.to_string()
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
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

    fn get_tag(&self) -> String {
        self.members.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.members.tag = tag.to_string()
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
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

    fn get_tag(&self) -> String {
        self.members.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.members.tag = tag.to_string()
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

pub struct Splitter {
    members: BasicGateMembers,
    outputs_per_input: usize,
}

#[allow(dead_code)]
impl Splitter {
    pub fn new(input_num: usize, outputs_per_input: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Splitter {
                    members: BasicGateMembers::new(
                        input_num,
                        input_num * outputs_per_input,
                        GateType::SplitterType,
                        Some(LOW),
                    ),
                    outputs_per_input,
                }
            )
        )
    }

    pub fn get_index_for_output(&self, current_gate_output_index: usize, index_of_output: usize) -> usize {
        current_gate_output_index * self.outputs_per_input + index_of_output
    }
}

impl LogicGate for Splitter {
    //current_gate_output_key is meant to be extracted from Splitter::get_index_for_output()
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        let output_signal = calculate_input_signal_from_single_inputs(
            &self.members.input_signals[current_gate_output_key/self.outputs_per_input]
        );

        GateLogic::connect_output_to_next_gate_no_calculate(
            self.get_unique_id(),
            &mut self.members.output_states,
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
            output_signal,
            self.members.gate_type,
            self.members.should_print_output,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        //output_states is outputs_per_input*num_inputs length and input_states is num_inputs length.
        let input_signals = calculate_input_signals_from_all_inputs(&self.members.input_signals);
        for (i, output) in self.members.output_states.iter_mut().enumerate() {
            match output {
                GateOutputState::NotConnected(signal) => {
                    *signal = input_signals[i / self.outputs_per_input].clone()
                }
                GateOutputState::Connected(connected_output) => {
                    connected_output.throughput.signal = input_signals[i / self.outputs_per_input].clone()
                }
            }
        }

        if self.members.should_print_output {
            GateLogic::print_gate_output(
                &self.members.gate_type,
                &self.members.unique_id,
                &String::from(""),
                &self.members.output_states,
            );
        }

        Ok(self.members.output_states.clone())
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

    fn get_tag(&self) -> String {
        self.members.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.members.tag = tag.to_string()
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
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
                        Some(NONE),
                    )
                }
            )
        )
    }
}

impl LogicGate for ControlledBuffer {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        let enable_index = self.get_index_from_tag("E");
        let input_signals = calculate_input_signals_from_all_inputs(&self.members.input_signals);
        let output_signal = if input_signals[enable_index] == HIGH {
            input_signals[current_gate_output_key].clone()
        } else {
            NONE
        };

        GateLogic::connect_output_to_next_gate_no_calculate(
            self.get_unique_id(),
            &mut self.members.output_states,
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
            output_signal,
            self.members.gate_type,
            self.members.should_print_output,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        //When input index 0 is HIGH, allow the signal through, otherwise return NotConnected.
        let input_signals = calculate_input_signals_from_all_inputs(&self.members.input_signals);
        let enable_index = self.get_index_from_tag("E");
        let output = if input_signals[enable_index] == HIGH {
            //input_signals and output_states are the same length.
            for (i, output) in self.members.output_states.iter_mut().enumerate() {
                match output {
                    GateOutputState::NotConnected(signal) => {
                        *signal = input_signals[i].clone()
                    }
                    GateOutputState::Connected(connected_output) => {
                        connected_output.throughput.signal = input_signals[i].clone()
                    }
                }
            }

            self.members.output_states.clone()
        } else {
            let mut output_states = Vec::new();
            for output_state in self.members.output_states.iter() {
                match output_state {
                    GateOutputState::NotConnected(_) => {
                        output_states.push(GateOutputState::NotConnected(NONE));
                    }
                    GateOutputState::Connected(connected_output) => {
                        output_states.push(
                            GateOutputState::Connected(
                                ConnectedOutput {
                                    throughput: GateInput::new(
                                        connected_output.throughput.input_index,
                                        NONE,
                                        connected_output.throughput.sending_id,
                                    ),
                                    gate: connected_output.gate.clone(),
                                }
                            )
                        );
                    }
                }
            }
            output_states
        };

        if self.members.should_print_output {
            GateLogic::print_gate_output(
                &self.members.gate_type,
                &self.members.unique_id,
                &String::from(""),
                &output,
            );
        }

        Ok(output)
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

    fn get_tag(&self) -> String {
        self.members.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.members.tag = tag.to_string()
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        if tag == "E" {
            self.members.input_signals.len() - 1
        } else {
            panic!("Gate {} using tag {} id {} did not exist.", self.get_tag(), tag, self.get_unique_id().id())
        }
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::globals::CLOCK_TICK_NUMBER;
    use crate::logic::foundations::{ComplexGateMembers, Signal};
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::{AutomaticInput, SimpleInput};
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::{run_circuit, start_clock};
    use crate::test_stuff::{check_for_single_element_signal, collect_outputs_from_output_gates};
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

        controlled_buffer.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        controlled_buffer.borrow_mut().update_input_signal(
            GateInput::new(
                0,
                signal.clone(),
                UniqueID::zero_id(),
            )
        );

        let enable_index = controlled_buffer.borrow_mut().get_index_from_tag("E");
        controlled_buffer.borrow_mut().update_input_signal(
            GateInput::new(
                enable_index,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        let output = controlled_buffer.borrow_mut().fetch_output_signals().unwrap();

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

    fn collect_output_for_run_circuit(collected_output: &mut Vec<Vec<Signal>>, output_gates: &&Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>) {
        assert_eq!(output_gates.len(), 1);

        let mut single_collected_output = Vec::new();
        collect_outputs_from_output_gates(&output_gates, &mut single_collected_output);

        collected_output.push(single_collected_output);
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
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        let output = mut_controlled_buffer.fetch_output_signals().unwrap();

        for gate_output_state in output {
            match gate_output_state {
                GateOutputState::NotConnected(_) => panic!("Output should be connected."),
                GateOutputState::Connected(output) => {
                    assert_eq!(output.throughput.signal, NONE);
                }
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

    #[test]
    fn test_none_signal_working() {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        let num_gates = rand::thread_rng().gen_range(2..=16);
        let throughput_gate_index = rand::thread_rng().gen_range(0..num_gates);

        let output_gate = SimpleOutput::new("OUT");

        let single_enable_input_gate = AutomaticInput::new(vec![LOW, LOW, LOW, LOW, HIGH, HIGH, HIGH, HIGH], 1, "Single_Enable");
        // If this value goes high, the value will be unpredictable.
        let other_enable_input_gates = AutomaticInput::new(vec![LOW; 8], num_gates - 1, "Other_Enable");

        let output_signal = vec![[NONE], [NONE], [NONE], [NONE], [HIGH], [HIGH], [LOW], [LOW]];

        for i in 0..num_gates {
            let input_gate =
                if i == throughput_gate_index {
                    AutomaticInput::new(vec![HIGH, HIGH, LOW, LOW, HIGH, HIGH, LOW, LOW], 1, "Start_Throughput")
                } else {
                    AutomaticInput::new(vec![HIGH, LOW, HIGH, LOW, HIGH, LOW, HIGH, LOW], 1, "Start_Normal")
                };

            println!("AutomaticInput input id {} created", input_gate.borrow_mut().get_unique_id().id());

            let controlled_buffer = ControlledBuffer::new(1);
            println!("ControlledBuffer id {} created", controlled_buffer.borrow_mut().get_unique_id().id());

            input_gate.borrow_mut().connect_output_to_next_gate(
                0,
                0,
                controlled_buffer.clone(),
            );

            controlled_buffer.borrow_mut().connect_output_to_next_gate(
                0,
                0,
                output_gate.clone(),
            );

            let enable_index = controlled_buffer.borrow_mut().get_index_from_tag("E");
            if i == throughput_gate_index {
                single_enable_input_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    enable_index,
                    controlled_buffer.clone(),
                );
            } else {
                let next_index =
                    if i > throughput_gate_index {
                        i - 1
                    } else {
                        i
                    };
                other_enable_input_gates.borrow_mut().connect_output_to_next_gate(
                    next_index,
                    enable_index,
                    controlled_buffer.clone(),
                );
            }

            input_gates.push(input_gate);
        }

        input_gates.push(single_enable_input_gate);
        input_gates.push(other_enable_input_gates);
        output_gates.push(output_gate);

        let mut collected_output: Vec<Vec<Signal>> = Vec::new();
        let mut propagate_signal_through_circuit = true;
        let mut continue_clock = true;

        while continue_clock {
            unsafe {
                CLOCK_TICK_NUMBER += 1;
            }

            continue_clock = run_circuit(
                &input_gates,
                &output_gates,
                propagate_signal_through_circuit,
                &mut |_clock_tick_inputs, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                    collect_output_for_run_circuit(&mut collected_output, &output_gates);
                },
            );

            propagate_signal_through_circuit = false;
        }

        assert_eq!(collected_output, output_signal);
    }

    #[test]
    #[should_panic]
    fn test_controlled_buffer_multiple_inputs() {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        let enable_input_gate = AutomaticInput::new(vec![HIGH], 2, "Enable_Inputs");
        let input_gate = AutomaticInput::new(vec![HIGH], 2, "Inputs");
        let output_gate = SimpleOutput::new("OUT");

        let controlled_buffers: [Rc<RefCell<ControlledBuffer>>; 2] = [ControlledBuffer::new(1), ControlledBuffer::new(1)];

        let output_signal: Vec<Vec<Signal>> = Vec::new();

        for i in 0..2 {
            input_gate.borrow_mut().connect_output_to_next_gate(
                i,
                0,
                controlled_buffers[i].clone(),
            );

            let enable_index = controlled_buffers[i].borrow_mut().get_index_from_tag("E");
            enable_input_gate.borrow_mut().connect_output_to_next_gate(
                i,
                enable_index,
                controlled_buffers[i].clone(),
            );

            controlled_buffers[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                output_gate.clone(),
            );
        }

        input_gates.push(enable_input_gate);
        input_gates.push(input_gate);
        output_gates.push(output_gate);

        let mut collected_output: Vec<Vec<Signal>> = Vec::new();

        run_circuit(
            &input_gates,
            &output_gates,
            false,
            &mut |_clock_tick_inputs, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                collect_output_for_run_circuit(&mut collected_output, &output_gates);
            },
        );

        assert_eq!(collected_output, output_signal);
    }

    #[test]
    fn test_controlled_buffer_nested_in_complex_gate() {
        struct ControlledBufferWrapper {
            complex_gate: ComplexGateMembers,
            controlled_buffer: Rc<RefCell<ControlledBuffer>>,
        }

        impl ControlledBufferWrapper {
            pub fn new() -> Rc<RefCell<Self>> {
                let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
                let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
                let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

                let output_gate = SimpleOutput::new("Output");

                input_gates.push(SimpleInput::new(1, "Input"));
                output_gates.push(output_gate.clone());
                output_gates_logic.push(output_gate);

                let mut gate = ControlledBufferWrapper {
                    complex_gate: ComplexGateMembers::new(
                        1,
                        1,
                        GateType::UnknownType,
                        input_gates,
                        output_gates,
                    ),
                    controlled_buffer: ControlledBuffer::new(1),
                };

                gate.build_and_prime_circuit(output_gates_logic);

                Rc::new(RefCell::new(gate))
            }

            fn build_and_prime_circuit(
                &mut self,
                output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
            ) {
                self.complex_gate.input_gates[self.get_index_from_tag("Input")]
                    .borrow_mut()
                    .connect_output_to_next_gate(
                        0,
                        0,
                        self.controlled_buffer.clone(),
                    );

                self.controlled_buffer.borrow_mut().connect_output_to_next_gate(
                    0,
                    0,
                    output_gates[0].clone(),
                );

                //Force the enable to low so that NONE is always returned.
                let enable_index = self.controlled_buffer.borrow_mut().get_index_from_tag("E");
                self.controlled_buffer.borrow_mut().update_input_signal(
                    GateInput::new(
                        enable_index,
                        LOW,
                        UniqueID::zero_id(),
                    )
                );

                //Prime gates
                self.complex_gate.calculate_output_from_inputs(true);
            }
        }

        impl LogicGate for ControlledBufferWrapper {
            fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
                self.complex_gate.connect_output_to_next_gate(
                    self.get_unique_id(),
                    current_gate_output_key,
                    next_gate_input_key,
                    next_gate,
                );
            }

            fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
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

            fn get_tag(&self) -> String {
                self.complex_gate.simple_gate.tag.clone()
            }

            fn set_tag(&mut self, tag: &str) {
                self.complex_gate.simple_gate.tag = tag.to_string()
            }

            fn get_index_from_tag(&self, tag: &str) -> usize {
                self.complex_gate.get_index_from_tag(tag)
            }

            fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
                self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
            }
        }

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        let input_gate = AutomaticInput::new(vec![HIGH], 2, "Inputs");
        let output_gate = SimpleOutput::new("OUT");

        let wrapper = ControlledBufferWrapper::new();

        let output_signal = vec![[NONE]];

        input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            wrapper.clone(),
        );

        wrapper.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        input_gates.push(input_gate);
        output_gates.push(output_gate);

        let mut collected_output: Vec<Vec<Signal>> = Vec::new();

        run_circuit(
            &input_gates,
            &output_gates,
            false,
            &mut |_clock_tick_inputs, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                collect_output_for_run_circuit(&mut collected_output, &output_gates);
            },
        );

        assert_eq!(collected_output, output_signal);
    }

    #[test]
    fn splitter_properly_splits() {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        let input_num = rand::thread_rng().gen_range(1..=16);
        let outputs_per_input = rand::thread_rng().gen_range(2..=16);

        let splitter = Splitter::new(input_num, outputs_per_input);

        let mut output_signal = Vec::new();
        let mut single_turn_output = Vec::new();
        for i in 0..input_num {
            let signal_num = rand::thread_rng().gen_range(0..=2);
            let signal = match signal_num {
                0 => LOW,
                1 => HIGH,
                _ => NONE,
            };
            let input_tag = format!("IN_{}", i);
            let input_gate = AutomaticInput::new(vec![signal.clone()], 1, input_tag.as_str());

            for _ in 0..outputs_per_input {
                single_turn_output.push(signal.clone());
            }

            input_gate.borrow_mut().connect_output_to_next_gate(
                0,
                i,
                splitter.clone(),
            );

            input_gates.push(input_gate);
        }

        output_signal.push(single_turn_output);

        for i in 0..(input_num * outputs_per_input) {
            let output_tag = format!("OUT_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());

            splitter.borrow_mut().connect_output_to_next_gate(
                i,
                0,
                output_gate.clone(),
            );

            output_gates.push(output_gate);
        }

        let mut collected_output: Vec<Vec<Signal>> = Vec::new();

        run_circuit(
            &input_gates,
            &output_gates,
            false,
            &mut |_clock_tick_inputs, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                assert_eq!(output_gates.len(), input_num * outputs_per_input);

                let mut single_collected_output = Vec::new();
                collect_outputs_from_output_gates(&output_gates, &mut single_collected_output);

                collected_output.push(single_collected_output);
            },
        );

        assert_eq!(collected_output, output_signal);
    }
}

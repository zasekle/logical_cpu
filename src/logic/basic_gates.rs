use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, BasicGateMembers, InputSignalReturn, ConnectedOutput, calculate_input_signals_from_all_inputs};
use crate::logic::foundations::Signal::{HIGH, NONE};

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
        // println!("ControlledBuffer input: {:#?}", input);
        //TODO: What is happening right now is that on round one, the controlled_buffer is being updated
        // to HIGH. Then it is updated to HIGH again on the next round and so it doesn't propagate b/c
        // this returns false. So only the second controlled_buffer propagates and the output ends up
        // as NONE b/c only NONE was passed (and it was one the first clock tick).
        // The question is, will this be a problem elsewhere or just on the controlled buffer that
        // initiates the problem? So lets take a similar situation and say that RAM_1 has the
        // output of HIGH and RAM_2 has the output of NONE. Next round RAM_1 doesn't change, only one
        // RAM can be active at a time, so this will work just fine.
        // So lets say two different signals go into an AND gate input. The is HIGH

        //TODO: Maybe another solution is that I could set up the gates to take a variety of inputs
        // maybe congregate them somehow at the same index, or at a different index and then iterate
        // through them and if I get a bunch of NONE and a single input I can use it, if I get more
        // than one HIGH or LOW I can panic.

        //TODO: Probably undo this input signal stuff here.
        InputSignalReturn {
            changed_count_this_tick: self.members.update_input_signal(input).changed_count_this_tick,
            input_signal_updated: true
        }
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

            let output_clone = self.members.output_states.clone();

            output_clone
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
    use crate::globals::CLOCK_TICK_NUMBER;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::{run_circuit, start_clock};
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

        let num_gates = 2;
        let throughput_gate_index = 1;

        let output_gate = SimpleOutput::new("OUT");
        output_gate.borrow_mut().toggle_output_printing(true); //TODO: delete me

        let single_enable_input_gate = AutomaticInput::new(vec![HIGH, HIGH], 1, "Single_Enable");
        //If this value goes high, the value will be unpredictable.
        let other_enable_input_gates = AutomaticInput::new(vec![LOW; 2], num_gates - 1, "Other_Enable");
        // let single_enable_input_gate = AutomaticInput::new(vec![LOW, LOW, LOW, LOW, HIGH, HIGH, HIGH, HIGH], 1, "Single_Enable");
        // //If this value goes high, the value will be unpredictable.
        // let other_enable_input_gates = AutomaticInput::new(vec![LOW; 8], num_gates - 1, "Other_Enable");

        let output_signal = vec![[HIGH], [HIGH]]; //TODO
        // let output_signal = vec![[NONE], [NONE], [NONE], [NONE], [HIGH], [HIGH], [LOW], [LOW]]; //TODO

        for i in 0..num_gates {
            let input_gate =
                if i == throughput_gate_index {
                    AutomaticInput::new(vec![HIGH, HIGH], 1, "Start_Throughput")
                    // AutomaticInput::new(vec![HIGH, HIGH, LOW, LOW, HIGH, HIGH, LOW, LOW], 1, "Start_Throughput")
                } else {
                    AutomaticInput::new(vec![HIGH, LOW], 1, "Start_Normal")
                    // AutomaticInput::new(vec![HIGH, LOW, HIGH, LOW, HIGH, LOW, HIGH, LOW], 1, "Start_Normal")
                };

            let controlled_buffer = ControlledBuffer::new(1);

            let mut mut_controlled_buffer = controlled_buffer.borrow_mut();
            input_gate.borrow_mut().connect_output_to_next_gate(
                0,
                0,
                controlled_buffer.clone(),
            );

            mut_controlled_buffer.connect_output_to_next_gate(
                0,
                0,
                output_gate.clone(),
            );

            mut_controlled_buffer.toggle_output_printing(true); //TODO

            let enable_index = mut_controlled_buffer.get_index_from_tag("E");
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
                    assert_eq!(output_gates.len(), 1);

                    let mut single_collected_output = Vec::new();
                    for output in output_gates.iter() {
                        let mut output = output.borrow_mut();

                        let output = output.fetch_output_signals().unwrap();

                        assert_eq!(output.len(), 1);

                        let output = output.first().unwrap();

                        match output {
                            GateOutputState::NotConnected(signal) => {
                                single_collected_output.push(signal.clone());
                            }
                            GateOutputState::Connected(_) => panic!("Final output gate should not be connected"),
                        }
                    }

                    collected_output.push(single_collected_output);
                },
            );

            propagate_signal_through_circuit = false;
        }

        assert_eq!(collected_output, output_signal);
    }

    //TODO: need to make sure that the NONE type won't work if two connected (might want to make this a controlled buffer test)
}
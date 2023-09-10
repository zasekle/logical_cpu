use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, Signal, InputSignalReturn, BasicGateMembers};
use crate::logic::foundations::{Signal::{HIGH, LOW}};

pub struct Clock {
    output_states: Vec<GateOutputState>,
    unique_id: UniqueID,
    should_print_output: bool,
    gate_type: GateType,
    tag: String,
}

#[allow(dead_code)]
impl Clock {
    pub fn new(output_num: usize, tag: &str) -> Rc<RefCell<Self>> {
        let mut clock = Clock {
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            should_print_output: false,
            gate_type: GateType::Clock,
            tag: String::from(tag),
        };

        clock.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(HIGH),
        );

        Rc::new(RefCell::new(clock))
    }
}

impl LogicGate for Clock {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        GateLogic::connect_output_to_next_gate(
            self.gate_type,
            None,
            &mut self.output_states,
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, _input: GateInput) -> InputSignalReturn {
        //TODO: This needs to handle oscillation properly and return a formatted count tick.
        // self.members.update_input_signal(input)

        InputSignalReturn {
            changed_count_this_tick: 0,
            input_signal_updated: false,
        }
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals(
            &self.gate_type,
            None,
            &mut self.output_states,
            self.unique_id,
            self.should_print_output,
        )
    }

    fn get_gate_type(&self) -> GateType {
        self.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.should_print_output = print_output;
    }

    fn get_tag(&self) -> String {
        self.tag.clone()
    }

    fn is_input_gate(&self) -> bool {
        true
    }
}

pub struct AutomaticInput {
    values_to_be_output: Vec<Signal>,
    output_states: Vec<GateOutputState>,
    unique_id: UniqueID,
    should_print_output: bool,
    gate_type: GateType,
    tag: String,
}

#[allow(dead_code)]
impl AutomaticInput {
    pub fn new(values_to_be_output: Vec<Signal>, output_num: usize, tag: &str) -> Rc<RefCell<Self>> {
        let mut automatic_input = AutomaticInput {
            values_to_be_output,
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            should_print_output: false,
            gate_type: GateType::AutomaticInput,
            tag: String::from(tag),
        };

        automatic_input.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(HIGH),
        );

        Rc::new(RefCell::new(automatic_input))
    }
}

impl LogicGate for AutomaticInput {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_index: usize,
        next_gate_input_index: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        GateLogic::connect_output_to_next_gate(
            self.gate_type,
            Some(&self.values_to_be_output),
            &mut self.output_states,
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //TODO: This needs to handle oscillation properly and return a formatted count tick.
        // self.members.update_input_signal(input)
        self.values_to_be_output.push(input.signal);

        InputSignalReturn {
            changed_count_this_tick: 0,
            input_signal_updated: false,
        }
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        if let Some(_) = self.values_to_be_output.get(0) {
            let result = GateLogic::fetch_output_signals(
                &self.gate_type,
                Some(&self.values_to_be_output),
                &mut self.output_states,
                self.unique_id,
                self.should_print_output,
            );

            self.values_to_be_output.remove(0);
            result
        } else {
            Err(GateLogicError::NoMoreAutomaticInputsRemaining)
        }
    }

    fn get_gate_type(&self) -> GateType {
        self.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.should_print_output = print_output;
    }

    fn get_tag(&self) -> String {
        self.tag.clone()
    }

    fn is_input_gate(&self) -> bool {
        true
    }
}

pub struct SimpleInput {
    members: BasicGateMembers,
    tag: String,
}

#[allow(dead_code)]
impl SimpleInput {
    pub fn new(tag: &str) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                SimpleInput {
                    members: BasicGateMembers::new(
                        1,
                        1,
                        GateType::SimpleInput,
                        Some(LOW),
                    ),
                    tag: String::from(tag),
                }
            )
        )
    }
}

impl LogicGate for SimpleInput {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_index: usize,
        next_gate_input_index: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        GateLogic::connect_output_to_next_gate(
            self.members.gate_type,
            Some(&self.members.input_signals),
            &mut self.members.output_states,
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals(
            &self.members.gate_type,
            Some(&self.members.input_signals),
            &mut self.members.output_states,
            self.members.unique_id,
            self.members.should_print_output,
        )
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
        self.tag.clone()
    }

    fn is_input_gate(&self) -> bool {
        true
    }
}

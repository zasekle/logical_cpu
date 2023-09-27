use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, Signal, InputSignalReturn, BasicGateMembers};
use crate::logic::foundations::{Signal::{HIGH, LOW_}};

pub struct Clock {
    output_states: Vec<GateOutputState>,
    unique_id: UniqueID,
    should_print_output: bool,
    gate_type: GateType,
    tag: String,
    previous_signal: Vec<Signal>,
}

#[allow(dead_code)]
impl Clock {
    pub fn new(output_num: usize, tag: &str) -> Rc<RefCell<Self>> {
        let mut clock = Clock {
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            should_print_output: false,
            gate_type: GateType::ClockType,
            tag: String::from(tag),
            previous_signal: vec![HIGH],
        };

        clock.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(HIGH),
        );

        Rc::new(RefCell::new(clock))
    }

    fn get_formatted_input(&self) -> Vec<HashMap<UniqueID, Signal>> {
        self.previous_signal
            .iter()
            .map(|val| {
                let mut map = HashMap::new();
                map.insert(self.unique_id, val.clone());
                map
            })
            .collect()
    }
}

impl LogicGate for Clock {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>
    ) {
        GateLogic::connect_output_to_next_gate(
            self.gate_type,
            self.unique_id,
            &self.get_formatted_input(),
            &mut self.output_states,
            current_gate_output_key,
            &self.tag,
            next_gate_input_key,
            next_gate,
            self.should_print_output,
        );
    }

    fn update_input_signal(&mut self, _input: GateInput) -> InputSignalReturn {
        //Want to return 1 here because run_circuit expects it.
        InputSignalReturn {
            changed_count_this_tick: 1,
            input_signal_updated: false,
        }
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        //TODO: Does this need to save the new clock tick to the index 0 of `previous signal`?
        GateLogic::fetch_output_signals(
            &self.gate_type,
            &self.get_formatted_input(),
            &mut self.output_states,
            self.unique_id,
            self.should_print_output,
            self.tag.as_str(),
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

    fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string()
    }

    fn is_input_gate(&self) -> bool {
        true
    }

    fn internal_update_index_to_id(&mut self, _sending_id: UniqueID, _gate_input_index: usize, _signal: Signal) {}
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
            gate_type: GateType::AutomaticInputType,
            tag: String::from(tag),
        };

        automatic_input.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(HIGH),
        );

        Rc::new(RefCell::new(automatic_input))
    }

    fn get_formatted_input(&self) -> Vec<HashMap<UniqueID, Signal>> {
        self.values_to_be_output
            .iter()
            .map(|val| {
                let mut map = HashMap::new();
                map.insert(self.unique_id, val.clone());
                map
            })
            .collect()
    }
}

impl LogicGate for AutomaticInput {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        let mut values_to_be_output = self.get_formatted_input();
        GateLogic::connect_output_to_next_gate(
            self.gate_type,
            self.unique_id,
            &mut values_to_be_output,
            &mut self.output_states,
            current_gate_output_key,
            &self.tag,
            next_gate_input_key,
            next_gate,
            self.should_print_output,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //This doesn't ever actually 'change' input. So there is no reason to update oscillation.
        // New inputs are simply pushed into the back of the vector.
        self.values_to_be_output.push(input.signal);

        //Want to return 1 here because run_circuit expects it.
        InputSignalReturn {
            changed_count_this_tick: 1,
            input_signal_updated: true,
        }
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        if let Some(_) = self.values_to_be_output.get(0) {
            let values_to_be_output = self.get_formatted_input();

            let result = GateLogic::fetch_output_signals(
                &self.gate_type,
                &values_to_be_output,
                &mut self.output_states,
                self.unique_id,
                self.should_print_output,
                self.tag.as_str(),
            );

            // println!("AutomaticInput id {} fetch_output \n{:#?}", self.unique_id.id(), result);

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

    fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string()
    }

    fn is_input_gate(&self) -> bool {
        true
    }

    fn internal_update_index_to_id(&mut self, _sending_id: UniqueID, _gate_input_index: usize, _signal: Signal) {}
}

pub struct SimpleInput {
    members: BasicGateMembers,
    tag: String,
}

#[allow(dead_code)]
impl SimpleInput {
    pub fn new(output_num: usize, tag: &str) -> Rc<RefCell<Self>> {
        assert_ne!(output_num, 0);

        Rc::new(
            RefCell::new(
                SimpleInput {
                    members: BasicGateMembers::new(
                        1,
                        output_num,
                        GateType::SimpleInputType,
                        Some(LOW_),
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
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        GateLogic::connect_output_to_next_gate(
            self.members.gate_type,
            self.members.unique_id,
            &mut self.members.input_signals,
            &mut self.members.output_states,
            current_gate_output_key,
            &self.tag,
            next_gate_input_key,
            next_gate,
            self.members.should_print_output,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals(
            &self.members.gate_type,
            &self.members.input_signals,
            &mut self.members.output_states,
            self.members.unique_id,
            self.members.should_print_output,
            self.tag.as_str(),
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

    fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string()
    }

    fn is_input_gate(&self) -> bool {
        true
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

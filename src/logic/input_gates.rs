use std::collections::HashMap;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, Signal, InputSignalReturn, BasicGateMembers, ConnectedOutput, set_all_gate_output_to_signal};
use crate::logic::foundations::{Signal::{HIGH, LOW_}};
use crate::shared_mutex::{new_shared_mutex, SharedMutex};

pub struct Clock {
    output_states: Vec<GateOutputState>,
    unique_id: UniqueID,
    should_print_output: bool,
    print_each_input_output_gate: bool,
    gate_type: GateType,
    tag: String,
    previous_signal: Signal,
}

#[allow(dead_code)]
impl Clock {
    pub fn new(output_num: usize, tag: &str) -> SharedMutex<Self> {
        assert_ne!(output_num, 0);
        let mut clock = Clock {
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            should_print_output: false,
            print_each_input_output_gate: true,
            gate_type: GateType::ClockType,
            tag: String::from(tag),
            previous_signal: LOW_,
        };

        clock.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(LOW_),
        );

        new_shared_mutex(clock)
    }

    fn get_formatted_input(&self) -> Vec<HashMap<UniqueID, Signal>> {
        vec![
            HashMap::from([(self.unique_id, self.previous_signal.clone())])
        ]
    }

    //Note that this function makes calls to borrow_mut(). Therefore it cannot be used while running
    // the circuit, only before or after.
    pub fn disconnect_gate(
        &mut self,
        current_output_index: usize,
    ) {
        disconnect_gate(
            current_output_index,
            &mut self.output_states,
            &self.gate_type,
            self.unique_id.clone(),
            self.tag.as_str(),
        );
    }

    pub fn set_clock_state(&mut self, clock_signal: Signal) {
        self.previous_signal = clock_signal.clone();

        set_all_gate_output_to_signal(
            &mut self.output_states,
            clock_signal.clone(),
        );
    }
}

impl LogicGate for Clock {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: SharedMutex<dyn LogicGate>,
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
        let input_signals = &self.get_formatted_input();
        let fetch_result = GateLogic::fetch_output_signals(
            &self.gate_type,
            &input_signals,
            &mut self.output_states,
            self.unique_id,
            self.should_print_output,
            self.print_each_input_output_gate,
            self.tag.as_str(),
        );

        let output_signal = GateLogic::calculate_output_from_inputs(
            &self.gate_type,
            &input_signals,
        )?;

        self.previous_signal = output_signal;

        fetch_result
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        panic!("Clock never has any input. Passed id {}, passed index {}", connected_id.id(), input_index);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.print_each_input_output_gate = print_each_input_output_gate;
    }
}

pub struct AutomaticInput {
    values_to_be_output: Vec<Signal>,
    output_states: Vec<GateOutputState>,
    unique_id: UniqueID,
    should_print_output: bool,
    print_each_input_output_gate: bool,
    gate_type: GateType,
    tag: String,
}

#[allow(dead_code)]
impl AutomaticInput {
    pub fn new(values_to_be_output: Vec<Signal>, output_num: usize, tag: &str) -> SharedMutex<Self> {
        let mut automatic_input = AutomaticInput {
            values_to_be_output,
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            should_print_output: false,
            print_each_input_output_gate: true,
            gate_type: GateType::AutomaticInputType,
            tag: String::from(tag),
        };

        automatic_input.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(HIGH),
        );

        new_shared_mutex(automatic_input)
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

    //Note that this function makes calls to borrow_mut(). Therefore it cannot be used while running
    // the circuit, only before or after.
    pub fn disconnect_gate(
        &mut self,
        current_output_index: usize,
    ) {
        disconnect_gate(
            current_output_index,
            &mut self.output_states,
            &self.gate_type,
            self.unique_id.clone(),
            self.tag.as_str(),
        );
    }
}

impl LogicGate for AutomaticInput {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: SharedMutex<dyn LogicGate>,
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
                self.print_each_input_output_gate,
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        panic!("AutomaticInput never has any input. Passed id {}, passed index {}", connected_id.id(), input_index);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.print_each_input_output_gate = print_each_input_output_gate;
    }
}

pub struct SimpleInput {
    members: BasicGateMembers,
    tag: String,
}

#[allow(dead_code)]
impl SimpleInput {
    pub fn new(output_num: usize, tag: &str) -> SharedMutex<Self> {
        assert_ne!(output_num, 0);

        new_shared_mutex(
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
    }
}

impl LogicGate for SimpleInput {
    fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: SharedMutex<dyn LogicGate>,
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
            self.members.print_each_input_output_gate,
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(
            input_index, connected_id,
        );
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }
}

fn disconnect_gate(
    current_output_index: usize,
    output_states: &mut Vec<GateOutputState>,
    gate_type: &GateType,
    unique_id: UniqueID,
    tag: &str,
) {
    let next_gate_info: ConnectedOutput;
    if let Some(output_state) = output_states.get(current_output_index) {
        match output_state {
            GateOutputState::NotConnected(_) => {
                panic!(
                    "When attempting to disconnect a gate, the gate with type {} id {} tag {} was not connected.",
                    gate_type,
                    unique_id.id(),
                    tag
                )
            }
            GateOutputState::Connected(connected_output) => {
                next_gate_info = connected_output.clone();
            }
        }
    } else {
        panic!(
            "When attempting to disconnect a gate, the gate with type {} id {} tag {} was not connected.",
            gate_type,
            unique_id.id(),
            tag
        )
    }

    next_gate_info.gate.lock().unwrap().remove_connected_input(
        next_gate_info.throughput.input_index, unique_id,
    );

    output_states[current_output_index] = GateOutputState::NotConnected(next_gate_info.throughput.signal);
}

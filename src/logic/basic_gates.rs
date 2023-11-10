use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, BasicGateMembers, InputSignalReturn, ConnectedOutput, calculate_input_signals_from_all_inputs, Signal, calculate_input_signal_from_single_inputs};
use crate::logic::foundations::Signal::{HIGH, LOW_, NONE};
use crate::shared_mutex::{new_shared_mutex, SharedMutex};

pub struct Or {
    pub members: BasicGateMembers,
}

#[allow(dead_code)]
impl Or {
    pub fn new(input_num: usize, output_num: usize) -> SharedMutex<Self> {
        let or_gate = Or {
            members: BasicGateMembers::new(
                input_num,
                output_num,
                GateType::OrType,
                0,
                None,
            )
        };
        new_shared_mutex(
            or_gate.get_unique_id().id(),
            or_gate,
        )
    }
}

impl LogicGate for Or {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.members.connect_output(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_calculate_basic_gate(&mut self.members)
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_no_calculate_basic_gate(&mut self.members)
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

pub struct And {
    pub members: BasicGateMembers,
}

#[allow(dead_code)]
impl And {
    pub fn new(input_num: usize, output_num: usize) -> SharedMutex<Self> {
        let and_gate = And {
            members: BasicGateMembers::new(
                input_num,
                output_num,
                GateType::AndType,
                0,
                None,
            )
        };
        new_shared_mutex(
            and_gate.get_unique_id().id(),
            and_gate,
        )
    }
}

impl LogicGate for And {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.members.connect_output(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_calculate_basic_gate(&mut self.members)
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_no_calculate_basic_gate(&mut self.members)
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
       self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

pub struct Not {
    pub members: BasicGateMembers,
}

#[allow(dead_code)]
impl Not {
    pub fn new(output_num: usize) -> SharedMutex<Self> {
        let not_gate = Not {
            members: BasicGateMembers::new(
                1,
                output_num,
                GateType::NotType,
                0,
                None,
            )
        };
        new_shared_mutex(
            not_gate.get_unique_id().id(),
            not_gate,
        )
    }
}

impl LogicGate for Not {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.members.connect_output(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_calculate_basic_gate(&mut self.members)
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_no_calculate_basic_gate(&mut self.members)
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

pub struct Nor {
    pub members: BasicGateMembers,
}

#[allow(dead_code)]
impl Nor {
    pub fn new(input_num: usize, output_num: usize) -> SharedMutex<Self> {
        let nor_gate = Nor {
            members: BasicGateMembers::new(
                input_num,
                output_num,
                GateType::NorType,
                0,
                None,
            )
        };
        new_shared_mutex(
            nor_gate.get_unique_id().id(),
            nor_gate,
        )
    }
}

impl LogicGate for Nor {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.members.connect_output(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_calculate_basic_gate(&mut self.members)
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_no_calculate_basic_gate(&mut self.members)
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

pub struct Nand {
    pub members: BasicGateMembers,
}

#[allow(dead_code)]
impl Nand {
    pub fn new(input_num: usize, output_num: usize) -> SharedMutex<Self> {
        let nand_gate = Nand {
            members: BasicGateMembers::new(
                input_num,
                output_num,
                GateType::NandType,
                0,
                None,
            )
        };
        new_shared_mutex(
            nand_gate.get_unique_id().id(),
            nand_gate,
        )
    }
}

impl LogicGate for Nand {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.members.connect_output(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_calculate_basic_gate(&mut self.members)
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_no_calculate_basic_gate(&mut self.members)
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

pub struct XOr {
    pub members: BasicGateMembers,
}

impl XOr {
    pub fn new(input_num: usize, output_num: usize) -> SharedMutex<Self> {
        let xor_gate = XOr {
            members: BasicGateMembers::new(
                input_num,
                output_num,
                GateType::XOrType,
                0,
                None,
            )
        };
        new_shared_mutex(
            xor_gate.get_unique_id().id(),
            xor_gate,
        )
    }
}

impl LogicGate for XOr {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.members.connect_output(
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_calculate_basic_gate(&mut self.members)
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_no_calculate_basic_gate(&mut self.members)
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

pub struct Splitter {
    pub members: BasicGateMembers,
    outputs_per_input: usize,
    pull_output: Option<Signal>,
}

#[allow(dead_code)]
impl Splitter {
    pub fn new(input_num: usize, outputs_per_input: usize) -> SharedMutex<Self> {
        assert_ne!(outputs_per_input, 0);
        let splitter = Splitter {
            members: BasicGateMembers::new(
                input_num,
                input_num * outputs_per_input,
                GateType::SplitterType,
                0,
                Some(LOW_),
            ),
            outputs_per_input,
            pull_output: None,
        };
        new_shared_mutex(
            splitter.get_unique_id().id(),
            splitter,
        )
    }

    pub fn get_index_for_output(&self, input_index: usize, index_of_output: usize) -> usize {
        assert!(input_index < self.members.input_signals.len());
        assert!(index_of_output < self.outputs_per_input);

        input_index * self.outputs_per_input + index_of_output
    }

    pub fn pull_output(&mut self, signal: Signal) {
        self.pull_output = Some(signal);
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        //output_states is outputs_per_input*num_inputs length and input_states is num_inputs length.
        let input_signals = calculate_input_signals_from_all_inputs(&self.members.input_signals)?;

        for (i, output) in self.members.output_states.iter_mut().enumerate() {

            // 25 % 8 = 1
            let idx = i / self.outputs_per_input;
            let input_signal = input_signals[idx].clone();

            let input_signal =
                if input_signal == NONE {
                    if let Some(signal) = self.pull_output.clone() {
                        signal
                    } else {
                        NONE
                    }
                } else {
                    input_signal
                };
            match output {
                GateOutputState::NotConnected(signal) => {
                    *signal = input_signal;
                }
                GateOutputState::Connected(connected_output) => {
                    connected_output.throughput.signal = input_signal;
                }
            }
        }

        if self.members.should_print_output {
            GateLogic::print_gate_output(
                &self.members.gate_type,
                &self.members.unique_id,
                &String::from(""),
                &self.members.input_signals,
                &self.members.output_states,
            );
        }

        Ok(self.members.output_states.clone())
    }
}

impl LogicGate for Splitter {
    //current_gate_output_key is meant to be extracted from Splitter::get_index_for_output()
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        //When gates are being connected, there should be no issues with this error.
        let output_signal = calculate_input_signal_from_single_inputs(
            &self.members.input_signals[current_gate_output_key / self.outputs_per_input]
            // &self.members.input_signals[current_gate_output_key % self.members.input_signals.len()]
        ).unwrap();

        GateLogic::connect_output_no_calculate(
            self.get_unique_id(),
            &mut self.members.output_states,
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
            output_signal.clone(),
            self.members.gate_type,
            &self.members.tag,
            self.members.should_print_output,
        );

        output_signal
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.fetch_output_signals()
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.fetch_output_signals()
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
        self.members.get_index_from_tag(tag)
    }

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

pub struct ControlledBuffer {
    pub members: BasicGateMembers,
}

#[allow(dead_code)]
impl ControlledBuffer {
    pub fn new(input_output_num: usize) -> SharedMutex<Self> {
        let controlled_buffer = ControlledBuffer {
            members: BasicGateMembers::new(
                input_output_num + 1,
                input_output_num,
                GateType::ControlledBufferType,
                0,
                Some(NONE),
            )
        };
        new_shared_mutex(
            controlled_buffer.get_unique_id().id(),
            controlled_buffer,
        )
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        //When input index 0 is HIGH, allow the signal through, otherwise return NotConnected.
        let input_signals = calculate_input_signals_from_all_inputs(&self.members.input_signals)?;
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
                &self.members.input_signals,
                &output,
            );
        }

        Ok(output)
    }
}

impl LogicGate for ControlledBuffer {
    fn internal_connect_output(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: SharedMutex<dyn LogicGate>,
    ) -> Signal {
        let enable_index = self.get_index_from_tag("E");
        //When gates are being connected, there should be no issues with this error.
        let input_signals =
            calculate_input_signals_from_all_inputs(&self.members.input_signals).unwrap();
        let output_signal = if input_signals[enable_index] == HIGH {
            input_signals[current_gate_output_key].clone()
        } else {
            NONE
        };

        GateLogic::connect_output_no_calculate(
            self.get_unique_id(),
            &mut self.members.output_states,
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
            output_signal.clone(),
            self.members.gate_type,
            &self.members.tag,
            self.members.should_print_output,
        );

        output_signal
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.members.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.fetch_output_signals()
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.fetch_output_signals()
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
            self.members.get_index_from_tag(tag)
        }
    }

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.members.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.members.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.members.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        panic!("Basic gates do not have input gates");
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::globals::CLOCK_TICK_NUMBER;
    use crate::logic::foundations::{ComplexGateMembers, connect_gates, Signal};
    use crate::logic::foundations::Signal::{HIGH, LOW_};
    use crate::logic::input_gates::{AutomaticInput, SimpleInput};
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::{collect_outputs_from_output_gates, test_simple_gate};
    use super::*;

    fn test_controlled_buffer(
        signal: Signal
    ) {
        let output_gate = SimpleOutput::new("OUT");
        let controlled_buffer = ControlledBuffer::new(1);

        connect_gates(
            controlled_buffer.clone(),
            0,
            output_gate.clone(),
            0,
        );

        controlled_buffer.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                signal.clone(),
                UniqueID::zero_id(),
            )
        );

        let enable_index = controlled_buffer.lock().unwrap().get_index_from_tag("E");
        controlled_buffer.lock().unwrap().update_input_signal(
            GateInput::new(
                enable_index,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        let output = controlled_buffer.lock().unwrap().fetch_output_signals_calculate().unwrap();

        for gate_output_state in output {
            match gate_output_state {
                GateOutputState::NotConnected(_) => panic!("Output should be connected when pin is low."),
                GateOutputState::Connected(connected_output) => {
                    assert_eq!(connected_output.throughput.signal, signal);
                    let connected_id = connected_output.gate.lock().unwrap().get_unique_id().id();
                    let output_id = output_gate.lock().unwrap().get_unique_id().id();
                    assert_eq!(connected_id, output_id);
                }
            }
        }
    }

    fn collect_output_for_run_circuit(collected_output: &mut Vec<Vec<Signal>>, output_gates: &&Vec<SharedMutex<dyn LogicGateAndOutputGate>>) {
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
            LOW_,
            Some(LOW_),
            LOW_,
        );
    }

    #[test]
    fn test_or_gate_low_high() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            LOW_,
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
            Some(LOW_),
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
            LOW_,
            Some(LOW_),
            LOW_,
        );
    }

    #[test]
    fn test_and_gate_low_high() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            LOW_,
            Some(HIGH),
            LOW_,
        );
    }

    #[test]
    fn test_and_gate_high_low() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            HIGH,
            Some(LOW_),
            LOW_,
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
            LOW_,
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
            LOW_,
        );
    }

    #[test]
    fn test_nor_gate_low_low() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            LOW_,
            Some(LOW_),
            HIGH,
        );
    }

    #[test]
    fn test_nor_gate_low_high() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            LOW_,
            Some(HIGH),
            LOW_,
        );
    }

    #[test]
    fn test_nor_gate_high_low() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            HIGH,
            Some(LOW_),
            LOW_,
        );
    }

    #[test]
    fn test_nor_gate_high_high() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            HIGH,
            Some(HIGH),
            LOW_,
        );
    }

    #[test]
    fn test_nand_gate_low_low() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            LOW_,
            Some(LOW_),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_low_high() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            HIGH,
            Some(LOW_),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_high_low() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            LOW_,
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
            LOW_,
        );
    }

    #[test]
    fn test_xor_gate_low_low() {
        let xor_gate = XOr::new(2, 1);

        test_simple_gate(
            xor_gate,
            LOW_,
            Some(LOW_),
            LOW_,
        );
    }

    #[test]
    fn test_xor_gate_low_high() {
        let xor_gate = XOr::new(2, 1);

        test_simple_gate(
            xor_gate,
            LOW_,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_xor_gate_high_low() {
        let xor_gate = XOr::new(2, 1);

        test_simple_gate(
            xor_gate,
            HIGH,
            Some(LOW_),
            HIGH,
        );
    }

    #[test]
    fn test_xor_gate_high_high() {
        let xor_gate = XOr::new(2, 1);

        test_simple_gate(
            xor_gate,
            HIGH,
            Some(HIGH),
            LOW_,
        );
    }

    #[test]
    fn test_controlled_buffer_initialization() {
        let output_gate = SimpleOutput::new("OUT");
        let controlled_buffer = ControlledBuffer::new(1);

        connect_gates(
            controlled_buffer.clone(),
            0,
            output_gate.clone(),
            0,
        );

        controlled_buffer.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        let output = controlled_buffer.lock().unwrap().fetch_output_signals_calculate().unwrap();

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
        test_controlled_buffer(LOW_);
    }

    #[test]
    fn test_none_signal_working() {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        let num_gates = rand::thread_rng().gen_range(2..=16);
        let throughput_gate_index = rand::thread_rng().gen_range(0..num_gates);

        let output_gate = SimpleOutput::new("OUT");

        let single_enable_input_gate = AutomaticInput::new(vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH], 1, "Single_Enable");
        // If this value goes high, the value will be unpredictable.
        let other_enable_input_gates = AutomaticInput::new(vec![LOW_; 8], num_gates - 1, "Other_Enable");

        let output_signal = vec![[NONE], [NONE], [NONE], [NONE], [HIGH], [HIGH], [LOW_], [LOW_]];

        for i in 0..num_gates {
            let input_gate =
                if i == throughput_gate_index {
                    AutomaticInput::new(vec![HIGH, HIGH, LOW_, LOW_, HIGH, HIGH, LOW_, LOW_], 1, "Start_Throughput")
                } else {
                    AutomaticInput::new(vec![HIGH, LOW_, HIGH, LOW_, HIGH, LOW_, HIGH, LOW_], 1, "Start_Normal")
                };

            let controlled_buffer = ControlledBuffer::new(1);

            connect_gates(
                input_gate.clone(),
                0,
                controlled_buffer.clone(),
                0,
            );

            connect_gates(
                controlled_buffer.clone(),
                0,
                output_gate.clone(),
                0,
            );

            let enable_index = controlled_buffer.lock().unwrap().get_index_from_tag("E");
            if i == throughput_gate_index {
                connect_gates(
                    single_enable_input_gate.clone(),
                    0,
                    controlled_buffer.clone(),
                    enable_index,
                );
            } else {
                let next_index =
                    if i > throughput_gate_index {
                        i - 1
                    } else {
                        i
                    };
                connect_gates(
                    other_enable_input_gates.clone(),
                    next_index,
                    controlled_buffer.clone(),
                    enable_index,
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
                &mut |_clock_tick_inputs, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
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
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        let enable_input_gate = AutomaticInput::new(vec![HIGH], 2, "Enable_Inputs");
        let input_gate = AutomaticInput::new(vec![HIGH], 2, "Inputs");
        let output_gate = SimpleOutput::new("OUT");

        let controlled_buffers: [SharedMutex<ControlledBuffer>; 2] = [ControlledBuffer::new(1), ControlledBuffer::new(1)];

        let output_signal: Vec<Vec<Signal>> = Vec::new();

        for i in 0..2 {
            connect_gates(
                input_gate.clone(),
                i,
                controlled_buffers[i].clone(),
                0,
            );

            let enable_index = controlled_buffers[i].lock().unwrap().get_index_from_tag("E");
            connect_gates(
                enable_input_gate.clone(),
                i,
                controlled_buffers[i].clone(),
                enable_index,
            );

            connect_gates(
                controlled_buffers[i].clone(),
                0,
                output_gate.clone(),
                0,
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
            &mut |_clock_tick_inputs, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
                collect_output_for_run_circuit(&mut collected_output, &output_gates);
            },
        );

        assert_eq!(collected_output, output_signal);
    }

    #[test]
    fn test_controlled_buffer_nested_in_complex_gate() {
        struct ControlledBufferWrapper {
            complex_gate: ComplexGateMembers,
            controlled_buffer: SharedMutex<ControlledBuffer>,
        }

        impl ControlledBufferWrapper {
            pub fn new() -> SharedMutex<Self> {
                let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
                let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
                let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

                new_shared_mutex(gate.get_unique_id().id(), gate)
            }

            fn build_and_prime_circuit(
                &mut self,
                output_gates: Vec<SharedMutex<dyn LogicGate>>,
            ) {
                connect_gates(
                    self.complex_gate.input_gates[self.get_index_from_tag("Input")].clone(),
                    0,
                    self.controlled_buffer.clone(),
                    0,
                );

                connect_gates(
                    self.controlled_buffer.clone(),
                    0,
                    output_gates[0].clone(),
                    0,
                );

                //Force the enable to low so that NONE is always returned.
                let enable_index = self.controlled_buffer.lock().unwrap().get_index_from_tag("E");
                self.controlled_buffer.lock().unwrap().update_input_signal(
                    GateInput::new(
                        enable_index,
                        LOW_,
                        UniqueID::zero_id(),
                    )
                );

                //Prime gates
                self.complex_gate.calculate_output_from_inputs_and_set_child_count(
                    true,
                );
            }
        }

        impl LogicGate for ControlledBufferWrapper {
            fn internal_connect_output(
                &mut self,
                current_gate_output_key: usize,
                next_gate_input_key: usize,
                next_gate: SharedMutex<dyn LogicGate>,
            ) -> Signal {
                self.complex_gate.connect_output(
                    self.get_unique_id(),
                    current_gate_output_key,
                    next_gate_input_key,
                    next_gate,
                )
            }

            fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
                self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
            }

            fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
                self.complex_gate.update_input_signal(input)
            }

            fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
                self.complex_gate.fetch_output_signals_calculate(
                    &self.get_tag(),
                )
            }

            fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
                self.complex_gate.fetch_output_signals_no_calculate(
                    &self.get_tag(),
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
                self.complex_gate.simple_gate.tag = tag.to_string()
            }

            fn get_index_from_tag(&self, tag: &str) -> usize {
                self.complex_gate.get_index_from_tag(tag)
            }

            fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
                self.complex_gate.remove_connected_input(input_index, connected_id);
            }

            fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
                self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
            }

            fn num_children_gates(&self) -> usize {
                self.complex_gate.simple_gate.number_child_gates
            }

            fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
                self.complex_gate.input_gates.clone()
            }
        }

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        let input_gate = AutomaticInput::new(vec![HIGH], 2, "Inputs");
        let output_gate = SimpleOutput::new("OUT");

        let wrapper = ControlledBufferWrapper::new();

        let output_signal = vec![[NONE]];

        connect_gates(
            input_gate.clone(),
            0,
            wrapper.clone(),
            0,
        );

        connect_gates(
            wrapper.clone(),
            0,
            output_gate.clone(),
            0,
        );

        input_gates.push(input_gate);
        output_gates.push(output_gate);

        let mut collected_output: Vec<Vec<Signal>> = Vec::new();

        run_circuit(
            &input_gates,
            &output_gates,
            false,
            &mut |_clock_tick_inputs, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
                collect_output_for_run_circuit(&mut collected_output, &output_gates);
            },
        );

        assert_eq!(collected_output, output_signal);
    }

    #[test]
    fn splitter_properly_splits() {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        let input_num = rand::thread_rng().gen_range(1..=16);
        let outputs_per_input = rand::thread_rng().gen_range(2..=16);

        let splitter = Splitter::new(input_num, outputs_per_input);

        let mut output_signal = Vec::new();
        let mut single_turn_output = Vec::new();
        for i in 0..input_num {
            let signal_num = rand::thread_rng().gen_range(0..=2);
            let signal = match signal_num {
                0 => LOW_,
                1 => HIGH,
                _ => NONE,
            };
            let input_tag = format!("IN_{}", i);
            let input_gate = AutomaticInput::new(
                vec![signal.clone()],
                1,
                input_tag.as_str(),
            );

            println!("signal: {:?}", signal.clone());

            for _ in 0..outputs_per_input {
                single_turn_output.push(signal.clone());
            }

            connect_gates(
                input_gate.clone(),
                0,
                splitter.clone(),
                i,
            );

            input_gates.push(input_gate);
        }

        output_signal.push(single_turn_output);

        for i in 0..input_num {
            for j in 0..outputs_per_input {
                let output_tag = format!("OUT_{}", i);
                let output_gate = SimpleOutput::new(output_tag.as_str());
                let splitter_output = splitter.lock().unwrap().get_index_for_output(i, j);
                println!("i {i} j {j} splitter_output {splitter_output}");
                connect_gates(
                    splitter.clone(),
                    splitter_output,
                    output_gate.clone(),
                    0,
                );

                output_gates.push(output_gate);
            }
        }

        let mut collected_output: Vec<Vec<Signal>> = Vec::new();

        run_circuit(
            &input_gates,
            &output_gates,
            false,
            &mut |_clock_tick_inputs, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
                assert_eq!(output_gates.len(), input_num * outputs_per_input);

                let mut single_collected_output = Vec::new();
                collect_outputs_from_output_gates(&output_gates, &mut single_collected_output);

                collected_output.push(single_collected_output);
            },
        );

        println!("{:#?}", collected_output);
        assert_eq!(collected_output, output_signal);
    }
}

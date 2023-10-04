use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, Signal, OscillationDetection, InputSignalReturn, calculate_input_signal_from_single_inputs};

pub trait OutputGate {
    fn get_output_tag(&self) -> String;
}

pub trait LogicGateAndOutputGate: LogicGate + OutputGate {}

impl<T: LogicGate + OutputGate> LogicGateAndOutputGate for T {}

pub struct SimpleOutput {
    output_state: HashMap<UniqueID, Signal>,
    unique_id: UniqueID,
    oscillation_detection: OscillationDetection,
    should_print_output: bool,
    print_each_input_output_gate: bool,
    gate_type: GateType,
    tag: String,
}

#[allow(dead_code)]
impl SimpleOutput {
    pub fn new(tag: &str) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                SimpleOutput {
                    output_state: HashMap::from([(UniqueID::zero_id(), Signal::LOW_)]),
                    unique_id: UniqueID::generate(),
                    oscillation_detection: OscillationDetection::new(),
                    should_print_output: false,
                    print_each_input_output_gate: true,
                    gate_type: GateType::SimpleOutputType,
                    tag: String::from(tag),
                }
            )
        )
    }
}

impl OutputGate for SimpleOutput {
    fn get_output_tag(&self) -> String {
        self.tag.clone()
    }
}

impl LogicGate for SimpleOutput {
    fn connect_output_to_next_gate(
        &mut self,
        _current_gate_output_key: usize,
        _next_gate_input_key: usize,
        _next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        panic!("An output gate should be the end of the circuit, it should never connect to another input.");
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        let changed_count_this_tick = self.oscillation_detection.detect_oscillation(
            &self.gate_type,
            &self.unique_id,
            &self.tag,
            &input.sending_id,
        );

        let input_signal_updated = if self.output_state[&input.sending_id] == input.signal {
            false
        } else {
            self.output_state.insert(input.sending_id, input.signal.clone());
            true
        };

        InputSignalReturn {
            changed_count_this_tick,
            input_signal_updated,
        }
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        let output_clone = calculate_input_signal_from_single_inputs(&self.output_state)?;
        // println!("SimpleOutput id {} output_clone: {:#?}", self.unique_id.id() ,output_clone);

        if self.should_print_output && self.print_each_input_output_gate {
            GateLogic::print_gate_output(
                &self.gate_type,
                &self.unique_id,
                &self.get_tag(),
                &None::<Signal>,
                &output_clone,
            );
        }

        Ok(vec![GateOutputState::NotConnected(output_clone)])
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
        self.tag.to_string()
    }

    fn set_tag(&mut self, tag: &str) {
        self.tag = tag.to_string()
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, _gate_input_index: usize, signal: Signal) {
        //Whenever an input is updated, remove the zero index. Even adding the zero index it will
        // simply be inserted immediately afterwards.
        self.output_state.remove(&UniqueID::zero_id());

        //This is a temporary signal. When the input is updated afterwards, it will add it.
        self.output_state.insert(sending_id, signal);
    }

    fn remove_connected_input(&mut self, _input_index: usize, connected_id: UniqueID) {
        self.output_state
            .remove(&connected_id)
            .expect(
                format!(
                    "When attempting to disconnect a gate, the gate with type {} id {} tag {} was not connected.",
                    self.gate_type,
                    self.unique_id.id(),
                    self.tag
                ).as_str()
            );
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.print_each_input_output_gate = print_each_input_output_gate;
    }
}

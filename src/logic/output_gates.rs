use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, Signal, OscillationDetection, InputSignalReturn};

pub struct SimpleOutput {
    output_state: Signal,
    unique_id: UniqueID,
    oscillation_detection: OscillationDetection,
    should_print_output: bool,
    gate_type: GateType,
}

#[allow(dead_code)]
impl SimpleOutput {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                SimpleOutput {
                    output_state: Signal::LOW,
                    unique_id: UniqueID::generate(),
                    oscillation_detection: OscillationDetection::new(),
                    should_print_output: false,
                    gate_type: GateType::SimpleOutput,
                }
            )
        )
    }
}

impl LogicGate for SimpleOutput {
    fn connect_output_to_next_gate(
        &mut self,
        _current_gate_output_index: usize,
        _next_gate_input_index: usize,
        _next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {}

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        let changed_count_this_tick = self.oscillation_detection.detect_oscillation(&self.gate_type);

        let input_signal_updated = if self.output_state == input.signal {
            false
        } else {
            self.output_state = input.signal.clone();
            true
        };

        InputSignalReturn {
            changed_count_this_tick,
            input_signal_updated
        }
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        let output_clone = self.output_state.clone();

        if self.should_print_output {
            GateLogic::print_gate_output(
                &self.gate_type,
                &self.unique_id,
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
}

use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic};
use crate::logic::foundations::{Signal::HIGH};

pub struct Clock {
    output_states: Vec<GateOutputState>,
    unique_id: UniqueID,
    should_print_output: bool,
    gate_type: GateType,
}

impl Clock {
    pub fn new(output_num: usize) -> Rc<RefCell<Self>> {
        let mut clock = Clock {
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            should_print_output: false,
            gate_type: GateType::CLOCK,
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

    fn update_input_signal(&mut self, _input: GateInput) -> bool {
        false
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
}

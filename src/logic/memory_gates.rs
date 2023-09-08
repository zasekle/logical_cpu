use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::foundations::{BasicGateMembers, GateInput, GateLogic, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, UniqueID};

pub struct SRLatch {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl SRLatch {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                SRLatch {
                    members: BasicGateMembers::new(
                        2,
                        2,
                        GateType::SRLatch,
                    )
                }
            )
        )
    }
}

impl LogicGate for SRLatch {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        //TODO: relies on 'calculate_output_from_inputs'
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::check_for_single_element_signal;
    use super::*;

    #[test]
    fn sr_gate() {}
}
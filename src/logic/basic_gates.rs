use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, GateLogic, BasicGateMembers};

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
                        GateType::Or,
                    )
                }
            )
        )
    }
}

impl LogicGate for Or {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> bool {
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
                        GateType::Not,
                    )
                }
            )
        )
    }
}

impl LogicGate for Not {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> bool {
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

//TODO:
// And
// Not (must test for cycles here)
// Nand
// Nor
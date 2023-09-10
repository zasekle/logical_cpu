use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::logic::foundations::{LogicGate, UniqueID};
use crate::logic::foundations::Signal::{HIGH, LOW};

#[allow(unused_imports)]
use crate::logic::basic_gates::{
    Or,
    And,
    Not,
    Nor,
    Nand,
};
#[allow(unused_imports)]
use crate::logic::input_gates::{
    AutomaticInput,
    Clock
};
use crate::logic::memory_gates::SRLatch;
use crate::logic::output_gates::LogicGateAndOutputGate;
#[allow(unused_imports)]
use crate::logic::output_gates::SimpleOutput;

pub struct InputAndOutputGates {
    pub input_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>>,
    pub output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGateAndOutputGate>>>,
}

pub fn build_simple_circuit() -> InputAndOutputGates {

    // let clock = Clock::new(1);
    // let first_input = AutomaticInput::new(vec![LOW, LOW, HIGH, HIGH], 1, "Reset");
    // let second_input = AutomaticInput::new(vec![LOW, HIGH, LOW, HIGH], 1, "Set");
    let first_input = AutomaticInput::new(vec![LOW], 1, "OUT_Reset");
    let second_input = AutomaticInput::new(vec![LOW], 1, "OUT_Set");
    let first_output_gate = SimpleOutput::new("OUT_Q");
    let second_output_gate = SimpleOutput::new("OUT_~Q");
    let mut input_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
    let mut output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGateAndOutputGate>>> = HashMap::new();

    input_gates.insert(first_input.borrow_mut().get_unique_id(), first_input.clone());
    input_gates.insert(second_input.borrow_mut().get_unique_id(), second_input.clone());
    output_gates.insert(first_output_gate.borrow_mut().get_unique_id(), first_output_gate.clone());
    output_gates.insert(second_output_gate.borrow_mut().get_unique_id(), second_output_gate.clone());

    let sr_latch = SRLatch::new();
    sr_latch.borrow_mut().toggle_output_printing(true);

    first_input.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        sr_latch.clone()
    );

    second_input.borrow_mut().connect_output_to_next_gate(
        0,
        1,
        sr_latch.clone()
    );

    sr_latch.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        first_output_gate.clone()
    );

    sr_latch.borrow_mut().connect_output_to_next_gate(
        1,
        0,
        second_output_gate.clone()
    );

    InputAndOutputGates {
        input_gates,
        output_gates,
    }
}
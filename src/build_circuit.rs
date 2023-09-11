use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::foundations::LogicGate;

#[allow(unused_imports)]
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
    pub input_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    pub output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
}

pub fn build_simple_circuit() -> InputAndOutputGates {

    // let clock = Clock::new(1);
    let first_input = AutomaticInput::new(vec![LOW, HIGH, LOW, LOW, LOW, HIGH, LOW], 1, "R");
    let second_input = AutomaticInput::new(vec![LOW, LOW, LOW, HIGH, LOW, HIGH, LOW], 1, "S");
    let first_output_gate = SimpleOutput::new("Q");
    let second_output_gate = SimpleOutput::new("~Q");
    let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

    input_gates.push(first_input.clone());
    input_gates.push(second_input.clone());
    output_gates.push(first_output_gate.clone());
    output_gates.push(second_output_gate.clone());

    let sr_latch = SRLatch::new();
    // sr_latch.borrow_mut().toggle_output_printing(true);

    first_input.borrow_mut().connect_output_to_next_gate(
        0,
        sr_latch.borrow_mut().get_index_from_tag("R"),
        sr_latch.clone()
    );

    second_input.borrow_mut().connect_output_to_next_gate(
        0,
        sr_latch.borrow_mut().get_index_from_tag("S"),
        sr_latch.clone()
    );

    let mut mut_sr_latch = sr_latch.borrow_mut();
    let q_output_idx = mut_sr_latch.get_index_from_tag("Q");
    mut_sr_latch.connect_output_to_next_gate(
        q_output_idx,
        0,
        first_output_gate.clone()
    );

    let not_q_output_idx = mut_sr_latch.get_index_from_tag("~Q");
    mut_sr_latch.connect_output_to_next_gate(
        not_q_output_idx,
        0,
        second_output_gate.clone()
    );

    InputAndOutputGates {
        input_gates,
        output_gates,
    }
}
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
#[allow(unused_imports)]
use crate::logic::memory_gates::{ActiveLowSRLatch, SRLatch, OneBitMemoryCell};
#[allow(unused_imports)]
use crate::logic::output_gates::LogicGateAndOutputGate;
#[allow(unused_imports)]
use crate::logic::output_gates::SimpleOutput;

pub struct InputAndOutputGates {
    pub input_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    pub output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
}

pub fn build_simple_circuit() -> InputAndOutputGates {
    let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
    let r_input_gate = AutomaticInput::new(vec![LOW], 1, "START_R");
    let s_input_gate = AutomaticInput::new(vec![LOW], 1, "START_S");
    let q_output_gate = SimpleOutput::new("END_Q");
    let not_q_output_gate = SimpleOutput::new("END_~Q");

    let sr_latch = SRLatch::new();

    sr_latch.borrow_mut().toggle_output_printing(true);

    r_input_gate.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        sr_latch.clone(),
    );

    s_input_gate.borrow_mut().connect_output_to_next_gate(
        0,
        1,
        sr_latch.clone(),
    );

    sr_latch.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        q_output_gate.clone(),
    );

    sr_latch.borrow_mut().connect_output_to_next_gate(
        1,
        0,
        not_q_output_gate.clone(),
    );

    input_gates.push(r_input_gate);
    input_gates.push(s_input_gate);
    output_gates.push(q_output_gate);
    output_gates.push(not_q_output_gate);

    // let clock = Clock::new(1);
    // let first_input = AutomaticInput::new(vec![LOW, HIGH, LOW, HIGH, LOW], 1, "START_E");
    // let second_input = AutomaticInput::new(vec![LOW, HIGH, LOW, LOW, HIGH], 1, "START_S");
    // let first_output_gate = SimpleOutput::new("END_1");
    // let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    // let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
    //
    // input_gates.push(first_input.clone());
    // input_gates.push(second_input.clone());
    // output_gates.push(first_output_gate.clone());
    //
    // let sr_latch = OneBitMemoryCell::new();
    // // let mut mut_sr_latch = sr_latch.borrow_mut();
    // // mut_sr_latch.toggle_output_printing(true);
    //
    // let e_index = sr_latch.borrow_mut().get_index_from_tag("E");
    // first_input.borrow_mut().connect_output_to_next_gate(
    //     0,
    //     e_index,
    //     sr_latch.clone()
    // );
    //
    // let s_index = sr_latch.borrow_mut().get_index_from_tag("S");
    // second_input.borrow_mut().connect_output_to_next_gate(
    //     0,
    //     s_index,
    //     sr_latch.clone()
    // );
    //
    // let q_output_idx = sr_latch.borrow_mut().get_index_from_tag("Q");
    // sr_latch.borrow_mut().connect_output_to_next_gate(
    //     q_output_idx,
    //     0,
    //     first_output_gate.clone()
    // );

    InputAndOutputGates {
        input_gates,
        output_gates,
    }
}
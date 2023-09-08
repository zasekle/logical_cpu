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
use crate::logic::output_gates::LogicGateAndOutputGate;
#[allow(unused_imports)]
use crate::logic::output_gates::SimpleOutput;

pub struct InputAndOutputGates {
    pub input_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>>,
    pub output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGateAndOutputGate>>>,
}

pub fn build_simple_circuit() -> InputAndOutputGates {

    // let clock = Clock::new(1);
    let first_input = AutomaticInput::new(vec![LOW, LOW, HIGH, HIGH], 1, "Reset");
    let second_input = AutomaticInput::new(vec![LOW, HIGH, LOW, HIGH], 1, "Set");
    let first_output_gate = SimpleOutput::new("Q");
    let second_output_gate = SimpleOutput::new("~Q");
    let mut input_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
    let mut output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGateAndOutputGate>>> = HashMap::new();

    input_gates.insert(first_input.borrow_mut().get_unique_id(), first_input.clone());
    input_gates.insert(second_input.borrow_mut().get_unique_id(), second_input.clone());
    output_gates.insert(first_output_gate.borrow_mut().get_unique_id(), first_output_gate.clone());
    output_gates.insert(second_output_gate.borrow_mut().get_unique_id(), second_output_gate.clone());

    let first_nor_gate = Nor::new(2, 2);
    let second_nor_gate = Nor::new(2, 2);

    first_input.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        first_nor_gate.clone(),
    );

    second_input.borrow_mut().connect_output_to_next_gate(
        0,
        1,
        second_nor_gate.clone(),
    );

    first_nor_gate.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        first_output_gate.clone(),
    );

    first_nor_gate.borrow_mut().connect_output_to_next_gate(
        1,
        0,
        second_nor_gate.clone(),
    );

    second_nor_gate.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        second_output_gate.clone(),
    );

    second_nor_gate.borrow_mut().connect_output_to_next_gate(
        1,
        1,
        first_nor_gate.clone(),
    );

    InputAndOutputGates {
        input_gates,
        output_gates,
    }
}
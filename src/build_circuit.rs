use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::logic::foundations::{LogicGate, UniqueID};
use crate::logic::foundations::Signal::{HIGH, LOW};

#[allow(unused_imports)]
use crate::logic::basic_gates::{
    Or,
    Not,
};
#[allow(unused_imports)]
use crate::logic::input_gates::{
    AutomaticInput,
    Clock
};
#[allow(unused_imports)]
use crate::logic::output_gates::SimpleOutput;

pub struct InputAndOutputGates {
    pub input_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>>,
    pub output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>>,
}

pub fn build_simple_circuit() -> InputAndOutputGates {

    // let clock = Clock::new(1);
    let automatic_inputs = AutomaticInput::new(vec![HIGH, HIGH, LOW], 1);
    let first_or_gate = Or::new(2, 1);
    let not_gate = Not::new(1);
    let output_gate = SimpleOutput::new();

    automatic_inputs.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        not_gate.clone(),
    );

    not_gate.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        first_or_gate.clone(),
    );

    first_or_gate.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        output_gate.clone(),
    );

    let mut input_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
    input_gates.insert(automatic_inputs.borrow_mut().get_unique_id(), automatic_inputs.clone());

    let mut output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
    output_gates.insert(output_gate.borrow_mut().get_unique_id(), output_gate.clone());

    InputAndOutputGates {
        input_gates,
        output_gates,
    }
}
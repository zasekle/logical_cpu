use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::foundations::LogicGate;

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{HIGH, LOW_};

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

#[allow(dead_code)]
pub struct InputAndOutputGates {
    pub input_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    pub output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
}

#[allow(dead_code)]
pub fn build_simple_circuit() -> InputAndOutputGates {
    let  input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let  output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

    InputAndOutputGates {
        input_gates,
        output_gates,
    }
}
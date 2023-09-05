extern crate core;

mod logic;

use std::cell::RefCell;
use std::collections::{HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize};
use crate::logic::basic_gates::{
    Or,
    Not,
};
use crate::logic::input_gates::{
    Clock,
};
use crate::logic::foundations::{
    GateOutputState,
    LogicGate,
    UniqueID
};


/// This will represent the current clock tick number. It should only ever by incremented directly
/// after a clock tick occurs with only a single thread running.
static mut CLOCK_TICK_NUMBER: usize = 0;

pub fn get_clock_tick_number() -> usize {
    let clock_tick_number;
    unsafe {
        clock_tick_number = CLOCK_TICK_NUMBER;
    }
    clock_tick_number
}

/// This is the maximum number of times an input can change in a single clock tick. After this,
/// oscillation will be assumed and the program will panic.
static MAX_INPUT_CHANGES: usize = 5000;

/// This will allow each gate to have a unique indexing number.
static NEXT_UNIQUE_ID: AtomicUsize = AtomicUsize::new(0);

fn main() {

    //TODO: Maybe a little more cleanup on the code, it is much better than it was.

    //TODO: How do I give it manual inputs? Maybe the clock works on a separate thread to the input
    // and I just feed it commands from the GUI? Maybe have the clock always running and it checks
    // a vector for possible commands, then on the main thread here I input commands.
    // Maybe I can just have an object that is `manual inputs`.
    //TODO: So maybe I want to actually make something I can control the input in. Maybe call it
    // `input` type logic gate or something?

    //TODO: Do some light documentation.
    //TODO: Write some tests.
    //  NOT gate feeding back into itself (state will oscillate).
    //  OR gate feeding back into itself (on forever).

    let clock = Clock::new(1);
    let first_or_gate = Or::new(2, 2);
    let not_gate = Not::new(1);

    clock.borrow_mut().connect_output_to_next_gate(
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
        1,
        first_or_gate.clone(),
    );

    for _ in 0..2 {
        //This should be the ONLY place this is ever updated.
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        let mut next_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
        let mut final_output = Vec::new();

        next_gates.insert(clock.borrow_mut().get_unique_id(), clock.clone());

        while !next_gates.is_empty() {
            let gates = next_gates;
            next_gates = HashMap::new();

            for gate in gates.values() {
                let mut gate = gate.borrow_mut();
                let gate_output = gate.fetch_output_signals().unwrap();

                drop(gate);
                for output in gate_output {
                    match output {
                        GateOutputState::NotConnected(signal) => {
                            final_output.push(signal);
                        }
                        GateOutputState::Connected(next_gate_info) => {
                            let next_gate = Rc::clone(&next_gate_info.gate);
                            let mut mutable_next_gate = next_gate.borrow_mut();

                            let input_changed = mutable_next_gate.update_input_signal(next_gate_info.throughput);
                            let gate_id = mutable_next_gate.get_unique_id();

                            //It is important to remember that a situation such as an OR gate feeding
                            // back into itself is perfectly valid. This can be interpreted that if the
                            // input was not changed, the output was not changed either and so nothing
                            // needs to be done with this gate.
                            //Also each gate only needs to be stored inside the map once. All changed
                            // inputs are saved as part of the state, so collect_output() only needs
                            // run once.
                            if input_changed && !next_gates.contains_key(&gate_id) {
                                drop(mutable_next_gate);
                                next_gates.insert(gate_id, next_gate);
                            }
                        }
                    }
                }
            }
        }

        for o in final_output.iter() {
            println!("o: {:#?}", o);
        }
    }

    println!("Program Completed!");
}

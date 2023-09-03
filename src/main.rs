use std::cell::RefCell;
use std::collections::{HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize};
use crate::basic_logic::{Clock, GateOutput, LogicGate, UniqueID};

mod basic_logic;

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
static MAX_NUMBER_TIMES_INPUTS_CHANGE: usize = 5000;

/// This will allow each gate to have a unique indexing number.
static UNIQUE_INDEXING_NUMBER: AtomicUsize = AtomicUsize::new(0);

fn main() {

    //TODO: How do I give it manual inputs? Maybe the clock works on a separate thread to the input
    // and I just feed it commands from the GUI? Maybe have the clock always running and it checks
    // a vector for possible commands, then on the main thread here I input commands.
    //TODO: Think about future debugging, what will be the best way to implement it. I would like
    // something to print the circuit in a human readable way. Maybe make it print the schematic or
    // something.
    //TODO: Do some light documentation.
    //TODO: Write some tests.
    //  NOT gate feeding back into itself (state will oscillate).
    //  OR gate feeding back into itself (on forever).
    //TODO: Can I set this up to somehow allow multithreading in the future? I may be able to
    // roughly separate out chunks of the map and divide them up into different maps, then combine
    // them at the end somehow (probably store all output in a personal map, then return it somehow
    // and combine them at the end).

    //TODO: Probably want to separate out LogicGate and OutputNode into a different file so they
    // can be used in other gates. (All of the basic stuff, UniqueId should go with it).

    //TODO: Redo the names of the major components so they make sense.
    //TODO: Need to do some other stuff with the code to make it nicer and more extendable.

    let clock = Clock::new(1);
    let first_or_gate = basic_logic::Or::new(2, 2);
    let not_gate = basic_logic::Not::new(1);

    clock.borrow_mut().connect_output(
        0,
        0,
        not_gate.clone(),
    );

    not_gate.borrow_mut().connect_output(
        0,
        0,
        first_or_gate.clone(),
    );

    first_or_gate.borrow_mut().connect_output(
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

        next_gates.insert(clock.borrow_mut().get_id(), clock.clone());

        while !next_gates.is_empty() {
            let gates = next_gates;
            next_gates = HashMap::new();

            for gate in gates.values() {
                let mut gate = gate.borrow_mut();
                let gate_output = gate.collect_output().unwrap();

                drop(gate);
                for output in gate_output {
                    match output {
                        GateOutput::NotConnected(signal) => {
                            final_output.push(signal);
                        }
                        GateOutput::Connected(next_gate_info) => {
                            let next_gate = Rc::clone(&next_gate_info.gate);
                            let mut mutable_next_gate = next_gate.borrow_mut();

                            let input_changed = mutable_next_gate.change_input(&next_gate_info.input);
                            let gate_id = mutable_next_gate.get_id();

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

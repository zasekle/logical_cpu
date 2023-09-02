use std::cell::RefCell;
use std::rc::Rc;
use crate::basic_logic::{Clock, GateOutput, LogicGate};

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

fn main() {

    //TODO: How do I give it manual inputs? Maybe the clock works on a separate thread to the input
    // and I just feed it commands from the GUI? Maybe have the clock always running and it checks
    // a vector for possible commands, then on the main thread here I input commands.
    //TODO: Think about future debugging, what will be the best way to implement it. I would like
    // something to print the circuit in a human readable way. Maybe make it print the schematic or
    // something.
    //TODO: Write some tests.
    //TODO: Can I set this up to somehow allow multithreading in the future?

    //TODO: Probably want to separate out LogicGate and OutputNode into a different file so they can be used in other gates.

    // TODO Make sure these 2 Situations are covered
    //  NOT gate feeding back into itself (state will oscillate).
    //  OR gate feeding back into itself (on forever).

    //TODO: I need an ID for each of them, so maybe a better way to do this is to store all gates inside an array
    // (can drop the RefCell) then each of them can have their unique ID set up by their index
    // inside the array. This would let me change the return value to an index instead of using an Rc as well.

    //TODO: Redo the names of the major components so they make sense.

    let clock = Clock::new(1);
    // let first_or_gate = basic_logic::Or::new(2, 1);
    // let second_or_gate = basic_logic::Or::new(2, 1);
    let not_gate = basic_logic::Not::new(1);

    clock.borrow_mut().connect_output(
        0,
        not_gate.clone(),
        0,
    );

    not_gate.borrow_mut().connect_output(
        0,
        not_gate.clone(),
        0,
    );

    // clock.borrow_mut().connect_output(
    //     0,
    //     first_or_gate.clone(),
    //     0,
    // );

    // first_or_gate.borrow_mut().connect_output(
    //     0,
    //     second_or_gate.clone(),
    //     1,
    // );

    for _ in 0..2 {
        //This should be the ONLY place this is ever updated.
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        let mut next_gates: Vec<Rc<RefCell<dyn LogicGate>>> = vec![];
        let mut final_output = Vec::new();

        next_gates.push(clock.clone());

        while !next_gates.is_empty() {
            let gates = next_gates;
            next_gates = Vec::new();

            for gate in gates.iter() {
                let mut gate = gate.borrow_mut();
                let gate_output = gate.collect_output().unwrap();

                for output in gate_output {
                    match output {
                        GateOutput::NotConnected(signal) => {
                            final_output.push(signal);
                        }
                        GateOutput::Connected(next_gate_info) => {
                            let next_gate = Rc::clone(&next_gate_info.gate);

                            next_gate.borrow_mut().change_input(&next_gate_info.input);

                            //TODO: I need to make sure to only add the gate if the ID is unique (follow the comment below), I think
                            // maybe it truly is better to store them all in a big vector and just
                            // access them. This would also give an 'index' value to each one and allow
                            // for ease of storing in say a set (or an ordered vector).

                            //It is important to remember that a situation such as an OR gate feeding
                            // back into itself is perfectly valid. This can be interpreted that if the
                            // input was not changed, the output was not changed either and so nothing
                            // needs to be done with this gate.
                            next_gates.push(next_gate);
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

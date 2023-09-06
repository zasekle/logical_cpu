extern crate core;

mod logic;

use std::cell::RefCell;
use std::collections::{HashMap};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize};

use crate::logic::foundations::{
    GateOutputState,
    LogicGate,
    UniqueID,
    Signal::{
        HIGH,
        LOW,
    },
    GateLogicError
};

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

    //TODO: Probably want to extract the building of the circuit and the running of the circuit into
    // different places. This will also be necessary when running tests so that the circuits can
    // be simulated by running them.

    //TODO: Write some tests.
    //  NOT gate feeding back into itself (state will oscillate).
    //  OR gate feeding back into itself (on forever).

    //TODO: Do some light documentation.

    // let clock = Clock::new(1);
    let inputs = AutomaticInput::new(vec![HIGH, HIGH, LOW], 1);
    let first_or_gate = Or::new(2, 1);
    let not_gate = Not::new(1);
    let output_gate = SimpleOutput::new();

    inputs.borrow_mut().connect_output_to_next_gate(
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

    //TODO: Maybe make this a function that is just `add_output` or something and it returns the
    // output gate that it created to be connected to other gates. This would also allow me to send
    // in a tag or something to identify the specific gate (much more useful than the id).
    let mut output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
    output_gates.insert(output_gate.borrow_mut().get_unique_id(), output_gate.clone());

    'clock_cycle: loop {
        //This should be the ONLY place this is ever updated.
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        let mut next_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
        let mut final_output = Vec::new();

        // next_gates.insert(clock.borrow_mut().get_unique_id(), clock.clone());
        next_gates.insert(inputs.borrow_mut().get_unique_id(), inputs.clone());

        while !next_gates.is_empty() {
            let gates = next_gates;
            next_gates = HashMap::new();

            for gate in gates.values() {
                let mut gate = gate.borrow_mut();
                let gate_output = gate.fetch_output_signals();

                let gate_output = if let Err(err) = gate_output {
                    match err {
                        GateLogicError::NoMoreAutomaticInputsRemaining => {
                            break 'clock_cycle;
                        }
                    }
                } else {
                    gate_output.unwrap()
                };

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

        println!("Output for clock-tick #{}", get_clock_tick_number());
        for (id, output_gate) in output_gates.iter() {
            let mut output_gate = output_gate.borrow_mut();
            let fetched_signal = output_gate.fetch_output_signals().unwrap();
            let output = fetched_signal.first().unwrap();

            if let GateOutputState::NotConnected(signal) = output {
                println!("   type: {:?} id: {:?} signal: {:?}", output_gate.get_gate_type(), id.id(), signal);
            } else {
                panic!("An output gate did not have any output");
            }
        }
    }

    println!("Program Completed!");
}

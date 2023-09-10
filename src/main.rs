extern crate core;

mod logic;
mod run_circuit;
mod globals;
mod build_circuit;
mod test_stuff;

use globals::get_clock_tick_number;

use crate::logic::foundations::{GateOutputState, Signal};

use build_circuit::build_simple_circuit;
use crate::build_circuit::InputAndOutputGates;
use crate::run_circuit::start_clock;

fn main() {

    //TODO: Working on SR Latch
    // The output are 'interesting' because of how this latch works and how I simulate the propagating
    //  of electricity, the latch can start in either the state 0 1 or 1 0 if the input are both low
    //  will need to research how this is actually handled. Apparently this mimics real life to some
    //  extent because depending on which gate receives power first, the output changes. So this
    //  initial state will need to be handled when building the circuit. Not 100% sure I understand,
    //  on real circuits, there may be a defined state on something called 'power up'.
    // BUT shouldn't my switch be deterministic because when a new SRLatch is created it primes it
    //  right? Why is it changing with different inputs?
    // Make some tests for it
    // Allow me to connect by tag as well

    println!("Building circuit!");

    let InputAndOutputGates{input_gates, output_gates} =
        build_simple_circuit();

    println!("Running circuit!");

    start_clock(
        &input_gates,
        &output_gates,
        |clock_tick_inputs, output_gates| {
            let clock_tick_number = get_clock_tick_number();
            println!("Inputs for clock-tick #{}", clock_tick_number);
            for (tag, gate_output_state) in clock_tick_inputs.iter() {
                let output_states: Vec<Signal> = gate_output_state
                    .iter()
                    .map(|g| {
                        match g {
                            GateOutputState::NotConnected(signal) => {
                                signal.clone()
                            }
                            GateOutputState::Connected(connected_output) => {
                                connected_output.throughput.signal.clone()
                            }
                        }
                    })
                    .collect();
                println!("   tag: {:?} signals: {:?}", tag, output_states);
            }

            println!("Output for clock-tick #{}", clock_tick_number);
            for (_id, output_gate) in output_gates.iter() {
                let mut output_gate = output_gate.borrow_mut();
                let fetched_signal = output_gate.fetch_output_signals().unwrap();
                let output = fetched_signal.first().unwrap();

                if let GateOutputState::NotConnected(signal) = output {
                    println!("   tag: {:?} signal: {:?}", output_gate.get_output_tag(), signal);
                } else {
                    panic!("An output gate did not have any output.");
                }
            }
        }
    );

    println!("Program Completed!");
}

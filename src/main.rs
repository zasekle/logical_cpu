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
use crate::run_circuit::run_circuit;

fn main() {

    println!("Building circuit!");

    let InputAndOutputGates{input_gates, output_gates} =
        build_simple_circuit();

    println!("Running circuit!");

    run_circuit(
        input_gates,
        output_gates,
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

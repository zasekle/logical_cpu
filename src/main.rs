extern crate core;

mod logic;
mod run_circuit;
mod globals;
mod build_circuit;
mod test_stuff;

use globals::get_clock_tick_number;

use crate::logic::foundations::{
    GateOutputState,
};

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
        |output_gates| {
            println!("Output for clock-tick #{}", get_clock_tick_number());
            for (id, output_gate) in output_gates.iter() {
                let mut output_gate = output_gate.borrow_mut();
                let fetched_signal = output_gate.fetch_output_signals().unwrap();
                let output = fetched_signal.first().unwrap();

                if let GateOutputState::NotConnected(signal) = output {
                    println!("   type: {:?} id: {:?} signal: {:?}", output_gate.get_gate_type(), id.id(), signal);
                } else {
                    panic!("An output gate did not have any output.");
                }
            }
        }
    );

    println!("Program Completed!");
}

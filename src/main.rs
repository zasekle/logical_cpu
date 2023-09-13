extern crate core;
extern crate rand;

mod logic;
mod run_circuit;
mod globals;
mod build_circuit;
mod test_stuff;

use globals::get_clock_tick_number;

use crate::logic::foundations::pretty_print_output;

use build_circuit::build_simple_circuit;
use crate::build_circuit::InputAndOutputGates;
use crate::run_circuit::start_clock;

fn main() {
    //TODO: Should I make a way to pass a 'bus' in to things? I think I should build the register
    // first, then I should see what would be a convenient way to pass information. Maybe I can just
    // make a global function or something that takes different types and this will solve my
    // problem.
    //TODO: For some reason the CPU_ENABLE is running twice inside the register. Figure out why.
    // cargo test -- logic::processor_components::tests::processor_register_simple_test --nocapture
    //TODO: Do more tests for the register.

    println!("Building circuit!");

    let InputAndOutputGates{input_gates, output_gates} =
        build_simple_circuit();

    println!("Running circuit!");

    start_clock(
        &input_gates,
        &output_gates,
        |clock_tick_inputs, output_gates| {
            let clock_tick_number = get_clock_tick_number();
            let input_string = format!("Global inputs for clock-tick #{}", clock_tick_number);
            let output_string = format!("Global outputs for clock-tick #{}", clock_tick_number);

            pretty_print_output(
                true,
                clock_tick_inputs,
                output_gates,
                input_string.as_str(),
                output_string.as_str(),
            );
        }
    );

    println!("Program Completed!");
}

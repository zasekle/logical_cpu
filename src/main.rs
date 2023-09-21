extern crate core;
extern crate rand;
extern crate backtrace;

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

    //TODO: Rule for the ALU
    // idx 0 is the least significant bit
    // idx 7 is the most significant bit

    //TODO: Make a note inside somewhere for possible optimizations for the ALU

    //TODO: need
    // ALU & don't forget the SignalGatekeeper
    //  SignalGatekeeper can be put before every gate except the XOR I think, it will need to be
    //  OFF until the decoder switches it on

    //TODO: There are other gates I can probably tie together using run_circuit, although I don't
    // know if it will matter. Any of the ones that have a Vec of gates inside them.

    //TODO: With the way that I did run_circuit and grouping the gates before running them, it might
    // be possible to run them in a multithreaded way. Or maybe every time a signal splits I can
    // make a new thread (or a new task to pass into a thread pool at least).

    //Remember that when running stuff in the registers, there is always the possibility that
    // multiple clock ticks are needed. The first will do something like enable the `Set` bit. The
    // second will keep the `Set` bit high and change the input values. The third will bring the
    // `Set` bit low without changing the inputs.

    println!("Building circuit!");

    let InputAndOutputGates{input_gates, output_gates} =
        build_simple_circuit();

    println!("Running circuit!");

    start_clock(
        &input_gates,
        &output_gates,
        |clock_tick_inputs, output_gates| {

            //NOTE FOR LATER: Make sure to make a match statement for the final output because there
            // is more than LOW and HIGH that `Signal` can return.

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

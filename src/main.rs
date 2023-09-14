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

    //TODO: There is a problem right now, the controlled buffer will essentially 'break' the
    // connection by returning 'not connected'. However, when 'fetch_output' is called, it will
    // still collect the most recent output that it can. Maybe I need to return or store an error
    // in some way saying 'no signal' inside the output. How would this propagate though? I could
    // feed in a few different signals, then when comparing the signals in output or something
    // check them? But realistically, having two wires going into the same element (well for output
    // this can't happen) and one saying no signal? Errors, Signal Enum is there more?

    //TODO: Might want to add some output to the memory inside the register to debug? How would I
    // display this?
    //TODO: Need to add a controlled buffer to the register.
    //TODO: Build a little RAM cell.
    //TODO: Build a large RAM cell.

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

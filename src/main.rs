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

    //TODO: Make toggle_output_printing turn off (or able to be turned off) for the really big gates
    // in fetch. ComplexGateMembers.fetch_output_signals should have it. Can probably individually
    // build it into the large gates like RAMUnit and ControlUnit. Would be nice to have a prettier
    // way to print the bus too, but it is a Splitter, a simple gate.

    //TODO
    // Right now the way that it gets the memory is actually part of the circuit, might want to change
    // that so it is programmatically done instead. There isn't actually any need to do it the way
    // I currently am.

    //TODO: I can probably tie my SignalGatekeeper to the set values of a lot of other places. For
    // example the RAM, the registers, the memory etc... Otherwise, it will need to propagate through
    // the entire thing every time.

    //TODO: may be worthwhile to change the way RAMUnit and VariableBitCPU work to take more than
    // just the absolute size of a decoder and instead take more fine grained numbers.

    //TODO: There are other gates I can probably tie together using run_circuit, although I don't
    // know if it will matter. Any of the ones that have a Vec of gates inside them. However, I think
    // it only matters if they have a variable number of inputs to the specific blocks inside of them
    // to avoid the same block from being called twice. Might be worthwhile in the adder (or other ALU
    // gates?)

    //TODO: With the way that I did run_circuit and grouping the gates before running them, it might
    // be possible to run them in a multithreaded way. Or maybe every time a signal splits I can
    // make a new thread (or a new task to pass into a thread pool at least). Or I could just make
    // multiple processors connected to the same RAM and executing different instructions lol.
    // Probably not the last one, while it would give me some insight into how to handle some of the
    // multi-threaded problems, it would also make the simulation run the propagation of electricity
    // on multiple CPUs.

    //Remember that when running stuff in the registers, there is always the possibility that
    // multiple clock ticks are needed. The first will do something like enable the `Set` bit. The
    // second will keep the `Set` bit high and change the input values. The third will bring the
    // `Set` bit low without changing the inputs.

    println!("Building circuit!");

    let InputAndOutputGates{input_gates, output_gates} =
        build_simple_circuit();

    println!("Running circuit!");

    //TODO: I don't think the propagation needs to be done because it is done inside VariableBitCPU
    // priming (it is done inside start_clock).
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

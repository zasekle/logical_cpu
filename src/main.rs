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

    //TODO: There are technically a few things to be built still.
    // Accumulator (This is identical to a VariableBitRegister)
    // TMP (This is identical to a VariableBitMemoryCell)
    // Bus (Can probably just make a VariableBitBus or something and connect all the inputs & outputs to it)
    // Clock (This will need logic on top of what I already have for it).

    //TODO: I can do this
    // 1) I can make the Control Section
    //  As I build things inside the Control section I can test the output bits and were they are
    //  supposed to be at different clock ticks.

    //TODO: I assume that I will want to accumulate everything in the bus, then propagate it out.

    //TODO: I can probably tie my SignalGatekeeper to the set values of a lot of other places. For
    // example the RAM, the registers, the memory etc... Otherwise, it will need to propagate through
    // the entire thing every time.

    //TODO: I wonder if I can tie things together like this to force them to happen only AFTER the
    // set or enable bit have been changed. Does it matter?
    //TODO: There are other gates I can probably tie together using run_circuit, although I don't
    // know if it will matter. Any of the ones that have a Vec of gates inside them. However, I think
    // it only matters if they have a variable number of inputs to the specific blocks inside of them
    // to avoid the same block from being called twice.

    //TODO: With the way that I did run_circuit and grouping the gates before running them, it might
    // be possible to run them in a multithreaded way. Or maybe every time a signal splits I can
    // make a new thread (or a new task to pass into a thread pool at least). Or I could just make
    // multiple processors connected to the same RAM and executing different instructions lol.

    //TODO: The reset pin on the ControlUnit relies on all registers being set to pull down, so if
    // they get NONE as the input, they need the bits to be set low.

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

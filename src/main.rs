extern crate core;
extern crate rand;
extern crate backtrace;

mod logic;
mod run_circuit;
mod globals;
mod build_circuit;
mod test_stuff;

use std::fs::File;
use std::io::Read;
use std::time::Duration;
use crate::globals::get_clock_tick_number;

use crate::run_circuit::{collect_signals_from_logic_gate, run_instructions};
use crate::test_stuff::extract_output_tags_sorted_by_index;

pub(crate) static mut RAM_TIME: Duration = Duration::new(0, 0);
pub(crate) static mut CONTROL_SECTION_TIME: Duration = Duration::new(0, 0);
pub(crate) static mut ALU_TIME: Duration = Duration::new(0, 0);

fn main() {

    //NOTE: This processor runs at ~54Hz. I was originally planning to attempt to simulate something
    // more complex with it. But even a processor from the early 90s will run at ~20MHz which is
    // something like 500,000x faster. So in order to make this processor able to do anything in
    // a reasonable amount of time, I would probably need to re-write the entire simulation with
    // dropping most of the logic gates themselves and this would defeat the purpose.

    //TODO
    // Right now the way that it loads the memory is actually part of the circuit, might want to change
    // that so it is programmatically done instead. There isn't actually any need to do it the way
    // I currently am.

    //TODO: I can probably tie my SignalGatekeeper to the set values of a lot of other places. For
    // example the RAM, the registers, the memory etc... Otherwise, it will need to propagate through
    // the entire thing every time.

    //TODO: With the way that I did run_circuit and grouping the gates before running them, it might
    // be possible to run them in a multithreaded way. Or maybe every time a signal splits I can
    // make a new thread (or a new task to pass into a thread pool at least).

    //TODO: How to make this multithreaded.
    // Lets say that I make it so that run_circuit has access to a global thread pool, each time an
    // object is added the the queue inside run_circuit,

    let number_bits = 8;
    let num_decoder_input = 4;

    let mut file = File::open("programs/multiplication.ms").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let mut machine_code = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if line.bytes().len() != number_bits {
            panic!("Failed to parse machine code. Line number {} is an invalid length.", i);
        }

        for c in line.bytes() {
            if c != b'0' && c != b'1' {
                panic!("Invalid char of {} found on line number {} of machine code.", c as char, i)
            }
        }

        machine_code.push(line);
    }

    let cpu = run_instructions(
        number_bits,
        num_decoder_input,
        &machine_code,
    );

    let tags_sorted_by_index = extract_output_tags_sorted_by_index(&cpu.borrow_mut().get_complex_gate());
    let collected_signals = collect_signals_from_logic_gate(cpu.clone());

    assert_eq!(collected_signals.len(), tags_sorted_by_index.len());

    for i in 0..tags_sorted_by_index.len() {
        println!("{} {:?}", tags_sorted_by_index[i], collected_signals[i]);
    }

    unsafe {
        println!("RAM_TIME: {:?}", RAM_TIME);
        println!("CONTROL_SECTION_TIME: {:?}", CONTROL_SECTION_TIME);
        println!("ALU_TIME: {:?}", ALU_TIME);
    }
}

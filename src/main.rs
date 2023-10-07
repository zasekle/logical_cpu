extern crate core;
extern crate rand;
extern crate backtrace;

mod logic;
mod run_circuit;
mod globals;
mod build_circuit;
mod test_stuff;
mod shared_mutex;

use std::fs::File;
use std::io::Read;
use std::time::Duration;

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
    // object is added the the queue inside run_circuit, it will send a notification or something to
    // the thread pool, a thread will wake up and process it.
    //  It will need some kind of shared data structure in order to properly process the items. This
    //   will also avoid errors.
    //  How big of items should it process? Maybe I can add a feature to count the number of unique
    //   gates in the object during priming. Then I can make a minimum number of gates that is actually
    //   run in a multi-threaded way.
    //  Do my items each need to have a Mutex? So Arc<Mutex?
    //  I have some unsafe code in foundations.rs, how will this work?
    //  The above two points work together. At a basic level, my system works just fine, there
    //   should be no deadlock potential with simply transitioning everything to Arc<Mutex<>> types
    //   This is because gates are called from top to bottom, never sideways. However, there is a
    //   problem with the unsafe code in foundations.rs because it calls backwards.
    //  Change 'connect_output_to_next_gate' to some kind of a global function. This will allow the
    //   functions, `internal_update_index_to_id` and the current `connect_output_to_next_gate` to
    //   be called at the same time yet not inside the same function.
    //  The hierarchy disappearing is still a problem. Right now it essentially works in a standard
    //   recursive way. It will grab the next gate and then wait for the output to calculate, then
    //   grab the gate after that. But lets say that I call a complex gate, then I call the gates
    //   inside of it. Lets take RAMUnit for example, If I send in each RAM Cell individually to my
    //   thread pool, how exactly will the RAMUnit 'know' that it has completed and to do the
    //   outputs of it. I need a different state I guess? Maybe instead of doing it the way I am, I
    //   can link an output gate to the gate itself? But if I do this, there is no real way to tell
    //   when it is 'completed'. Maybe I give each gate an atomic counter that tells how many gates
    //   are currently in queue and when it hits 0, I add the gate back into the
    //   waiting_to_be_processed queue?

    //TODO: Shared data structure.
    // What happens if an item is pushed while another thread is currently processing it? Before
    //  that thread finishes processing that item, it should check and make sure the item was not
    //  added to the queue again somehow. Maybe I have a 'currently_processing` queue and a
    //  'waiting_to_be_processed' queue. Then if it is currently_processing, it can put the object
    //  in a special place for that queue (or better yet interrupt the thread that is processing
    //  it). If not, add it to waiting_to_be_processed assuming it is not already in the queue.

    //TODO: Stuff to do.
    // Create a shared data structure.
    // Will need a separate run_circuit function for multi-threaded situations.
    // Make something that calculates
    // Change 'connect_output_to_next_gate' to some kind of a global function. Maybe instead, I can
    //  just make the current one private inside foundation, then make a global wrapper to call it
    //  or something. Whatever works the best is what I want to do as I do an overhaul to the
    //  program.
    // Change all the Rc<RefCell<>> to Arc<Mutex<>>


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

    let tags_sorted_by_index = extract_output_tags_sorted_by_index(&cpu.lock().unwrap().get_complex_gate());
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

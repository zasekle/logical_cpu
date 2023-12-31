use std::sync::atomic::{AtomicBool, AtomicUsize};

/// This will represent the current clock tick number. It should only ever by incremented directly
/// after a clock tick occurs with only a single thread running.
pub(crate) static mut CLOCK_TICK_NUMBER: usize = 0;

pub fn get_clock_tick_number() -> usize {
    unsafe {
        CLOCK_TICK_NUMBER.clone()
    }
}

/// This is the maximum number of times an input can change in a single clock tick. After this,
/// oscillation will be assumed and the program will panic.
pub(crate) static MAX_INPUT_CHANGES: usize = 25000;

/// This will allow each gate to have a unique indexing number.
/// ID 0 is reserved and used with the input for gates representing no input.
pub(crate) static NEXT_UNIQUE_ID: AtomicUsize = AtomicUsize::new(1);

/// This is just used for testing purposes. Can enable it so a single run_circuit function prints
/// output.
pub(crate) static RUN_CIRCUIT_IS_HIGH_LEVEL: AtomicBool = AtomicBool::new(false);

/// This is set to the output gate that ends the program.
pub static END_OUTPUT_GATE_TAG: &'static str = "_END_OUTPUT_";
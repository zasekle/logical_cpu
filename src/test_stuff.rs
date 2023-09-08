use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use crate::logic::foundations::{GateOutputState, Signal, UniqueID};
use crate::logic::output_gates::LogicGateAndOutputGate;

#[allow(dead_code)]
pub fn check_for_single_element_signal(
    output_gates: &HashMap<UniqueID, Rc<RefCell<dyn LogicGateAndOutputGate>>>,
    output: Signal,
) {
    assert_eq!(output_gates.len(), 1);
    let (_key, value) = output_gates.into_iter().next().unwrap();
    let mut value = value.borrow_mut();
    let output_signals = value.fetch_output_signals().unwrap();

    assert_eq!(output_signals.len(), 1);

    let gate_output_state = output_signals.first().unwrap();

    match gate_output_state {
        GateOutputState::NotConnected(signal) => {
            assert_eq!(*signal, output)
        }
        GateOutputState::Connected(_) => {
            panic!("The output gate should never be connected.");
        }
    }
}

#[allow(dead_code)]
pub fn run_test_with_timeout<F: Send + 'static>(
    timeout_duration: Duration,
    test: F
) where F: FnOnce()
{
    let (tx, rx) = channel();

    thread::spawn(move || {
        test();

        // Notify that the test is complete
        tx.send("done").expect("send should succeed");
    });

    match rx.recv_timeout(timeout_duration) {
        Ok(_) => {
            //Test completed within 500ms
        }
        Err(_) => {
            //Test timed out, because some tests rely on panic! being triggered, assert! is
            // used instead.
            assert!(false);
        }
    }
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use crate::globals::CLOCK_TICK_NUMBER;
use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, Signal};
use crate::logic::input_gates::AutomaticInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
use crate::run_circuit::run_circuit;

#[allow(dead_code)]
pub fn check_for_single_element_signal(
    output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
    output: Signal,
) {
    assert_eq!(output_gates.len(), 1);
    let mut output_gate = output_gates.first().unwrap().borrow_mut();
    let output_signals = output_gate.fetch_output_signals().unwrap();

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

#[allow(dead_code)]
pub fn run_multi_input_output_logic_gate(
    input_signals: Vec<Vec<Signal>>,
    output_signal: Vec<Vec<Signal>>,
    tagged_input_signal: HashMap<&str, Vec<Signal>>,
    gate: Rc<RefCell<dyn LogicGate>>,
) {
    assert!(!input_signals.is_empty());
    assert_eq!(input_signals.len(), output_signal.len());

    let num_bits = input_signals[0].len();
    for i in 0..input_signals.len() {
        assert_eq!(input_signals[i].len(), num_bits);
        assert_eq!(output_signal[i].len(), num_bits);
    }

    for (_tag, signals) in tagged_input_signal.iter() {
        assert_eq!(signals.len(), input_signals.len());
    }

    let mut mut_gate = gate.borrow_mut();

    {
        let output_size = mut_gate.fetch_output_signals().unwrap();

        assert_eq!(output_size.len(), input_signals[0].len());
    }

    let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

    for (i, signals) in input_signals.into_iter().enumerate() {
        for (j, signal) in signals.into_iter().enumerate() {
            let cell_index = mut_gate.get_index_from_tag(format!("i_{}", j).as_str());
            if i == 0 {
                let input_gate = AutomaticInput::new(vec![signal], 1, "Start");

                input_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    cell_index,
                    Rc::clone(&gate),
                );

                input_gates.push(input_gate);
            } else {
                input_gates[j].borrow_mut().update_input_signal(
                    GateInput {
                        input_index: cell_index,
                        signal: signal.clone(),
                    }
                );
            }
        }
    }

    for (tag, signals) in tagged_input_signal {
        let tagged_input_gate = AutomaticInput::new(signals, 1, format!("Start_{}", tag).as_str());
        input_gates.push(tagged_input_gate.clone());

        tagged_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            mut_gate.get_index_from_tag(tag),
            Rc::clone(&gate),
        );
    }

    for i in 0..num_bits {
        let output_gate = SimpleOutput::new("End");

        mut_gate.connect_output_to_next_gate(
            i,
            0,
            output_gate.clone(),
        );

        output_gates.push(output_gate);
    }

    drop(mut_gate);

    let mut collected_output: Vec<Vec<Signal>> = Vec::new();
    let mut propagate_signal_through_circuit = true;
    let mut continue_clock = true;

    while continue_clock {
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        continue_clock = run_circuit(
            &input_gates,
            &output_gates,
            propagate_signal_through_circuit,
            &mut |_clock_tick_inputs, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                assert_eq!(output_gates.len(), num_bits);

                let mut single_collected_output = Vec::new();
                for output in output_gates.iter() {
                    let mut output = output.borrow_mut();

                    let output = output.fetch_output_signals().unwrap();

                    assert_eq!(output.len(), 1);

                    let output = output.first().unwrap();

                    match output {
                        GateOutputState::NotConnected(signal) => {
                            single_collected_output.push(signal.clone());
                        }
                        GateOutputState::Connected(_) => panic!("Final output gate should not be connected"),
                    }
                }

                collected_output.push(single_collected_output);
            },
        );

        propagate_signal_through_circuit = false;
    }

    assert_eq!(collected_output, output_signal);
}

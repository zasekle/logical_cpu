use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use crate::globals::CLOCK_TICK_NUMBER;
use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, Signal, UniqueID};
use crate::logic::input_gates::AutomaticInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
use crate::run_circuit::{run_circuit, start_clock};

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
    test: F,
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
    tagged_input_signal: HashMap<&str, Vec<Vec<Signal>>>,
    gate: Rc<RefCell<dyn LogicGate>>,
) {
    let num_outputs = output_signal[0].len();

    let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

    for (i, signals) in input_signals.into_iter().enumerate() {
        for (j, signal) in signals.into_iter().enumerate() {
            let cell_index = gate.borrow_mut().get_index_from_tag(format!("i_{}", j).as_str());
            if i == 0 {
                let input_gate = AutomaticInput::new(vec![signal], 1, "Start");

                input_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    cell_index,
                    Rc::clone(&gate),
                );

                input_gates.push(input_gate);
            } else {
                let mut input_gate = input_gates[j].borrow_mut();
                input_gate.update_input_signal(
                    GateInput::new(
                        cell_index,
                        signal.clone(),
                        UniqueID::zero_id(),
                    )
                );
            }
        }
    }

    for (tag, signals) in tagged_input_signal.into_iter() {
        let starting_index = input_gates.len();
        for (i, signals) in signals.into_iter().enumerate() {
            let size = signals.len();
            for (j, signal) in signals.into_iter().enumerate() {
                let tag =
                    if size == 1 {
                        tag.to_string()
                    } else {
                        format!("{}_{}", tag, j)
                    };

                let tag_index = gate.borrow_mut().get_index_from_tag(tag.as_str());
                if i == 0 {
                    let input_gate = AutomaticInput::new(vec![signal], 1, "Start");

                    input_gate.borrow_mut().connect_output_to_next_gate(
                        0,
                        tag_index,
                        Rc::clone(&gate),
                    );

                    input_gates.push(input_gate);
                } else {
                    let mut input_gate = input_gates[starting_index + j].borrow_mut();
                    input_gate.update_input_signal(
                        GateInput::new(
                            tag_index,
                            signal.clone(),
                            UniqueID::zero_id(),
                        )
                    );
                }
            }
        }
    }

    // let mut errors = vec![false; input_gates.len()];
    // while errors.contains(&false) {
    //     let mut gates = Vec::new();
    //     for (i, inp) in input_gates.iter().enumerate() {
    //         let out = inp.borrow_mut().fetch_output_signals();
    //
    //         if let Err(_) = out {
    //             errors[i] = true;
    //             gates.push(NONE);
    //             continue;
    //         }
    //
    //         match out.unwrap().first().unwrap() {
    //             GateOutputState::NotConnected(signal) => {
    //                 gates.push(signal.clone());
    //             }
    //             GateOutputState::Connected(out) => {
    //                 gates.push(out.throughput.signal.clone());
    //             }
    //         }
    //     }
    //     println!("input_line: {:?}", gates);
    // }

    for i in 0..num_outputs {
        let output_gate = SimpleOutput::new("End");

        gate.borrow_mut().connect_output_to_next_gate(
            i,
            0,
            output_gate.clone(),
        );

        output_gates.push(output_gate);
    }

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
                assert_eq!(output_gates.len(), num_outputs);

                let mut single_collected_output = Vec::new();
                collect_outputs_from_output_gates(&output_gates, &mut single_collected_output);

                collected_output.push(single_collected_output);
            },
            None,
        );

        propagate_signal_through_circuit = false;
    }

    assert_eq!(collected_output, output_signal);
}

pub fn collect_outputs_from_output_gates(output_gates: &&Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>, single_collected_output: &mut Vec<Signal>) {
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
}

#[allow(dead_code)]
pub fn test_simple_gate(
    gate: Rc<RefCell<dyn LogicGate>>,
    first_input: Signal,
    second_input: Option<Signal>,
    output: Signal,
) {
    let first_pin_input = AutomaticInput::new(vec![first_input], 1, "");
    let output_gate = SimpleOutput::new("");

    let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

    input_gates.push(first_pin_input.clone());
    output_gates.push(output_gate.clone());

    first_pin_input.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        gate.clone(),
    );

    if let Some(second_input) = second_input {
        let second_pin_input = AutomaticInput::new(vec![second_input], 1, "");

        second_pin_input.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            gate.clone(),
        );

        input_gates.push(second_pin_input.clone());
    }

    gate.borrow_mut().connect_output_to_next_gate(
        0,
        0,
        output_gate.clone(),
    );

    start_clock(
        &input_gates,
        &output_gates,
        &mut |_: &Vec<(String, Vec<GateOutputState>)>, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
            check_for_single_element_signal(&output_gates, output.clone());
        },
    );
}

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use crate::globals::CLOCK_TICK_NUMBER;
use crate::logic::foundations::{GateLogicError, GateOutputState, InputSignalReturn, LogicGate};
use crate::logic::output_gates::LogicGateAndOutputGate;

pub fn start_clock<F>(
    input_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
    mut handle_output: F,
) where
    F: FnMut(&Vec<(String, Vec<GateOutputState>)>, &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>)
{
    assert!(!input_gates.is_empty());
    assert!(!output_gates.is_empty());

    let mut propagate_signal_through_circuit = true;
    let mut continue_clock = true;

    while continue_clock {
        //This should be the ONLY place this is ever updated.
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        continue_clock = run_circuit(
            input_gates,
            output_gates,
            propagate_signal_through_circuit,
            &mut handle_output,
        );

        propagate_signal_through_circuit = false;
    }
}

//Returns true if the circuit has input remaining, false if it does not.
//Note that elements must be ordered so that some of the undetermined gates such as SR latches can
// have a defined starting state. Therefore, vectors are used even though they must be iterated
// through to guarantee uniqueness.
pub fn run_circuit<F>(
    input_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
    propagate_signal_through_circuit: bool,
    handle_output: &mut F,
) -> bool where
    F: FnMut(&Vec<(String, Vec<GateOutputState>)>, &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>)
{
    let mut clock_tick_inputs = Vec::new();
    let mut next_gates: Vec<Rc<RefCell<dyn LogicGate>>> = input_gates.clone();
    let mut final_output = Vec::new();

    while !next_gates.is_empty() {
        // println!("next_gates.len() = {}", next_gates.len());
        let gates = next_gates;
        next_gates = Vec::new();
        let mut next_gates_set = HashSet::new();

        for gate in gates.iter() {
            let mut gate = gate.borrow_mut();
            let gate_output = gate.fetch_output_signals();

            let gate_output = if let Err(err) = gate_output {
                return match err {
                    GateLogicError::NoMoreAutomaticInputsRemaining => {
                        false
                    }
                };
            } else {
                gate_output.unwrap()
            };

            if gate.is_input_gate() {
                clock_tick_inputs.push(
                    (gate.get_tag(), gate_output.clone())
                );
            }

            drop(gate);
            for output in gate_output {
                match output {
                    GateOutputState::NotConnected(signal) => {
                        final_output.push(signal);
                    }
                    GateOutputState::Connected(next_gate_info) => {
                        let next_gate = Rc::clone(&next_gate_info.gate);
                        let mut mutable_next_gate = next_gate.borrow_mut();

                        let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                            mutable_next_gate.update_input_signal(next_gate_info.throughput);
                        let gate_id = mutable_next_gate.get_unique_id();

                        let contains_id = next_gates_set.contains(&gate_id);

                        // println!("input_signal_updated: {} contains_key(): {:#?} changed_count_this_tick: {:?}", input_signal_updated, next_gates.contains_key(&gate_id), changed_count_this_tick);
                        //It is important to remember that a situation such as an OR gate feeding
                        // back into itself is perfectly valid. This can be interpreted that if the
                        // input was not changed, the output was not changed either and so nothing
                        // needs to be done with this gate.
                        //The first tick is a bit special, because the circuit needs to propagate
                        // the signal regardless of if the gates change or not. This leads to
                        // checking if it is the first time the gate is updated on the first
                        // clock tick.
                        //Also each gate only needs to be stored inside the map once. All changed
                        // inputs are saved as part of the state, so collect_output() only needs
                        // to run once.
                        if (input_signal_updated || (propagate_signal_through_circuit && changed_count_this_tick == 1)) && !contains_id {
                            drop(mutable_next_gate);
                            // println!("next_gates.insert()");
                            next_gates_set.insert(gate_id);
                            next_gates.push(next_gate);
                        }
                    }
                }
            }
        }
    }

    handle_output(
        &clock_tick_inputs,
        &output_gates,
    );

    true
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::logic::basic_gates::{Not, Or};
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::SimpleOutput;
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::{check_for_single_element_signal, run_test_with_timeout};
    use super::*;

    #[test]
    fn minimum_system() {
        let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
        let output_gate = SimpleOutput::new("");

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        run_circuit(
            &input_gates,
            &output_gates,
            false,
            &mut |_clock_tick_inputs, output_gates| {
                check_for_single_element_signal(output_gates, HIGH);
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_oscillation() {
        run_test_with_timeout(
            Duration::from_millis(500),
            || {
                let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
                let output_gate = SimpleOutput::new("");
                let not_gate = Not::new(2);

                let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
                let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

                input_gates.push(input_gate.clone());
                output_gates.push(output_gate.clone());

                input_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    0,
                    not_gate.clone(),
                );

                not_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    0,
                    output_gate.clone(),
                );

                //Create a loop.
                not_gate.borrow_mut().connect_output_to_next_gate(
                    1,
                    0,
                    not_gate.clone(),
                );

                run_circuit(
                    &input_gates,
                    &output_gates,
                    false,
                    &mut |_clock_tick_inputs, _output_gates| {
                        //An oscillation should panic! before it ever reaches this point. Cannot use the
                        // panic! macro because the test will not be able to check if it failed properly or
                        // not.
                        assert!(false);
                    },
                );
            },
        );
    }

    #[test]
    fn test_simple_loop() {
        run_test_with_timeout(
            Duration::from_millis(500),
            || {
                let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
                let output_gate = SimpleOutput::new("");
                let or_gate = Or::new(2, 2);

                let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
                let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

                input_gates.push(input_gate.clone());
                output_gates.push(output_gate.clone());

                input_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    0,
                    or_gate.clone(),
                );

                or_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    0,
                    output_gate.clone(),
                );

                //Create a loop.
                or_gate.borrow_mut().connect_output_to_next_gate(
                    1,
                    1,
                    or_gate.clone(),
                );

                run_circuit(
                    &input_gates,
                    &output_gates,
                    false,
                    &mut |_clock_tick_inputs, output_gates| {
                        check_for_single_element_signal(output_gates, HIGH);
                    },
                );
            },
        );
    }

    //Because this `not` gate has the default input value, its initial state will be set to LOW and
    // not be change under normal circumstances. However, the first clock tick everything must
    // propagate through the system to properly set the outputs. This means that the final output
    // should be changed to HIGH.
    #[test]
    fn first_tick_propagates() {
        let input_gate = AutomaticInput::new(vec![LOW], 1, "");
        let output_gate = SimpleOutput::new("");
        let not_gate = Not::new(1);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            not_gate.clone(),
        );

        not_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        start_clock(
            &input_gates,
            &output_gates,
            &mut |_: &Vec<(String, Vec<GateOutputState>)>, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                check_for_single_element_signal(output_gates, HIGH);
            },
        );
    }

    #[test]
    fn multiple_ticks() {
        let input_gate = AutomaticInput::new(vec![LOW, HIGH, HIGH], 1, "");
        let output_gate = SimpleOutput::new("");
        let not_gate = Not::new(1);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            not_gate.clone(),
        );

        not_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        let expected_outputs = vec![HIGH, LOW, LOW];
        let mut current_index = 0;

        start_clock(
            &input_gates,
            &output_gates,
            &mut |_: &Vec<(String, Vec<GateOutputState>)>, output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>| {
                assert!(current_index < expected_outputs.len());
                assert_eq!(output_gates.len(), 1);

                let value = output_gates.into_iter().next().unwrap();
                let mut value = value.borrow_mut();
                let output_signals = value.fetch_output_signals().unwrap();

                assert_eq!(output_signals.len(), 1);

                let gate_output_state = output_signals.first().unwrap();

                match gate_output_state {
                    GateOutputState::NotConnected(signal) => {
                        if let Some(output) = expected_outputs.get(current_index) {
                            assert_eq!(*signal, *output)
                        } else {
                            panic!("The number of outputs exceeded the maximum number.");
                        }
                    }
                    GateOutputState::Connected(_) => {
                        panic!("The output gate should never be connected.");
                    }
                }

                current_index += 1;
            },
        );

        assert_eq!(current_index, expected_outputs.len());
    }
}
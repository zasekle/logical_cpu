use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use crate::globals::{CLOCK_TICK_NUMBER, RUN_CIRCUIT_IS_HIGH_LEVEL};
use crate::logic::foundations::{GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate};
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
            None,
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
    gate_type_to_run_together: Option<GateType>,
) -> bool where
    F: FnMut(&Vec<(String, Vec<GateOutputState>)>, &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>)
{
    let print_output =
        if RUN_CIRCUIT_IS_HIGH_LEVEL.load(Ordering::SeqCst) {
            RUN_CIRCUIT_IS_HIGH_LEVEL.store(false, Ordering::SeqCst);
            true
        } else {
            false
        };

    let mut clock_tick_inputs = Vec::new();
    let mut next_gates: Vec<Rc<RefCell<dyn LogicGate>>> = input_gates.clone();

    let mut gathering_gates_to_run = true;
    let mut gathered_gates_set = HashSet::new();
    let mut gathered_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

    if print_output {
        println!("run_circuit");
    }
    while !next_gates.is_empty() {
        if print_output {
            println!("next_gates.len() = {}", next_gates.len());
        }
        let gates = next_gates;
        next_gates = Vec::new();
        let mut next_gates_set = HashSet::new();
        let mut num_invalid_gates: usize = 0;

        for gate_cell in gates.into_iter() {

            let mut gate = gate_cell.borrow_mut();
            let gate_output = gate.fetch_output_signals();

            let gate_output = if let Err(err) = gate_output {
                match err {
                    GateLogicError::NoMoreAutomaticInputsRemaining => {
                        return false
                    }
                    GateLogicError::MultipleValidSignalsWhenCalculating => {
                        num_invalid_gates += 1;
                        drop(gate);
                        next_gates.push(gate_cell);
                        continue
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
            if print_output {
                println!("gate_output.len(): {:?}", gate_output.len());
            }

            if let Some(gate_type) = gate_type_to_run_together {
                if gate_type == gate.get_gate_type() && gathering_gates_to_run
                {
                    if gathered_gates_set.insert(gate.get_unique_id()) {
                        drop(gate);
                        gathered_gates.push(gate_cell);
                    }
                    continue
                }
            }

            drop(gate);
            for output in gate_output.into_iter() {
                match output {
                    GateOutputState::NotConnected(signal) => {
                        if print_output {
                            println!("NotConnected(gate_output): {:?}", signal);
                        }
                    }
                    GateOutputState::Connected(next_gate_info) => {
                        if print_output {
                            println!("Connected(gate_output): {:?}", next_gate_info);
                        }
                        let next_gate = Rc::clone(&next_gate_info.gate);
                        let mut mutable_next_gate = next_gate.borrow_mut();

                        let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                            mutable_next_gate.update_input_signal(next_gate_info.throughput);
                        let gate_id = mutable_next_gate.get_unique_id();

                        let contains_id = next_gates_set.contains(&gate_id);

                        if print_output {
                            println!("checking gate {} tag {}", mutable_next_gate.get_gate_type(), mutable_next_gate.get_tag());
                            println!("input_signal_updated {input_signal_updated} propagate_signal_through_circuit {propagate_signal_through_circuit} changed_count_this_tick {changed_count_this_tick} contains_id {contains_id}");
                        }

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
                            if print_output {
                                println!("Pushing gate {} tag {}", mutable_next_gate.get_gate_type(), mutable_next_gate.get_tag());
                            }
                            drop(mutable_next_gate);
                            // println!("next_gates.insert()");
                            next_gates_set.insert(gate_id);
                            next_gates.push(next_gate);
                        }
                    }
                }
            }

        }


        //This is set up to handle invalid states. If all gates are in an invalid state the app will
        // panic. See calculate_input_signal_from_single_inputs() in foundations.rs for more
        // details.
        if num_invalid_gates > 0 && num_invalid_gates == next_gates.len() {
            let mut gates = Vec::new();
            for gate in next_gates {
                let mut_gate = gate.borrow_mut();
                gates.push(
                    format!("Gate {} id {} with tag {}.", mut_gate.get_gate_type(), mut_gate.get_unique_id().id(), mut_gate.get_tag())
                );

            }
            panic!("All gates inside the circuit have returned invalid input, aborting.\nInvalid Gate List\n{:#?}", gates);
        }

        //This will allow the function to group all the gates of the same type together and run
        // them at the same time.
        if !gathered_gates.is_empty() && next_gates.is_empty() {
            gathering_gates_to_run = false;
            next_gates = gathered_gates;
            gathered_gates = Vec::new();
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
    use crate::logic::foundations::Signal::{HIGH, LOW_};
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
            None,
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
                    None,
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
                    None,
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
        let input_gate = AutomaticInput::new(vec![LOW_], 1, "");
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
        let input_gate = AutomaticInput::new(vec![LOW_, HIGH, HIGH], 1, "");
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

        let expected_outputs = vec![HIGH, LOW_, LOW_];
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
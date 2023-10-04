use std::cell::{RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use crate::globals::{CLOCK_TICK_NUMBER, END_OUTPUT_GATE_TAG, get_clock_tick_number, RUN_CIRCUIT_IS_HIGH_LEVEL};
use crate::logic::foundations::{GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::foundations::Signal::{HIGH, LOW_};
use crate::logic::input_gates::{AutomaticInput, Clock};
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
use crate::logic::processor_components::RAMUnit;
use crate::logic::variable_bit_cpu::VariableBitCPU;
use crate::{ALU_TIME, CONTROL_SECTION_TIME, RAM_TIME};
use crate::test_stuff::extract_output_tags_sorted_by_index;

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
    let mut continue_clock = true;

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
                        return false;
                    }
                    GateLogicError::MultipleValidSignalsWhenCalculating => {
                        num_invalid_gates += 1;
                        drop(gate);
                        next_gates.push(gate_cell);
                        continue;
                    }
                };
            } else {
                gate_output.unwrap()
            };

            if gate.is_input_gate() {
                if print_output {
                    println!("input_gate_called: {}", gate.get_tag());
                }
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
                    continue;
                }
            }

            drop(gate);
            for output in gate_output.into_iter() {
                match output {
                    GateOutputState::NotConnected(signal) => {
                        if print_output {
                            println!("NOT_CONNECTED gate_tag {}", gate_cell.borrow_mut().get_tag());
                        }

                        if gate_cell.borrow_mut().get_tag() == END_OUTPUT_GATE_TAG
                            && signal == HIGH {
                            println!("End of program reached on clock-tick {}. Stopping execution.", get_clock_tick_number());
                            continue_clock = false;
                        }
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
                            mutable_next_gate.update_input_signal(next_gate_info.throughput.clone());
                        let gate_id = mutable_next_gate.get_unique_id();

                        let contains_id = next_gates_set.contains(&gate_id);

                        if print_output {
                            println!("checking gate {} tag {} signal {:?}", mutable_next_gate.get_gate_type(), mutable_next_gate.get_tag(), next_gate_info.throughput.signal.clone());
                            // println!("input_signal_updated: {} contains_key(): {:#?} changed_count_this_tick: {:?}", input_signal_updated, next_gates.contains_key(&gate_id), changed_count_this_tick);
                            println!("input_signal_updated: {input_signal_updated} propagate_signal_through_circuit: {propagate_signal_through_circuit} changed_count_this_tick {changed_count_this_tick} contains_id {contains_id}");
                        }

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

    continue_clock
}

pub fn generate_default_output(cpu: &Rc<RefCell<VariableBitCPU>>) -> Vec<Signal> {

    // Multi-bit outputs
    // VariableBitCPU::R0
    // VariableBitCPU::R1
    // VariableBitCPU::R2
    // VariableBitCPU::R3
    // VariableBitCPU::IR
    // VariableBitCPU::IAR
    // VariableBitCPU::ACC
    // VariableBitCPU::TMP
    // VariableBitCPU::BUS
    // RAM_registers (no constant)
    //
    // Single-bit outputs
    // VariableBitCPU::CLK
    // VariableBitCPU::CLKE
    // VariableBitCPU::CLKS
    // VariableBitCPU::IO
    // VariableBitCPU::DA
    // VariableBitCPU::END
    // VariableBitCPU::IO_CLK_E
    // VariableBitCPU::IO_CLK_S

    let mut generated_signals = vec![LOW_; cpu.borrow_mut().get_complex_gate().output_gates.len()];
    let clke_index = cpu.borrow_mut().get_complex_gate().gate_tags_to_index[VariableBitCPU::CLKE].index;
    generated_signals[clke_index] = HIGH;
    generated_signals
}

pub fn convert_binary_to_inputs_for_load(binary_strings: Vec<&str>, num_ram_cells: usize) -> Vec<Rc<RefCell<AutomaticInput>>> {
    assert_ne!(binary_strings.len(), 0);
    assert!(binary_strings.len() <= num_ram_cells);

    let mut ram_inputs = vec![vec![]; binary_strings.first().unwrap().len()];
    for (i, string) in binary_strings.iter().enumerate() {
        for (j, c) in string.chars().rev().enumerate() {
            let signal =
                if c == '0' {
                    LOW_
                } else {
                    HIGH
                };

            let num_pushes =
                if i != 0 {
                    4
                } else {
                    2
                };

            for _ in 0..num_pushes {
                ram_inputs[j].push(signal.clone());
            }
        }
    }

    //The vector is filled up so that it runs for each ram cell. Then there are two extra inputs
    // needed to put the clock from the end of LOAD to the starting clock state.
    let num_extra_inputs = (num_ram_cells - binary_strings.len()) * 4 + 2;
    for i in 0..ram_inputs.len() {
        for _ in 0..num_extra_inputs {
            ram_inputs[i].push(LOW_);
        }
    }

    let mut automatic_inputs = Vec::new();
    for (i, inp) in ram_inputs.iter().enumerate() {
        let input_tag = format!("Input_bit_{}", i);
        automatic_inputs.push(
            AutomaticInput::new(inp.clone(), 1, input_tag.as_str())
        );
    }

    automatic_inputs
}

pub fn collect_signals_from_logic_gate(gate: Rc<RefCell<dyn LogicGate>>) -> Vec<Signal> {
    let cpu_output = gate.borrow_mut().fetch_output_signals().unwrap();
    let mut collected_signals = Vec::new();
    for out in cpu_output.into_iter() {
        match out {
            GateOutputState::NotConnected(signal) => {
                collected_signals.push(signal);
            }
            GateOutputState::Connected(connected_output) => {
                collected_signals.push(connected_output.throughput.signal);
            }
        }
    }
    collected_signals
}

pub fn run_instructions(
    number_bits: usize,
    decoder_input_size: usize,
    binary_strings: &Vec<&str>,
) -> Rc<RefCell<VariableBitCPU>> {
    let cpu = VariableBitCPU::new(number_bits, decoder_input_size);

    let num_ram_cells = usize::pow(2, (decoder_input_size * 2) as u32);
    assert!(binary_strings.len() <= num_ram_cells);
    if !binary_strings.is_empty() {
        assert_eq!(binary_strings[0].len(), number_bits);
    }

    println!("Beginning to load values into RAM");

    let start_load = Instant::now();

    load_values_into_ram(
        &cpu,
        binary_strings,
        num_ram_cells,
    );

    let complete_load = Instant::now();

    let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let clock = Clock::new(1, "PRIMARY_CLOCK");
    let clk_in_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::CLK_IN);
    cpu.borrow_mut().get_clock_synced_with_cpu(&clock);

    clock.borrow_mut().connect_output_to_next_gate(
        0,
        clk_in_index,
        cpu.clone(),
    );

    input_gates.push(clock.clone());

    let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
    let end_output_gate = SimpleOutput::new(END_OUTPUT_GATE_TAG);

    let cpu_end_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::END);
    cpu.borrow_mut().connect_output_to_next_gate(
        cpu_end_index,
        0,
        end_output_gate.clone(),
    );

    output_gates.push(end_output_gate.clone());

    println!("\nCompleted load in {} clock-ticks. Beginning program.\n", get_clock_tick_number());
    unsafe {
        CLOCK_TICK_NUMBER = 0;
        RAM_TIME = Duration::new(0, 0);
        CONTROL_SECTION_TIME = Duration::new(0, 0);
        ALU_TIME = Duration::new(0, 0);
    }
    let mut continue_load_operation = true;
    let mut propagate_signal = true;
    while continue_load_operation {
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        continue_load_operation = run_circuit(
            &input_gates,
            &output_gates,
            propagate_signal,
            &mut |_clock_tick_inputs, _output_gates| {},
            None,
        );

        propagate_signal = false;
    }

    let complete_run = Instant::now();

    let run_time = complete_run.duration_since(complete_load);
    println!("Loading took {:?}", complete_load.duration_since(start_load));
    println!("Run took {:?}", run_time);
    println!("Total took {:?}", complete_run.duration_since(start_load));
    println!(
        "CPU ran at {}Hz",
        if run_time.as_secs() == 0 {
            0
        } else {
            get_clock_tick_number() as u64 / complete_run.duration_since(complete_load).as_secs()
        }
    );

    cpu
}

//This should leave the cpu in the same state as it started in. The only difference is that
// there will now be values loaded into RAM. It should be run without any inputs connected to
// the cpu itself.
pub fn load_values_into_ram(
    cpu: &Rc<RefCell<VariableBitCPU>>,
    binary_strings: &Vec<&str>,
    num_ram_cells: usize,
) {
    let automatic_inputs = convert_binary_to_inputs_for_load(
        binary_strings.clone(),
        num_ram_cells,
    );

    let num_cycles = num_ram_cells * 4 - 2;

    //The last cycle is to advance the clock to the starting position. AND to get the splitter
    // to the correct position.
    let output_values = vec![HIGH; num_cycles + 1];

    let load_automatic_input = AutomaticInput::new(
        output_values.clone(),
        1,
        "LOAD",
    );

    let memory_address_register_automatic_input = AutomaticInput::new(
        output_values,
        1,
        "MEMORY_ADDRESS_REGISTER",
    );

    let load_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::LOAD);
    load_automatic_input.borrow_mut().connect_output_to_next_gate(
        0,
        load_index,
        cpu.clone(),
    );

    let memory_address_register_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::MARS);
    memory_address_register_automatic_input.borrow_mut().connect_output_to_next_gate(
        0,
        memory_address_register_index,
        cpu.clone(),
    );

    let mut automatic_input_gates: Vec<Rc<RefCell<AutomaticInput>>> = Vec::new();
    let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
    let clock = Clock::new(1, "PRIMARY_CLOCK");
    cpu.borrow_mut().get_clock_synced_with_cpu(&clock);

    let clk_in_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::CLK_IN);
    clock.borrow_mut().connect_output_to_next_gate(
        0,
        clk_in_index,
        cpu.clone(),
    );

    input_gates.push(clock.clone());
    input_gates.push(load_automatic_input.clone());
    input_gates.push(memory_address_register_automatic_input.clone());
    automatic_input_gates.push(load_automatic_input);
    automatic_input_gates.push(memory_address_register_automatic_input);

    for (i, input) in automatic_inputs.iter().enumerate() {
        let ram_input_tag = format!("{}_{}", VariableBitCPU::RAM, i);
        let ram_input_index = cpu.borrow_mut().get_index_from_tag(ram_input_tag.as_str());
        input.borrow_mut().connect_output_to_next_gate(
            0,
            ram_input_index,
            cpu.clone(),
        );
        input_gates.push(input.clone());
        automatic_input_gates.push(input.clone());
    }

    let mut continue_load_operation = true;
    let mut propagate_signal = true;
    while continue_load_operation {
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }
        // println!("CLOCK TICK {}", get_clock_tick_number());

        continue_load_operation = run_circuit(
            &input_gates,
            &Vec::new(),
            propagate_signal,
            &mut |_clock_tick_inputs, _output_gates| {},
            None,
        );

        propagate_signal = false;
    }

    //Disconnect all inputs so that future connections can be made.
    for automatic_input_gate in automatic_input_gates.into_iter() {
        automatic_input_gate.borrow_mut().disconnect_gate(0);
    }

    clock.borrow_mut().disconnect_gate(0);

    //LOAD and MAR_S must be tied back to LOW before completing. They have already been
    // disconnected so the zero id is used.
    cpu.borrow_mut().update_input_signal(
        GateInput::new(
            load_index,
            LOW_,
            UniqueID::zero_id(),
        )
    );

    cpu.borrow_mut().update_input_signal(
        GateInput::new(
            memory_address_register_index,
            LOW_,
            UniqueID::zero_id(),
        )
    );

    let mut generated_output = generate_default_output(&cpu);

    for (i, binary_string) in binary_strings.iter().enumerate() {
        for (j, c) in binary_string.chars().rev().enumerate() {
            let output_tag = RAMUnit::get_ram_output_string(i, j);
            let output_index = cpu.borrow_mut().get_complex_gate().gate_tags_to_index[&output_tag.to_string()].index;

            let signal =
                if c == '0' {
                    LOW_
                } else {
                    HIGH
                };

            generated_output[output_index] = signal.clone();
        }
    }

    let collected_signals = collect_signals_from_logic_gate(cpu.clone());

    let failed = compare_generate_and_collected_output(&cpu, generated_output, collected_signals);

    assert!(!failed);
}

pub fn compare_generate_and_collected_output(
    cpu: &Rc<RefCell<VariableBitCPU>>,
    generated_output: Vec<Signal>,
    collected_signals: Vec<Signal>,
) -> bool {
    let tags_sorted_by_index = extract_output_tags_sorted_by_index(&cpu.borrow_mut().get_complex_gate());

    assert_eq!(collected_signals.len(), generated_output.len());
    assert_eq!(collected_signals.len(), tags_sorted_by_index.len());

    let mut failed = false;
    for i in 0..collected_signals.len() {
        let mut failed_map = HashMap::new();

        if (tags_sorted_by_index[i].clone(), generated_output[i].clone()) != (tags_sorted_by_index[i].clone(), collected_signals[i].clone()) {
            failed_map.insert(tags_sorted_by_index[i].clone(), (generated_output[i].clone(), collected_signals[i].clone()));
            failed = true;
        };

        if !failed_map.is_empty() {
            println!("E (passed, collected): {:?}", failed_map);
        }
    }
    failed
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
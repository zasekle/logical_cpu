use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::Nor;
use crate::logic::foundations::{ComplexGateMembers, ConnectedOutput, GateInput, GateLogic, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

pub struct SRLatch {
    complex_gate: ComplexGateMembers,
    top_nor_gate: Rc<RefCell<Nor>>,
    bottom_nor_gate: Rc<RefCell<Nor>>,
}

#[allow(dead_code)]
impl SRLatch {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_clone: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let reset_input_gate = SimpleInput::new("R");
        let set_input_gate = SimpleInput::new("S");
        let q_output_gate = SimpleOutput::new("Q");
        let nq_output_gate = SimpleOutput::new("~Q");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(set_input_gate.clone());
        input_gates.push(reset_input_gate.clone());

        output_gates.push(q_output_gate.clone());
        output_gates.push(nq_output_gate.clone());
        output_gates_clone.push(q_output_gate.clone());
        output_gates_clone.push(nq_output_gate.clone());

        let mut sr_latch = SRLatch {
            complex_gate: ComplexGateMembers::new(
                2,
                2,
                GateType::SRLatch,
                input_gates,
                output_gates,
            ),
            top_nor_gate: Nor::new(
                2, 2),
            bottom_nor_gate: Nor::new(
                2, 2),
        };

        sr_latch.build_and_prime_circuit(output_gates_clone);

        Rc::new(RefCell::new(sr_latch))
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let r_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("R")].clone();
        let mut r_input_gate = r_input_gate.borrow_mut();

        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();
        let mut s_input_gate = s_input_gate.borrow_mut();

        let q_output_gate = output_gates[self.get_index_from_tag("Q")].clone();
        let not_q_output_gate = output_gates[self.get_index_from_tag("~Q")].clone();

        let mut top_nor_gate = self.top_nor_gate.borrow_mut();
        let mut bottom_nor_gate = self.bottom_nor_gate.borrow_mut();

        r_input_gate.connect_output_to_next_gate(
            0,
            0,
            self.top_nor_gate.clone(),
        );

        s_input_gate.connect_output_to_next_gate(
            0,
            1,
            self.bottom_nor_gate.clone(),
        );

        top_nor_gate.connect_output_to_next_gate(
            0,
            0,
            q_output_gate.clone(),
        );

        top_nor_gate.connect_output_to_next_gate(
            1,
            0,
            self.bottom_nor_gate.clone(),
        );

        bottom_nor_gate.connect_output_to_next_gate(
            0,
            0,
            not_q_output_gate.clone(),
        );

        bottom_nor_gate.connect_output_to_next_gate(
            1,
            1,
            self.top_nor_gate.clone(),
        );

        drop(r_input_gate);
        drop(s_input_gate);
        drop(top_nor_gate);
        drop(bottom_nor_gate);

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(true);
    }
}

impl LogicGate for SRLatch {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        //Do not need to run calculate_output_from_inputs() here. It is run in simple gates for the
        // sake of getting the output. However, in my complex gates it can be time consuming.

        let signal = match &self.complex_gate.simple_gate.output_states[current_gate_output_key] {
            GateOutputState::NotConnected(signal) => {
                signal.clone()
            }
            GateOutputState::Connected(connected_output) => {
                connected_output.throughput.signal.clone()
            }
        };

        self.complex_gate.simple_gate.output_states[current_gate_output_key] =
            GateOutputState::Connected(
                ConnectedOutput {
                    throughput: GateInput::new(next_gate_input_key, signal),
                    gate: next_gate,
                }
            );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //Updating the inner 'input_signals' vector for consistency.
        self.complex_gate.simple_gate.update_input_signal(input.clone());

        let mut simple_input_gate = self.complex_gate.input_gates[input.input_index].borrow_mut();

        simple_input_gate.update_input_signal(
            GateInput {
                input_index: 0,
                signal: input.signal,
            }
        )
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.calculate_output_from_inputs(false);

        let output_clone = self.complex_gate.simple_gate.output_states.clone();

        if self.complex_gate.simple_gate.should_print_output {
            GateLogic::print_gate_output(
                &self.complex_gate.simple_gate.gate_type,
                &self.complex_gate.simple_gate.unique_id,
                &self.get_tag(),
                &output_clone,
            );
        }

        Ok(output_clone)
    }

    fn get_gate_type(&self) -> GateType {
        self.complex_gate.simple_gate.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.complex_gate.simple_gate.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.complex_gate.simple_gate.should_print_output = print_output;
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
    }
}

#[cfg(test)]
mod tests {
    use crate::globals::CLOCK_TICK_NUMBER;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::run_circuit;
    use super::*;

    fn run_sr_gate(
        r_input_signal: Vec<Signal>,
        s_input_signal: Vec<Signal>,
        q_output_signal: Vec<Signal>,
        not_q_output_signal: Vec<Signal>,
    ) {
        let r_input_gate = AutomaticInput::new(r_input_signal, 1, "Start_R");
        let s_input_gate = AutomaticInput::new(s_input_signal, 1, "Start_S");

        let q_output_gate = SimpleOutput::new("End_Q");
        let not_q_output_gate = SimpleOutput::new("End_~Q");

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        input_gates.push(r_input_gate.clone());
        input_gates.push(s_input_gate.clone());
        output_gates.push(q_output_gate.clone());
        output_gates.push(not_q_output_gate.clone());

        let sr_latch = SRLatch::new();
        // sr_latch.borrow_mut().toggle_output_printing(true);

        r_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            sr_latch.borrow_mut().get_index_from_tag("R"),
            sr_latch.clone(),
        );

        s_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            sr_latch.borrow_mut().get_index_from_tag("S"),
            sr_latch.clone(),
        );

        let mut mut_sr_latch = sr_latch.borrow_mut();
        let q_output_idx = mut_sr_latch.get_index_from_tag("Q");
        mut_sr_latch.connect_output_to_next_gate(
            q_output_idx,
            0,
            q_output_gate.clone(),
        );

        let not_q_output_idx = mut_sr_latch.get_index_from_tag("~Q");
        mut_sr_latch.connect_output_to_next_gate(
            not_q_output_idx,
            0,
            not_q_output_gate.clone(),
        );

        drop(mut_sr_latch);
        drop(not_q_output_idx);

        let mut collected_output: [Vec<Signal>; 2] = [Vec::new(), Vec::new()];
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
                    assert_eq!(output_gates.len(), 2);

                    let mut q_output = output_gates[0].borrow_mut();
                    let mut not_q_output = output_gates[1].borrow_mut();

                    let q_output = q_output.fetch_output_signals().unwrap();
                    let not_q_output = not_q_output.fetch_output_signals().unwrap();

                    assert_eq!(q_output.len(), 1);
                    assert_eq!(not_q_output.len(), 1);

                    let q_output = q_output.first().unwrap();
                    let not_q_output = not_q_output.first().unwrap();

                    match q_output {
                        GateOutputState::NotConnected(signal) => {
                            collected_output[0].push(signal.clone());
                        }
                        GateOutputState::Connected(_) => panic!("Final output gate should not be connected")
                    }

                    match not_q_output {
                        GateOutputState::NotConnected(signal) => {
                            collected_output[1].push(signal.clone());
                        }
                        GateOutputState::Connected(_) => panic!("Final output gate should not be connected")
                    }
                },
            );

            propagate_signal_through_circuit = false;
        }

        assert_eq!(collected_output[0], q_output_signal);
        assert_eq!(collected_output[1], not_q_output_signal);
    }

    #[test]
    fn sr_gate_initialization() {
        run_sr_gate(
            vec![LOW],
            vec![LOW],
            vec![LOW],
            vec![HIGH],
        );
    }

    #[test]
    fn sr_gate_low_low_after_low_high() {
        run_sr_gate(
            vec![LOW, LOW],
            vec![HIGH, LOW],
            vec![HIGH, HIGH],
            vec![LOW, LOW],
        );
    }

    #[test]
    fn sr_gate_low_low_after_high_low() {
        run_sr_gate(
            vec![HIGH, LOW],
            vec![LOW, LOW],
            vec![LOW, LOW],
            vec![HIGH, HIGH],
        );
    }

    #[test]
    fn sr_gate_low_high() {
        run_sr_gate(
            vec![LOW],
            vec![HIGH],
            vec![HIGH],
            vec![LOW],
        );
    }

    #[test]
    fn sr_gate_high_low() {
        run_sr_gate(
            vec![HIGH],
            vec![LOW],
            vec![LOW],
            vec![HIGH],
        );
    }

    #[test]
    fn sr_gate_high_high() {
        run_sr_gate(
            vec![HIGH],
            vec![HIGH],
            vec![LOW],
            vec![LOW],
        );
    }
}
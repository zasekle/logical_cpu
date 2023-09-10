use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::Nor;
use crate::logic::foundations::{ComplexGateMembers, ConnectedOutput, GateInput, GateLogic, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

pub struct SRLatch {
    complex_gate: ComplexGateMembers,
    set_input_gate: Rc<RefCell<SimpleInput>>,
    reset_input_gate: Rc<RefCell<SimpleInput>>,
    q_output_gate: Rc<RefCell<SimpleOutput>>,
    nq_output_gate: Rc<RefCell<SimpleOutput>>,
    top_nor_gate: Rc<RefCell<Nor>>,
    bottom_nor_gate: Rc<RefCell<Nor>>,
}

#[allow(dead_code)]
impl SRLatch {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        let reset_input_gate = SimpleInput::new("Reset");
        let set_input_gate = SimpleInput::new("Set");
        let q_output_gate = SimpleOutput::new("Q");
        let nq_output_gate = SimpleOutput::new("~Q");

        /// Order of gates is important here to force the circuit into a deterministic state.
        input_gates.push(reset_input_gate.clone());
        input_gates.push(set_input_gate.clone());
        output_gates.push(q_output_gate.clone());
        output_gates.push(nq_output_gate.clone());

        let mut sr_latch = SRLatch {
            complex_gate: ComplexGateMembers::new(
                2,
                2,
                GateType::SRLatch,
                input_gates,
                output_gates,
            ),
            set_input_gate,
            reset_input_gate,
            q_output_gate,
            nq_output_gate,
            top_nor_gate: Nor::new(
                2, 2),
            bottom_nor_gate: Nor::new(
                2, 2),
        };

        sr_latch.build_and_prime_circuit();

        Rc::new(RefCell::new(sr_latch))
    }

    fn build_and_prime_circuit(&mut self) {
        self.set_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.top_nor_gate.clone(),
        );

        self.reset_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.bottom_nor_gate.clone(),
        );

        let mut top_nor_gate = self.top_nor_gate.borrow_mut();
        let mut bottom_nor_gate = self.bottom_nor_gate.borrow_mut();

        top_nor_gate.connect_output_to_next_gate(
            0,
            0,
            self.q_output_gate.clone(),
        );

        top_nor_gate.connect_output_to_next_gate(
            1,
            0,
            self.bottom_nor_gate.clone(),
        );

        bottom_nor_gate.connect_output_to_next_gate(
            0,
            0,
            self.nq_output_gate.clone(),
        );

        bottom_nor_gate.connect_output_to_next_gate(
            1,
            1,
            self.top_nor_gate.clone(),
        );

        drop(top_nor_gate);
        drop(bottom_nor_gate);

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(true);
    }
}

impl LogicGate for SRLatch {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        //Do not need to run calculate_output_from_inputs() here. It is run in simple gates for the
        // sake of getting the output. However, in my complex gates it can be time consuming.

        let signal = match &self.complex_gate.simple_gate.output_states[current_gate_output_index] {
            GateOutputState::NotConnected(signal) => {
                signal.clone()
            }
            GateOutputState::Connected(connected_output) => {
                connected_output.throughput.signal.clone()
            }
        };

        self.complex_gate.simple_gate.output_states[current_gate_output_index] =
            GateOutputState::Connected(
                ConnectedOutput {
                    throughput: GateInput::new(next_gate_input_index, signal),
                    gate: next_gate,
                }
            );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //TODO: 2 problems
        // with a LOW, LOW input, the output should be HIGH, LOW

        //TODO: This will always return a 'current_clock_tick' of 0, the input gates need to properly handle this
        // if they are going to be inside a structure like this.
        //TODO: Make sure the other input and output gates handle these things appropriately as well.
        let mut input_gate = self.complex_gate.input_gates[input.input_index].borrow_mut();

        input_gate.update_input_signal(
            GateInput {
                input_index: 0,
                signal: input.signal
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::check_for_single_element_signal;
    use super::*;

    #[test]
    fn sr_gate() {}
}
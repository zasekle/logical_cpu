use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::{Nand, Nor};
use crate::logic::foundations::{ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW, HIGH};

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

        let reset_input_gate = SimpleInput::new(1, "R");
        let set_input_gate = SimpleInput::new(1, "S");
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
                GateType::SRLatchType,
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
        self.complex_gate.connect_output_to_next_gate(current_gate_output_key, next_gate_input_key, next_gate);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //SRLatch has an `invalid` state of HIGH HIGH. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(&self.get_tag())
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

pub struct ActiveLowSRLatch {
    complex_gate: ComplexGateMembers,
    top_nand_gate: Rc<RefCell<Nand>>,
    bottom_nand_gate: Rc<RefCell<Nand>>,
}

#[allow(dead_code)]
impl ActiveLowSRLatch {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_clone: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let reset_input_gate = SimpleInput::new(1, "R");
        let set_input_gate = SimpleInput::new(1, "S");
        let q_output_gate = SimpleOutput::new("Q");
        let nq_output_gate = SimpleOutput::new("~Q");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(set_input_gate.clone());
        input_gates.push(reset_input_gate.clone());

        output_gates.push(q_output_gate.clone());
        output_gates.push(nq_output_gate.clone());
        output_gates_clone.push(q_output_gate.clone());
        output_gates_clone.push(nq_output_gate.clone());

        let mut sr_latch = ActiveLowSRLatch {
            complex_gate: ComplexGateMembers::new(
                2,
                2,
                GateType::ActiveLowSRLatchType,
                input_gates,
                output_gates,
            ),
            top_nand_gate: Nand::new(
                2, 2),
            bottom_nand_gate: Nand::new(
                2, 2),
        };

        //An active low SR latch starts in the HIGH HIGH position.
        sr_latch.update_input_signal(
            GateInput {
                input_index: sr_latch.get_index_from_tag("R"),
                signal: HIGH,
            }
        );

        sr_latch.update_input_signal(
            GateInput {
                input_index: sr_latch.get_index_from_tag("S"),
                signal: HIGH,
            }
        );

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

        let mut top_nand_gate = self.top_nand_gate.borrow_mut();
        let mut bottom_nand_gate = self.bottom_nand_gate.borrow_mut();

        s_input_gate.connect_output_to_next_gate(
            0,
            0,
            self.top_nand_gate.clone(),
        );

        r_input_gate.connect_output_to_next_gate(
            0,
            1,
            self.bottom_nand_gate.clone(),
        );

        top_nand_gate.connect_output_to_next_gate(
            0,
            0,
            q_output_gate.clone(),
        );

        top_nand_gate.connect_output_to_next_gate(
            1,
            0,
            self.bottom_nand_gate.clone(),
        );

        bottom_nand_gate.connect_output_to_next_gate(
            0,
            0,
            not_q_output_gate.clone(),
        );

        bottom_nand_gate.connect_output_to_next_gate(
            1,
            1,
            self.top_nand_gate.clone(),
        );

        drop(r_input_gate);
        drop(s_input_gate);
        drop(top_nand_gate);
        drop(bottom_nand_gate);

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(true);
    }
}

impl LogicGate for ActiveLowSRLatch {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(current_gate_output_key, next_gate_input_key, next_gate);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(&self.get_tag())
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

pub struct OneBitMemoryCell {
    complex_gate: ComplexGateMembers,
    set_enable_nand_gate: Rc<RefCell<Nand>>,
    enable_nand_gate: Rc<RefCell<Nand>>,
    sr_top_nand_gate: Rc<RefCell<Nand>>,
    sr_bottom_nand_gate: Rc<RefCell<Nand>>,
}

#[allow(dead_code)]
impl OneBitMemoryCell {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        let set_input_gate = SimpleInput::new(1, "S");
        let enable_input_gate = SimpleInput::new(2, "E");
        let q_output_gate = SimpleOutput::new("Q");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(enable_input_gate.clone());
        input_gates.push(set_input_gate.clone());

        output_gates.push(q_output_gate.clone());

        let mut one_bit_memory_cell = OneBitMemoryCell {
            complex_gate: ComplexGateMembers::new(
                2,
                1,
                GateType::OneBitMemoryCellType,
                input_gates,
                output_gates,
            ),
            set_enable_nand_gate: Nand::new(
                2, 2,
            ),
            enable_nand_gate: Nand::new(
                2, 1,
            ),
            sr_top_nand_gate: Nand::new(
                2, 2,
            ),
            sr_bottom_nand_gate: Nand::new(
                2, 2,
            ),
        };

        //This will allow the circuit to be primed to the LOW output state.
        enable_input_gate.borrow_mut().update_input_signal(
            GateInput {
                input_index: 0,
                signal: HIGH,
            }
        );

        one_bit_memory_cell.build_and_prime_circuit(q_output_gate);

        enable_input_gate.borrow_mut().update_input_signal(
            GateInput {
                input_index: 0,
                signal: LOW,
            }
        );

        Rc::new(RefCell::new(one_bit_memory_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        q_output_gate: Rc<RefCell<SimpleOutput>>,
    ) {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();
        let mut e_input_gate = e_input_gate.borrow_mut();

        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();
        let mut s_input_gate = s_input_gate.borrow_mut();

        let mut set_enable_nand_gate = self.set_enable_nand_gate.borrow_mut();
        let mut enable_nand_gate = self.enable_nand_gate.borrow_mut();
        let mut sr_top_nand_gate = self.sr_top_nand_gate.borrow_mut();
        let mut sr_bottom_nand_gate = self.sr_bottom_nand_gate.borrow_mut();

        s_input_gate.connect_output_to_next_gate(
            0,
            0,
            self.set_enable_nand_gate.clone(),
        );

        e_input_gate.connect_output_to_next_gate(
            0,
            1,
            self.set_enable_nand_gate.clone(),
        );

        e_input_gate.connect_output_to_next_gate(
            1,
            1,
            self.enable_nand_gate.clone(),
        );

        set_enable_nand_gate.connect_output_to_next_gate(
            0,
            0,
            self.sr_top_nand_gate.clone(),
        );

        set_enable_nand_gate.connect_output_to_next_gate(
            1,
            0,
            self.enable_nand_gate.clone(),
        );

        enable_nand_gate.connect_output_to_next_gate(
            0,
            1,
            self.sr_bottom_nand_gate.clone(),
        );

        sr_top_nand_gate.connect_output_to_next_gate(
            0,
            1,
            self.sr_bottom_nand_gate.clone(),
        );

        sr_top_nand_gate.connect_output_to_next_gate(
            0,
            0,
            q_output_gate.clone(),
        );

        sr_top_nand_gate.connect_output_to_next_gate(
            1,
            0,
            self.sr_bottom_nand_gate.clone(),
        );

        sr_bottom_nand_gate.connect_output_to_next_gate(
            0,
            1,
            self.sr_top_nand_gate.clone(),
        );

        drop(e_input_gate);
        drop(s_input_gate);
        drop(set_enable_nand_gate);
        drop(enable_nand_gate);
        drop(sr_top_nand_gate);
        drop(sr_bottom_nand_gate);

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(true);
    }
}

impl LogicGate for OneBitMemoryCell {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(current_gate_output_key, next_gate_input_key, next_gate);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(&self.get_tag())
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

pub struct VariableBitMemoryCell {
    complex_gate: ComplexGateMembers,
    one_bit_memory_cells: Vec<Rc<RefCell<OneBitMemoryCell>>>,
}

#[allow(dead_code)]
impl VariableBitMemoryCell {
    pub fn new(number_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut one_bit_memory_cells: Vec<Rc<RefCell<OneBitMemoryCell>>> = Vec::new();

        for i in 0..number_bits {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            one_bit_memory_cells.push(
                OneBitMemoryCell::new()
            );
        }

        let set_input_gate = SimpleInput::new(number_bits, "S");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(set_input_gate.clone());

        let mut one_bit_memory_cell = VariableBitMemoryCell {
            complex_gate: ComplexGateMembers::new(
                number_bits + 1,
                number_bits,
                GateType::VariableBitMemoryCellType,
                input_gates,
                output_gates,
            ),
            one_bit_memory_cells,
        };

        one_bit_memory_cell.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(one_bit_memory_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();
        let mut s_input_gate = s_input_gate.borrow_mut();

        for i in 0..number_bits {
            let mut input_gate = self.complex_gate.input_gates[i].borrow_mut();
            let mut mem_gate = self.one_bit_memory_cells[i].borrow_mut();

            let enable_gate_index = mem_gate.get_index_from_tag("E");
            let set_gate_index = mem_gate.get_index_from_tag("S");

            input_gate.connect_output_to_next_gate(
                0,
                set_gate_index,
                self.one_bit_memory_cells[i].clone(),
            );

            s_input_gate.connect_output_to_next_gate(
                i,
                enable_gate_index,
                self.one_bit_memory_cells[i].clone(),
            );

            mem_gate.connect_output_to_next_gate(
                0,
                0,
                output_gates[i].clone(),
            );
        }

        drop(s_input_gate);

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(true);
    }
}

impl LogicGate for VariableBitMemoryCell {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(current_gate_output_key, next_gate_input_key, next_gate);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(&self.get_tag())
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
    use std::collections::HashMap;
    use crate::globals::CLOCK_TICK_NUMBER;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::run_circuit;
    use rand::Rng;
    use crate::test_stuff::run_multi_input_output_logic_gate;
    use super::*;

    fn run_sr_latch(
        r_input_signal: Vec<Signal>,
        s_input_signal: Vec<Signal>,
        q_output_signal: Vec<Signal>,
        not_q_output_signal: Vec<Signal>,
        sr_latch: Rc<RefCell<dyn LogicGate>>,
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

    fn run_one_bit_memory_cell(
        e_input_signal: Vec<Signal>,
        s_input_signal: Vec<Signal>,
        q_output_signal: Vec<Signal>,
    ) {
        let e_input_gate = AutomaticInput::new(e_input_signal, 1, "Start_E");
        let s_input_gate = AutomaticInput::new(s_input_signal, 1, "Start_S");

        let q_output_gate = SimpleOutput::new("End_Q");

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        input_gates.push(e_input_gate.clone());
        input_gates.push(s_input_gate.clone());
        output_gates.push(q_output_gate.clone());

        let one_bit_memory_cell = OneBitMemoryCell::new();

        e_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            one_bit_memory_cell.borrow_mut().get_index_from_tag("E"),
            one_bit_memory_cell.clone(),
        );

        s_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            one_bit_memory_cell.borrow_mut().get_index_from_tag("S"),
            one_bit_memory_cell.clone(),
        );

        let mut mut_one_bit_memory_cell = one_bit_memory_cell.borrow_mut();
        let q_output_idx = mut_one_bit_memory_cell.get_index_from_tag("Q");
        mut_one_bit_memory_cell.connect_output_to_next_gate(
            q_output_idx,
            0,
            q_output_gate.clone(),
        );

        drop(mut_one_bit_memory_cell);

        let mut collected_output: Vec<Signal> = Vec::new();
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
                    assert_eq!(output_gates.len(), 1);

                    let mut q_output = output_gates[0].borrow_mut();

                    let q_output = q_output.fetch_output_signals().unwrap();

                    assert_eq!(q_output.len(), 1);

                    let q_output = q_output.first().unwrap();

                    match q_output {
                        GateOutputState::NotConnected(signal) => {
                            collected_output.push(signal.clone());
                        }
                        GateOutputState::Connected(_) => panic!("Final output gate should not be connected"),
                    }
                },
            );

            propagate_signal_through_circuit = false;
        }

        assert_eq!(collected_output, q_output_signal);
    }

    #[test]
    fn sr_gate_initialization() {
        let sr_latch = SRLatch::new();

        let output = sr_latch.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), 2);

        let first_output = output.get(0).unwrap();
        let second_output = output.get(1).unwrap();

        match first_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, LOW),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }

        match second_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, HIGH),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }
    }

    #[test]
    fn sr_gate_low_low_after_low_high() {
        run_sr_latch(
            vec![LOW, LOW],
            vec![HIGH, LOW],
            vec![HIGH, HIGH],
            vec![LOW, LOW],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_low_low_after_high_low() {
        run_sr_latch(
            vec![HIGH, LOW],
            vec![LOW, LOW],
            vec![LOW, LOW],
            vec![HIGH, HIGH],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_low_high() {
        run_sr_latch(
            vec![LOW],
            vec![HIGH],
            vec![HIGH],
            vec![LOW],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_high_low() {
        run_sr_latch(
            vec![HIGH],
            vec![LOW],
            vec![LOW],
            vec![HIGH],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_high_high() {
        run_sr_latch(
            vec![HIGH],
            vec![HIGH],
            vec![LOW],
            vec![LOW],
            SRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_initialization() {
        let sr_latch = ActiveLowSRLatch::new();

        let output = sr_latch.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), 2);

        let first_output = output.get(0).unwrap();
        let second_output = output.get(1).unwrap();

        match first_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, HIGH),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }

        match second_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, LOW),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }
    }

    #[test]
    fn active_low_sr_gate_high_high_after_low_high() {
        run_sr_latch(
            vec![LOW, HIGH],
            vec![HIGH, HIGH],
            vec![LOW, LOW],
            vec![HIGH, HIGH],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_high_high_after_high_low() {
        run_sr_latch(
            vec![HIGH, HIGH],
            vec![LOW, HIGH],
            vec![HIGH, HIGH],
            vec![LOW, LOW],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_low_high() {
        run_sr_latch(
            vec![LOW],
            vec![HIGH],
            vec![LOW],
            vec![HIGH],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_high_low() {
        run_sr_latch(
            vec![HIGH],
            vec![LOW],
            vec![HIGH],
            vec![LOW],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_low_low() {
        run_sr_latch(
            vec![LOW],
            vec![LOW],
            vec![HIGH],
            vec![HIGH],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn one_bit_mem_initialization() {
        //initialization
        let one_bit_memory_cell = OneBitMemoryCell::new();

        let output = one_bit_memory_cell.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), 1);

        let first_output = output.first().unwrap();

        match first_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, LOW),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }
    }

    #[test]
    fn one_bit_mem_low_high() {
        run_one_bit_memory_cell(
            vec![LOW],
            vec![HIGH],
            vec![LOW],
        );
    }

    #[test]
    fn one_bit_mem_high_low() {
        run_one_bit_memory_cell(
            vec![HIGH],
            vec![LOW],
            vec![LOW],
        );
    }

    #[test]
    fn one_bit_mem_high_high() {
        run_one_bit_memory_cell(
            vec![HIGH],
            vec![HIGH],
            vec![HIGH],
        );
    }

    #[test]
    fn one_bit_mem_high_high_to_low_low() {
        run_one_bit_memory_cell(
            vec![HIGH, LOW],
            vec![HIGH, LOW],
            vec![HIGH, HIGH],
        );
    }

    #[test]
    fn one_bit_mem_low_high_to_low_low() {
        run_one_bit_memory_cell(
            vec![LOW, LOW],
            vec![HIGH, LOW],
            vec![LOW, LOW],
        );
    }

    #[test]
    fn one_bit_mem_high_low_to_low_low() {
        run_one_bit_memory_cell(
            vec![HIGH, LOW],
            vec![LOW, LOW],
            vec![LOW, LOW],
        );
    }

    #[test]
    fn one_bit_mem_low_low_to_low_low() {
        run_one_bit_memory_cell(
            vec![LOW, LOW],
            vec![LOW, LOW],
            vec![LOW, LOW],
        );
    }

    #[test]
    fn one_bit_mem_high_high_to_high_low() {
        run_one_bit_memory_cell(
            vec![HIGH, HIGH],
            vec![HIGH, LOW],
            vec![HIGH, LOW],
        );
    }

    #[test]
    fn one_bit_mem_high_low_to_high_high() {
        run_one_bit_memory_cell(
            vec![HIGH, HIGH],
            vec![LOW, HIGH],
            vec![LOW, HIGH],
        );
    }

    #[test]
    fn variable_bit_mem_initialization() {
        let num_bits = rand::thread_rng().gen_range(2..=16);
        let variable_bit_memory_cell = VariableBitMemoryCell::new(num_bits);

        let output = variable_bit_memory_cell.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), num_bits);
        for out in output {
            match out {
                GateOutputState::NotConnected(signal) => {
                    assert_eq!(signal, LOW);
                }
                GateOutputState::Connected(_) => panic!("Final output gate should never be connected.")
            }
        }
    }

    #[test]
    fn variable_bit_signal_high() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW, HIGH],
            ],
            vec![
                vec![HIGH, LOW, HIGH],
            ],
            HashMap::from(
                [("S",vec![HIGH])]
            ),
            VariableBitMemoryCell::new(3),
        );
    }

    #[test]
    fn variable_bit_signal_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW],
            ],
            vec![
                vec![LOW, LOW],
            ],
            HashMap::from(
                [("S",vec![LOW])]
            ),
            VariableBitMemoryCell::new(2),
        );
    }

    #[test]
    fn variable_bit_saved_states() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH, LOW],
                vec![HIGH, HIGH, LOW],
                vec![HIGH, HIGH, LOW],
                vec![LOW, LOW, HIGH],
            ],
            vec![
                vec![LOW, LOW, LOW],
                vec![HIGH, HIGH, LOW],
                vec![HIGH, HIGH, LOW],
                vec![LOW, LOW, HIGH],
            ],
            HashMap::from(
                [("S",vec![LOW, HIGH, LOW, HIGH])]
            ),
            VariableBitMemoryCell::new(3),
        );
    }
}
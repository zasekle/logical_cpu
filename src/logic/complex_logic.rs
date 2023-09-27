use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::{And, Nand, Not, Or, Splitter};

#[allow(unused_imports)]
use crate::logic::foundations::{BasicGateMembers, build_simple_inputs_and_outputs, build_simple_inputs_and_outputs_with_and, calculate_input_signals_from_all_inputs, ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};

use crate::logic::foundations::GateType::OneBitMemoryCellType;
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW_, HIGH};
use crate::logic::memory_gates::OneBitMemoryCell;

pub struct VariableOutputStepper {
    pub complex_gate: ComplexGateMembers,
    mem_cells: Vec<Rc<RefCell<OneBitMemoryCell>>>,
    output_and_gates: Vec<Rc<RefCell<And>>>,
    output_not_gates: Vec<Rc<RefCell<Not>>>,
    output_or_gate: Rc<RefCell<Or>>,
    clk_top_or_gate: Rc<RefCell<Or>>,
    clk_bottom_or_gate: Rc<RefCell<Or>>,
    clk_bottom_not_gate: Rc<RefCell<Not>>,
    mem_one_not_gate: Rc<RefCell<Not>>,
}

#[allow(dead_code)]
impl VariableOutputStepper {
    pub fn new(number_outputs: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_outputs, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_and_gates: Vec<Rc<RefCell<And>>> = Vec::new();
        let mut output_not_gates: Vec<Rc<RefCell<Not>>> = Vec::new();
        let mut mem_cells: Vec<Rc<RefCell<OneBitMemoryCell>>> = Vec::new();

        for i in 0..number_outputs {
            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            output_not_gates.push(
                Not::new(1)
            );

            if i != 0 {
                output_and_gates.push(
                    And::new(2, 1)
                );

                mem_cells.push(
                    OneBitMemoryCell::new(1)
                );

                mem_cells.push(
                    OneBitMemoryCell::new(3)
                );
            }
        }

        mem_cells.push(
            OneBitMemoryCell::new(1)
        );

        mem_cells.push(
            OneBitMemoryCell::new(5)
        );

        let enable_input_gate = SimpleInput::new(2, "CLK");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(enable_input_gate.clone());

        let mut variable_output_stepper = VariableOutputStepper {
            complex_gate: ComplexGateMembers::new(
                1,
                number_outputs,
                GateType::VariableOutputStepperType,
                input_gates,
                output_gates,
            ),
            mem_cells,
            output_and_gates,
            output_not_gates,
            output_or_gate: Or::new(2, 1),
            clk_top_or_gate: Or::new(2, number_outputs),
            clk_bottom_or_gate: Or::new(2, number_outputs),
            clk_bottom_not_gate: Not::new(1),
            mem_one_not_gate: Not::new(1),
        };

        variable_output_stepper.build_and_prime_circuit(
            number_outputs,
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_output_stepper))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_outputs: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        self.output_or_gate.borrow_mut().update_input_signal(
            GateInput::new(
                1,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        let clk_input = self.complex_gate.input_gates[self.get_index_from_tag("CLK")].clone();

        clk_input.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.clk_top_or_gate.clone(),
        );

        clk_input.borrow_mut().connect_output_to_next_gate(
            1,
            0,
            self.clk_bottom_not_gate.clone(),
        );

        self.clk_bottom_not_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.clk_bottom_or_gate.clone(),
        );

        let mem_cell_set_index = self.mem_cells[0].borrow_mut().get_index_from_tag("S");
        self.mem_one_not_gate.borrow_mut().connect_output_to_next_gate(
            0,
            mem_cell_set_index,
            self.mem_cells[0].clone(),
        );

        let skip_last_mem_gate = number_outputs * 2 - 1;
        for i in 0..skip_last_mem_gate {
            let mem_cell_output_index = self.mem_cells[i].borrow_mut().get_index_from_tag("Q");
            let mem_cell_set_index = self.mem_cells[i + 1].borrow_mut().get_index_from_tag("S");
            self.mem_cells[i].borrow_mut().connect_output_to_next_gate(
                mem_cell_output_index,
                mem_cell_set_index,
                self.mem_cells[i + 1].clone(),
            );

            if i % 2 == 0 {
                let mem_cell_enable_index = self.mem_cells[i].borrow_mut().get_index_from_tag("E");
                self.clk_top_or_gate.borrow_mut().connect_output_to_next_gate(
                    i / 2,
                    mem_cell_enable_index,
                    self.mem_cells[i].clone(),
                );
            } else {
                let idx = i / 2;
                let mem_cell_enable_index = self.mem_cells[i].borrow_mut().get_index_from_tag("E");
                self.clk_bottom_or_gate.borrow_mut().connect_output_to_next_gate(
                    idx,
                    mem_cell_enable_index,
                    self.mem_cells[i].clone(),
                );

                let mem_cell_output_index = self.mem_cells[i].borrow_mut().get_index_from_tag("Q_1");
                self.mem_cells[i].borrow_mut().connect_output_to_next_gate(
                    mem_cell_output_index,
                    0,
                    self.output_not_gates[idx].clone(),
                );

                let mem_cell_output_index = self.mem_cells[i].borrow_mut().get_index_from_tag("Q_2");
                self.mem_cells[i].borrow_mut().connect_output_to_next_gate(
                    mem_cell_output_index,
                    0,
                    self.output_and_gates[idx].clone(),
                );

                let next_gate: Rc<RefCell<dyn LogicGate>> =
                    if idx == 0 {
                        let or_gate = self.output_or_gate.clone();
                        or_gate.borrow_mut().connect_output_to_next_gate(
                            0,
                            0,
                            output_gates[idx].clone()
                        );
                        or_gate
                    } else {
                        let and_gate = self.output_and_gates[idx - 1].clone();
                        and_gate.borrow_mut().connect_output_to_next_gate(
                            0,
                            0,
                            output_gates[idx].clone()
                        );
                        and_gate
                    };

                self.output_not_gates[idx].borrow_mut().connect_output_to_next_gate(
                    0,
                    1,
                    next_gate,
                );
            }
        }

        let final_mem_cell_idx = self.mem_cells.len() - 1;
        let final_not_idx = self.output_not_gates.len() - 1;
        let final_and_idx = self.output_and_gates.len() - 1;

        self.output_and_gates[final_and_idx].borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[number_outputs - 1].clone()
        );

        self.output_not_gates[final_not_idx].borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.output_and_gates[final_and_idx].clone(),
        );

        let mem_cell_enable_index = self.mem_cells[final_mem_cell_idx].borrow_mut().get_index_from_tag("E");
        self.clk_bottom_or_gate.borrow_mut().connect_output_to_next_gate(
            number_outputs - 1,
            mem_cell_enable_index,
            self.mem_cells[final_mem_cell_idx].clone(),
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].borrow_mut().get_index_from_tag("Q");
        self.mem_cells[final_mem_cell_idx].borrow_mut().connect_output_to_next_gate(
            mem_cell_output_index,
            0,
            self.output_not_gates[final_not_idx].clone(),
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].borrow_mut().get_index_from_tag("Q_1");
        self.mem_cells[final_mem_cell_idx].borrow_mut().connect_output_to_next_gate(
            mem_cell_output_index,
            0,
            self.output_or_gate.clone()
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].borrow_mut().get_index_from_tag("Q_2");
        self.mem_cells[final_mem_cell_idx].borrow_mut().connect_output_to_next_gate(
            mem_cell_output_index,
            0,
            self.mem_one_not_gate.clone(),
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].borrow_mut().get_index_from_tag("Q_3");
        self.mem_cells[final_mem_cell_idx].borrow_mut().connect_output_to_next_gate(
            mem_cell_output_index,
            0,
            self.clk_top_or_gate.clone()
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].borrow_mut().get_index_from_tag("Q_4");
        self.mem_cells[final_mem_cell_idx].borrow_mut().connect_output_to_next_gate(
            mem_cell_output_index,
            0,
            self.clk_bottom_or_gate.clone()
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            Some(OneBitMemoryCellType),
        );
    }
}

impl LogicGate for VariableOutputStepper {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
            Some(OneBitMemoryCellType),
        )
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

    fn get_tag(&self) -> String {
        self.complex_gate.simple_gate.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.complex_gate.simple_gate.tag = tag.to_string();
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

pub struct VariableBitCPUEnable {
    complex_gate: ComplexGateMembers,
    and_gates: Vec<Rc<RefCell<And>>>,
}

#[allow(dead_code)]
impl VariableBitCPUEnable {
    pub fn new(number_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut one_bit_memory_cells: Vec<Rc<RefCell<And>>> = Vec::new();

        build_simple_inputs_and_outputs_with_and(
            number_bits,
            &mut input_gates,
            &mut output_gates,
            &mut output_gates_logic,
            &mut one_bit_memory_cells,
        );

        let enable_input_gate = SimpleInput::new(number_bits, "E");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(enable_input_gate.clone());

        let mut one_bit_memory_cell = VariableBitCPUEnable {
            complex_gate: ComplexGateMembers::new(
                number_bits + 1,
                number_bits,
                GateType::VariableCPUEnableType,
                input_gates,
                output_gates,
            ),
            and_gates: one_bit_memory_cells,
        };

        one_bit_memory_cell.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(one_bit_memory_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();

        for i in 0..number_bits {
            self.complex_gate.input_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                self.and_gates[i].clone(),
            );

            e_input_gate.borrow_mut().connect_output_to_next_gate(
                i,
                1,
                self.and_gates[i].clone(),
            );

            self.and_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                output_gates[i].clone(),
            );
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for VariableBitCPUEnable {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
            None,
        )
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

    fn get_tag(&self) -> String {
        self.complex_gate.simple_gate.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.complex_gate.simple_gate.tag = tag.to_string();
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

//Note that this is not a 'real' logic gate. Its purpose is to stop the signal from propagating to
// increase performance of the simulation.
pub struct SignalGatekeeper {
    complex_gate: ComplexGateMembers,
}

#[allow(dead_code)]
impl SignalGatekeeper {
    pub fn new(number_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        build_simple_inputs_and_outputs(
            number_bits,
            &mut input_gates,
            &mut output_gates,
            &mut output_gates_logic,
        );

        let enable_input_gate = SimpleInput::new(number_bits, "E");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(enable_input_gate.clone());

        let mut single_gate_keeper = SignalGatekeeper {
            complex_gate: ComplexGateMembers::new(
                number_bits + 1,
                number_bits,
                GateType::SignalGatekeeperType,
                input_gates,
                output_gates,
            ),
        };

        single_gate_keeper.build_and_prime_circuit(
            number_bits,
            output_gates_logic,
        );

        Rc::new(RefCell::new(single_gate_keeper))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..number_bits {
            self.complex_gate.input_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                output_gates[i].clone(),
            );
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for SignalGatekeeper {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();

        let e_output = e_input_gate.borrow_mut().fetch_output_signals()?;

        //The SimpleInput only has one output.
        let output = e_output.first().unwrap();

        let e_signal =
            match output {
                GateOutputState::NotConnected(signal) => {
                    signal
                }
                GateOutputState::Connected(connected_output) => panic!("The enable of SignalGatekeeper is never meant to be connected {:?}.", connected_output)
            };

        if *e_signal == HIGH { //Gate is enabled.
            self.complex_gate.fetch_output_signals(
                &self.get_tag(),
                None,
            )
        } else {
            let input_signals = calculate_input_signals_from_all_inputs(&self.complex_gate.simple_gate.input_signals)?;
            let mut output = Vec::new();
            for input in input_signals {
                output.push(
                    GateOutputState::NotConnected(input)
                );
            }
            Ok(output)
        }
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

    fn get_tag(&self) -> String {
        self.complex_gate.simple_gate.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.complex_gate.simple_gate.tag = tag.to_string();
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

pub struct MasterSlaveJKFlipFlop {
    complex_gate: ComplexGateMembers,
    j_input_nand: Rc<RefCell<Nand>>,
    k_input_nand: Rc<RefCell<Nand>>,
    q1_output_nand: Rc<RefCell<Nand>>,
    q1_not_output_nand: Rc<RefCell<Nand>>,
    q1_input_nand: Rc<RefCell<Nand>>,
    q1_not_input_nand: Rc<RefCell<Nand>>,
    q_output_nand: Rc<RefCell<Nand>>,
    q_not_output_nand: Rc<RefCell<Nand>>,
    not_gate: Rc<RefCell<Not>>,
}

#[allow(dead_code)]
impl MasterSlaveJKFlipFlop {

    //Inputs
    pub const J: &'static str = "J";
    pub const CLK_IN: &'static str = "CLK_IN";
    pub const K: &'static str = "K";

    //Outputs
    pub const Q: &'static str = "Q";
    pub const NOT_Q: &'static str = "NOT_Q";

    pub fn new() -> Rc<RefCell<Self>> {

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        input_gates.push(SimpleInput::new(1, MasterSlaveJKFlipFlop::J));
        input_gates.push(SimpleInput::new(3, MasterSlaveJKFlipFlop::CLK_IN));
        input_gates.push(SimpleInput::new(1, MasterSlaveJKFlipFlop::K));

        let mut store_output = |gate: Rc<RefCell<SimpleOutput>>| {
            output_gates.push(gate.clone());
            output_gates_logic.push(gate.clone());
        };

        store_output(SimpleOutput::new(MasterSlaveJKFlipFlop::Q));
        store_output(SimpleOutput::new(MasterSlaveJKFlipFlop::NOT_Q));

        //Order of input gates is important here to force the circuit into a deterministic state.

        let mut flip_flop = MasterSlaveJKFlipFlop {
            complex_gate: ComplexGateMembers::new(
                3,
                2,
                GateType::MasterSlaveJKFlipFlopType,
                input_gates,
                output_gates,
            ),
            j_input_nand: Nand::new(3, 1),
            k_input_nand: Nand::new(3, 1),
            q1_output_nand: Nand::new(2, 2),
            q1_not_output_nand: Nand::new(2,2),
            q1_input_nand: Nand::new(2, 1),
            q1_not_input_nand: Nand::new(2,1),
            q_output_nand: Nand::new(2, 3),
            q_not_output_nand: Nand::new(2, 3),
            not_gate: Not::new(2),
        };

        flip_flop.j_input_nand.borrow_mut().set_tag("j_input_nand");
        flip_flop.k_input_nand.borrow_mut().set_tag("k_input_nand");
        flip_flop.q1_output_nand.borrow_mut().set_tag("q1_output_nand");
        flip_flop.q1_not_output_nand.borrow_mut().set_tag("q1_not_output_nand");
        flip_flop.q1_input_nand.borrow_mut().set_tag("q1_input_nand");
        flip_flop.q1_not_input_nand.borrow_mut().set_tag("q1_not_input_nand");
        flip_flop.q_output_nand.borrow_mut().set_tag("q_output_nand");
        flip_flop.q_not_output_nand.borrow_mut().set_tag("q_not_output_nand");
        flip_flop.not_gate.borrow_mut().set_tag("not_gate");

        flip_flop.build_and_prime_circuit(output_gates_logic);

        Rc::new(RefCell::new(flip_flop))
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {

        //CLK_IN input
        let clk_input_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::CLK_IN);
        let clk_input_gate = self.complex_gate.input_gates[clk_input_index].clone();

        clk_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            2,
            self.j_input_nand.clone(),
        );

        clk_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            0,
            self.k_input_nand.clone(),
        );

        clk_input_gate.borrow_mut().connect_output_to_next_gate(
            2,
            0,
            self.not_gate.clone(),
        );

        //J input
        let j_input_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::J);
        let j_input_gate = self.complex_gate.input_gates[j_input_index].clone();

        j_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.j_input_nand.clone()
        );

        //K input
        let k_input_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::K);
        let k_input_gate = self.complex_gate.input_gates[k_input_index].clone();

        k_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.k_input_nand.clone()
        );

        self.j_input_nand.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.q1_output_nand.clone(),
        );

        self.k_input_nand.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.q1_not_output_nand.clone(),
        );

        //q1 output nand
        self.q1_output_nand.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.q1_input_nand.clone()
        );

        self.q1_output_nand.borrow_mut().connect_output_to_next_gate(
            1,
            0,
            self.q1_not_output_nand.clone(),
        );

        //q1 not output nand
        self.q1_not_output_nand.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.q1_not_input_nand.clone()
        );

        self.q1_not_output_nand.borrow_mut().connect_output_to_next_gate(
            1,
            1,
            self.q1_output_nand.clone(),
        );

        //not gate
        self.not_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.q1_input_nand.clone(),
        );

        self.not_gate.borrow_mut().connect_output_to_next_gate(
            1,
            0,
            self.q1_not_input_nand.clone(),
        );

        //q1 input nand
        self.q1_input_nand.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.q_output_nand.clone(),
        );

        //q1 not input nand
        self.q1_not_input_nand.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.q_not_output_nand.clone(),
        );

        //q output nand
        let output_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::Q);
        self.q_output_nand.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[output_index].clone(),
        );

        self.q_output_nand.borrow_mut().connect_output_to_next_gate(
            1,
            0,
            self.q_not_output_nand.clone(),
        );

        self.q_output_nand.borrow_mut().connect_output_to_next_gate(
            2,
            2,
            self.k_input_nand.clone(),
        );

        let output_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::NOT_Q);
        self.q_not_output_nand.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[output_index].clone(),
        );

        self.q_not_output_nand.borrow_mut().connect_output_to_next_gate(
            1,
            1,
            self.q_output_nand.clone()
        );

        self.q_not_output_nand.borrow_mut().connect_output_to_next_gate(
            2,
            0,
            self.j_input_nand.clone()
        );

        clk_input_gate.borrow_mut().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for MasterSlaveJKFlipFlop {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
            None,
        )
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

    fn get_tag(&self) -> String {
        self.complex_gate.simple_gate.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.complex_gate.simple_gate.tag = tag.to_string();
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}

pub struct FourCycleClockHookup {
    complex_gate: ComplexGateMembers,
    flip_flop: Rc<RefCell<MasterSlaveJKFlipFlop>>,
    q_splitter: Rc<RefCell<Splitter>>,
    q_input_and: Rc<RefCell<And>>,
    q_not_input_nand: Rc<RefCell<Nand>>,
    flip_flop_clk_in_not: Rc<RefCell<Not>>,
}

#[allow(dead_code)]
impl FourCycleClockHookup {
    //Inputs
    pub const CLK_IN: &'static str = "CLK_IN";

    //Outputs
    pub const CLK_OUT: &'static str = "CLK_OUT";
    pub const CLKS: &'static str = "CLKS";
    pub const CLKE: &'static str = "CLKE";

    pub fn new() -> Rc<RefCell<Self>> {

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let input_gate = SimpleInput::new(3, FourCycleClockHookup::CLK_IN);

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(input_gate.clone());

        let mut store_output = |gate: Rc<RefCell<SimpleOutput>>| {
            output_gates.push(gate.clone());
            output_gates_logic.push(gate.clone());
        };

        store_output(SimpleOutput::new(FourCycleClockHookup::CLK_OUT));
        store_output(SimpleOutput::new(FourCycleClockHookup::CLKS));
        store_output(SimpleOutput::new(FourCycleClockHookup::CLKE));

        let mut clock_hookup = FourCycleClockHookup {
            complex_gate: ComplexGateMembers::new(
                1,
                3,
                GateType::FourCycleClockHookupType,
                input_gates,
                output_gates,
            ),
            flip_flop: MasterSlaveJKFlipFlop::new(),
            q_splitter: Splitter::new(1, 2),
            q_input_and: And::new(2, 1),
            q_not_input_nand: Nand::new(2, 1),
            flip_flop_clk_in_not: Not::new(1),
        };

        clock_hookup.flip_flop.borrow_mut().set_tag("flip_flop");
        clock_hookup.q_splitter.borrow_mut().set_tag("q_splitter");
        clock_hookup.q_input_and.borrow_mut().set_tag("q_input_and");
        clock_hookup.q_not_input_nand.borrow_mut().set_tag("q_not_input_nand");
        clock_hookup.flip_flop_clk_in_not.borrow_mut().set_tag("flip_flop_clk_in_not");

        clock_hookup.build_and_prime_circuit(
            output_gates_logic
        );

        Rc::new(RefCell::new(clock_hookup))
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {

        //Set J to high
        let j_input_index = self.flip_flop.borrow_mut().get_index_from_tag(MasterSlaveJKFlipFlop::J);
        self.flip_flop.borrow_mut().update_input_signal(
            GateInput::new(
                j_input_index,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        //Set K to high
        let k_input_index = self.flip_flop.borrow_mut().get_index_from_tag(MasterSlaveJKFlipFlop::K);
        self.flip_flop.borrow_mut().update_input_signal(
            GateInput::new(
                k_input_index,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        //CLK_IN input
        let clk_input_index = self.get_index_from_tag(FourCycleClockHookup::CLK_IN);
        let clk_input_gate = self.complex_gate.input_gates[clk_input_index].clone();
        clk_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.q_input_and.clone(),
        );

        clk_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            1,
            self.q_not_input_nand.clone(),
        );

        clk_input_gate.borrow_mut().connect_output_to_next_gate(
            2,
            0,
            self.flip_flop_clk_in_not.clone(),
        );

        let clk_input_index = self.flip_flop.borrow_mut().get_index_from_tag(MasterSlaveJKFlipFlop::CLK_IN);
        self.flip_flop_clk_in_not.borrow_mut().connect_output_to_next_gate(
            0,
            clk_input_index,
            self.flip_flop.clone(),
        );

        let q_output_index = self.flip_flop.borrow_mut().get_index_from_tag(MasterSlaveJKFlipFlop::Q);
        self.flip_flop.borrow_mut().connect_output_to_next_gate(
            q_output_index,
            0,
            self.q_splitter.clone(),
        );

        let output_index = self.q_splitter.borrow_mut().get_index_for_output(0, 0);
        self.q_splitter.borrow_mut().connect_output_to_next_gate(
            output_index,
            1,
            self.q_input_and.clone(),
        );

        let output_index = self.get_index_from_tag(FourCycleClockHookup::CLK_OUT);
        let splitter_output_index = self.q_splitter.borrow_mut().get_index_for_output(0, 1);
        self.q_splitter.borrow_mut().connect_output_to_next_gate(
            splitter_output_index,
            1,
            output_gates[output_index].clone(),
        );

        let not_q_output_index = self.flip_flop.borrow_mut().get_index_from_tag(MasterSlaveJKFlipFlop::NOT_Q);
        self.flip_flop.borrow_mut().connect_output_to_next_gate(
            not_q_output_index,
            0,
            self.q_not_input_nand.clone(),
        );

        let output_index = self.get_index_from_tag(FourCycleClockHookup::CLKS);
        self.q_input_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[output_index].clone(),
        );

        let output_index = self.get_index_from_tag(FourCycleClockHookup::CLKE);
        self.q_not_input_nand.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[output_index].clone(),
        );

        #[cfg(feature = "high_restriction")]
        self.check_output();

        let mut input_signal = LOW_;
        for i in 0..4 {
            if input_signal == LOW_ {
                input_signal = HIGH;
            } else {
                input_signal = LOW_;
            }

            clk_input_gate.borrow_mut().update_input_signal(
                GateInput::new(
                    0,
                    input_signal.clone(),
                    UniqueID::zero_id(),
                )
            );

            self.complex_gate.calculate_output_from_inputs(
                if i == 0 {true} else {false},
                None,
            );
        }

        clk_input_gate.borrow_mut().update_input_signal(
            GateInput::new(
                0,
                LOW_,
                UniqueID::zero_id(),
            )
        );
    }

    #[cfg(feature = "high_restriction")]
    fn check_output(&self) {
        println!("Running high_restriction!");
        fn check_output(simple_gate: &BasicGateMembers) {
            for o in simple_gate.output_states.iter() {
                if let GateOutputState::NotConnected(_) = o {
                    panic!("A gate output for type {} id {} was not set\n{:#?}", simple_gate.gate_type, simple_gate.unique_id.id(), simple_gate.output_states);
                }
            }

            for i in simple_gate.input_signals.iter() {
                if i.len() == 0 {
                    panic!("A gate input for type {} id {} was empty", simple_gate.gate_type, simple_gate.unique_id.id());
                } else if i.len() == 1 {
                    let (id, _signal) = i.iter().next().unwrap();
                    if id.id() == 0 {
                        panic!("A gate input for type {} id {} was empty\n{:#?}", simple_gate.gate_type, simple_gate.unique_id.id(), simple_gate.input_signals);
                    }
                } else {
                    panic!("Multiple inputs connected for the same gate")
                }
            }
        }

        check_output(&self.j_input_nand.borrow_mut().members);
        check_output(&self.k_input_nand.borrow_mut().members);
        check_output(&self.q1_output_nand.borrow_mut().members);
        check_output(&self.q1_not_output_nand.borrow_mut().members);
        check_output(&self.q1_input_nand.borrow_mut().members);
        check_output(&self.q1_not_input_nand.borrow_mut().members);
        check_output(&self.q_output_nand.borrow_mut().members);
        check_output(&self.q_not_output_nand.borrow_mut().members);
        check_output(&self.q_input_nand.borrow_mut().members);
        check_output(&self.q_not_input_nand.borrow_mut().members);
        check_output(&self.jk_flip_flop_not_gate.borrow_mut().members);
        check_output(&self.q_output_not.borrow_mut().members);
    }
}

impl LogicGate for FourCycleClockHookup {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
            None,
        )
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

    fn get_tag(&self) -> String {
        self.complex_gate.simple_gate.tag.clone()
    }

    fn set_tag(&mut self, tag: &str) {
        self.complex_gate.simple_gate.tag = tag.to_string();
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        self.complex_gate.get_index_from_tag(tag)
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::logic::foundations::Signal::{HIGH, LOW_};
    use rand::Rng;
    use crate::test_stuff::run_multi_input_output_logic_gate;
    use super::*;

    #[test]
    fn cpu_enable_initialization() {
        let num_bits = rand::thread_rng().gen_range(1..=16);
        let cpu_enable = VariableBitCPUEnable::new(num_bits);

        let output = cpu_enable.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), num_bits);
        for out in output {
            match out {
                GateOutputState::NotConnected(signal) => {
                    assert_eq!(signal, LOW_);
                }
                GateOutputState::Connected(_) => panic!("Final output gate should never be connected.")
            }
        }
    }

    #[test]
    fn cpu_enable_inputs_change() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH, LOW_],
                vec![HIGH, HIGH, LOW_],
                vec![LOW_, HIGH, LOW_],
            ],
            vec![
                vec![HIGH, HIGH, LOW_],
                vec![LOW_, LOW_, LOW_],
                vec![LOW_, HIGH, LOW_],
            ],
            HashMap::from(
                [("E", vec![vec![HIGH], vec![LOW_], vec![HIGH]])]
            ),
            VariableBitCPUEnable::new(3),
        );
    }

    #[test]
    fn signal_gatekeeper_tests() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH, LOW_],
                vec![HIGH, HIGH, LOW_],
            ],
            vec![
                vec![LOW_, LOW_, LOW_],
                vec![HIGH, HIGH, LOW_],
            ],
            HashMap::from(
                [("E", vec![vec![LOW_], vec![HIGH]])]
            ),
            SignalGatekeeper::new(3),
        );
    }

    #[test]
    fn variable_output_stepper_tests() {
        run_multi_input_output_logic_gate(
            vec![],
            vec![
                vec![HIGH, LOW_, LOW_, LOW_, LOW_, LOW_], //0
                vec![HIGH, LOW_, LOW_, LOW_, LOW_, LOW_],
                vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_], //1
                vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_],
                vec![LOW_, LOW_, HIGH, LOW_, LOW_, LOW_], //2
                vec![LOW_, LOW_, HIGH, LOW_, LOW_, LOW_],
                vec![LOW_, LOW_, LOW_, HIGH, LOW_, LOW_], //3
                vec![LOW_, LOW_, LOW_, HIGH, LOW_, LOW_],
                vec![LOW_, LOW_, LOW_, LOW_, HIGH, LOW_], //4
                vec![LOW_, LOW_, LOW_, LOW_, HIGH, LOW_],
                vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH], //5
                vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH],
                vec![HIGH, LOW_, LOW_, LOW_, LOW_, LOW_], //0
            ],
            HashMap::from(
                [("CLK", vec![
                    vec![LOW_], //0
                    vec![HIGH],
                    vec![LOW_], //1
                    vec![HIGH],
                    vec![LOW_], //2
                    vec![HIGH],
                    vec![LOW_], //3
                    vec![HIGH],
                    vec![LOW_], //4
                    vec![HIGH],
                    vec![LOW_], //5
                    vec![HIGH],
                    vec![LOW_], //0
                ])]
            ),
            VariableOutputStepper::new(6),
        );
    }

    #[test]
    fn master_slave_jk_flip_flop_initialization() {
        let flip_flop = MasterSlaveJKFlipFlop::new();

        let clock_output = flip_flop.borrow_mut().fetch_output_signals().unwrap();

        let mut output_signals = Vec::new();
        for out in clock_output {
            match out {
                GateOutputState::NotConnected(signal) => {
                    output_signals.push(signal);
                }
                GateOutputState::Connected(_) => panic!("Clock should not be connected to anything")
            }
        }

        assert_eq!(
            output_signals,
            vec![HIGH, LOW_],
        );
    }

    #[test]
    fn master_slave_jk_flip_flop_run() {
        run_multi_input_output_logic_gate(
            vec![],
            vec![
                vec![HIGH, LOW_,], //Q ~Q
                vec![LOW_, HIGH,], //Q ~Q
                vec![LOW_, HIGH,], //Q ~Q
                vec![HIGH, LOW_,], //Q ~Q
                vec![HIGH, LOW_,], //Q ~Q
            ],
            HashMap::from(
                //He has a positive edge flip flop
                [(MasterSlaveJKFlipFlop::CLK_IN, vec![vec![HIGH], vec![LOW_], vec![HIGH], vec![LOW_], vec![HIGH]]),
                    (MasterSlaveJKFlipFlop::J, vec![vec![HIGH], vec![HIGH], vec![HIGH], vec![HIGH], vec![HIGH]]),
                    (MasterSlaveJKFlipFlop::K, vec![vec![HIGH], vec![HIGH], vec![HIGH], vec![HIGH], vec![HIGH]]),
                ]
            ),
            MasterSlaveJKFlipFlop::new(),
        );
    }

    #[test]
    fn four_cycle_clock_hookup_initialization() {
        let clock = FourCycleClockHookup::new();

        let clock_output = clock.borrow_mut().fetch_output_signals().unwrap();

        let mut output_signals = Vec::new();
        for out in clock_output {
            match out {
                GateOutputState::NotConnected(signal) => {
                    output_signals.push(signal);
                }
                GateOutputState::Connected(_) => panic!("Clock should not be connected to anything")
            }
        }

        assert_eq!(
            output_signals,
            vec![LOW_, LOW_, HIGH],
        );
    }

    #[test]
    fn four_cycle_clock_hookup_run() {
        let clock_hookup = FourCycleClockHookup::new();
        clock_hookup.borrow_mut().toggle_output_printing(true);
        run_multi_input_output_logic_gate(
            vec![],
            vec![
                vec![HIGH, HIGH, HIGH], //CLK, CLKS, CLKE
                vec![HIGH, LOW_, HIGH],
                vec![LOW_, LOW_, LOW_],
                vec![LOW_, LOW_, HIGH],
                vec![HIGH, HIGH, HIGH],
            ],
            HashMap::from(
                //He has a positive edge flip flop
                [(FourCycleClockHookup::CLK_IN, vec![vec![HIGH], vec![LOW_], vec![HIGH], vec![LOW_], vec![HIGH]])]
            ),
            clock_hookup,
        );
    }
}
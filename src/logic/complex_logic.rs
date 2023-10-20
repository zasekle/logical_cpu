use crate::logic::basic_gates::{And, Nand, Not, Or, Splitter};

#[allow(unused_imports)]
use crate::logic::foundations::{BasicGateMembers, build_simple_inputs_and_outputs, build_simple_inputs_and_outputs_with_and, calculate_input_signals_from_all_inputs, ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::foundations::connect_gates;

use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW_, HIGH};
use crate::logic::memory_gates::OneBitMemoryCell;
use crate::shared_mutex::{new_shared_mutex, SharedMutex};

pub struct VariableOutputStepper {
    pub complex_gate: ComplexGateMembers,
    mem_cells: Vec<SharedMutex<OneBitMemoryCell>>,
    output_and_gates: Vec<SharedMutex<And>>,
    output_not_gates: Vec<SharedMutex<Not>>,
    output_or_gate: SharedMutex<Or>,
    clk_top_or_gate: SharedMutex<Or>,
    clk_bottom_or_gate: SharedMutex<Or>,
    clk_bottom_not_gate: SharedMutex<Not>,
    mem_one_not_gate: SharedMutex<Not>,
}

#[allow(dead_code)]
impl VariableOutputStepper {
    pub fn new(number_outputs: usize) -> SharedMutex<Self> {
        assert_ne!(number_outputs, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_and_gates: Vec<SharedMutex<And>> = Vec::new();
        let mut output_not_gates: Vec<SharedMutex<Not>> = Vec::new();
        let mut mem_cells: Vec<SharedMutex<OneBitMemoryCell>> = Vec::new();

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

        // for input in input_gates.iter() {
        //     variable_output_stepper.complex_gate.simple_gate.number_child_gates += input.lock().unwrap().num_children_gates();
        // }
        //
        // for output in output_gates.iter() {
        //     variable_output_stepper.complex_gate.simple_gate.number_child_gates += output.lock().unwrap().num_children_gates();
        // }
        //
        // for mem_cell in variable_output_stepper.mem_cells.iter() {
        //     variable_output_stepper.complex_gate.simple_gate.number_child_gates += mem_cell.lock().unwrap().num_children_gates();
        // }
        //
        // for and_gate in variable_output_stepper.output_and_gates.iter() {
        //     variable_output_stepper.complex_gate.simple_gate.number_child_gates += and_gate.lock().unwrap().num_children_gates();
        // }
        //
        // for or_gate in variable_output_stepper.output_not_gates.iter() {
        //     variable_output_stepper.complex_gate.simple_gate.number_child_gates += or_gate.lock().unwrap().num_children_gates();
        // }
        //
        // variable_output_stepper.complex_gate.simple_gate.number_child_gates +=
        //     variable_output_stepper.output_or_gate.lock().unwrap().num_children_gates();
        //
        // variable_output_stepper.complex_gate.simple_gate.number_child_gates +=
        //     variable_output_stepper.clk_top_or_gate.lock().unwrap().num_children_gates();
        //
        // variable_output_stepper.complex_gate.simple_gate.number_child_gates +=
        //     variable_output_stepper.clk_bottom_or_gate.lock().unwrap().num_children_gates();
        //
        // variable_output_stepper.complex_gate.simple_gate.number_child_gates +=
        //     variable_output_stepper.clk_bottom_not_gate.lock().unwrap().num_children_gates();
        //
        // variable_output_stepper.complex_gate.simple_gate.number_child_gates +=
        //     variable_output_stepper.mem_one_not_gate.lock().unwrap().num_children_gates();

        variable_output_stepper.build_and_prime_circuit(
            number_outputs,
            output_gates_logic,
        );

        new_shared_mutex(variable_output_stepper)
    }

    fn build_and_prime_circuit(
        &mut self,
        number_outputs: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        self.output_or_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                1,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        let clk_input = self.complex_gate.input_gates[self.get_index_from_tag("CLK")].clone();

        connect_gates(
            clk_input.clone(),
            0,
            self.clk_top_or_gate.clone(),
            1,
        );

        connect_gates(
            clk_input.clone(),
            1,
            self.clk_bottom_not_gate.clone(),
            0,
        );

        connect_gates(
            self.clk_bottom_not_gate.clone(),
            0,
            self.clk_bottom_or_gate.clone(),
            1,
        );

        let mem_cell_set_index = self.mem_cells[0].lock().unwrap().get_index_from_tag("S");
        connect_gates(
            self.mem_one_not_gate.clone(),
            0,
            self.mem_cells[0].clone(),
            mem_cell_set_index,
        );

        let skip_last_mem_gate = number_outputs * 2 - 1;
        for i in 0..skip_last_mem_gate {
            let mem_cell_output_index = self.mem_cells[i].lock().unwrap().get_index_from_tag("Q");
            let mem_cell_set_index = self.mem_cells[i + 1].lock().unwrap().get_index_from_tag("S");
            connect_gates(
                self.mem_cells[i].clone(),
                mem_cell_output_index,
                self.mem_cells[i + 1].clone(),
                mem_cell_set_index,
            );

            if i % 2 == 0 {
                let mem_cell_enable_index = self.mem_cells[i].lock().unwrap().get_index_from_tag("E");
                connect_gates(
                    self.clk_top_or_gate.clone(),
                    i / 2,
                    self.mem_cells[i].clone(),
                    mem_cell_enable_index,
                );
            } else {
                let idx = i / 2;
                let mem_cell_enable_index = self.mem_cells[i].lock().unwrap().get_index_from_tag("E");
                connect_gates(
                    self.clk_bottom_or_gate.clone(),
                    idx,
                    self.mem_cells[i].clone(),
                    mem_cell_enable_index,
                );

                let mem_cell_output_index = self.mem_cells[i].lock().unwrap().get_index_from_tag("Q_1");
                connect_gates(
                    self.mem_cells[i].clone(),
                    mem_cell_output_index,
                    self.output_not_gates[idx].clone(),
                    0,
                );

                let mem_cell_output_index = self.mem_cells[i].lock().unwrap().get_index_from_tag("Q_2");
                connect_gates(
                    self.mem_cells[i].clone(),
                    mem_cell_output_index,
                    self.output_and_gates[idx].clone(),
                    0,
                );

                let next_gate: SharedMutex<dyn LogicGate> =
                    if idx == 0 {
                        let or_gate = self.output_or_gate.clone();
                        connect_gates(
                            or_gate.clone(),
                            0,
                            output_gates[idx].clone(),
                            0,
                        );
                        or_gate
                    } else {
                        let and_gate = self.output_and_gates[idx - 1].clone();
                        connect_gates(
                            and_gate.clone(),
                            0,
                            output_gates[idx].clone(),
                            0,
                        );
                        and_gate
                    };

                connect_gates(
                    self.output_not_gates[idx].clone(),
                    0,
                    next_gate.clone(),
                    1,
                );
            }
        }

        let final_mem_cell_idx = self.mem_cells.len() - 1;
        let final_not_idx = self.output_not_gates.len() - 1;
        let final_and_idx = self.output_and_gates.len() - 1;

        connect_gates(
            self.output_and_gates[final_and_idx].clone(),
            0,
            output_gates[number_outputs - 1].clone(),
            0,
        );

        connect_gates(
            self.output_not_gates[final_not_idx].clone(),
            0,
            self.output_and_gates[final_and_idx].clone(),
            1,
        );

        let mem_cell_enable_index = self.mem_cells[final_mem_cell_idx].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.clk_bottom_or_gate.clone(),
            number_outputs - 1,
            self.mem_cells[final_mem_cell_idx].clone(),
            mem_cell_enable_index,
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].lock().unwrap().get_index_from_tag("Q");
        connect_gates(
            self.mem_cells[final_mem_cell_idx].clone(),
            mem_cell_output_index,
            self.output_not_gates[final_not_idx].clone(),
            0,
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].lock().unwrap().get_index_from_tag("Q_1");
        connect_gates(
            self.mem_cells[final_mem_cell_idx].clone(),
            mem_cell_output_index,
            self.output_or_gate.clone(),
            0,
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].lock().unwrap().get_index_from_tag("Q_2");
        connect_gates(
            self.mem_cells[final_mem_cell_idx].clone(),
            mem_cell_output_index,
            self.mem_one_not_gate.clone(),
            0,
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].lock().unwrap().get_index_from_tag("Q_3");
        connect_gates(
            self.mem_cells[final_mem_cell_idx].clone(),
            mem_cell_output_index,
            self.clk_top_or_gate.clone(),
            0,
        );

        let mem_cell_output_index = self.mem_cells[final_mem_cell_idx].lock().unwrap().get_index_from_tag("Q_4");
        connect_gates(
            self.mem_cells[final_mem_cell_idx].clone(),
            mem_cell_output_index,
            self.clk_bottom_or_gate.clone(),
            0,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for VariableOutputStepper {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.complex_gate.connect_output(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.complex_gate.simple_gate.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        self.complex_gate.input_gates.clone()
    }
}

pub struct VariableBitCPUEnable {
    complex_gate: ComplexGateMembers,
    and_gates: Vec<SharedMutex<And>>,
}

#[allow(dead_code)]
impl VariableBitCPUEnable {
    pub fn new(number_bits: usize) -> SharedMutex<Self> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut one_bit_memory_cells: Vec<SharedMutex<And>> = Vec::new();

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

        new_shared_mutex(one_bit_memory_cell)
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();

        for i in 0..number_bits {
            connect_gates(
                self.complex_gate.input_gates[i].clone(),
                0,
                self.and_gates[i].clone(),
                0,
            );

            connect_gates(
                e_input_gate.clone(),
                i,
                self.and_gates[i].clone(),
                1,
            );

            connect_gates(
                self.and_gates[i].clone(),
                0,
                output_gates[i].clone(),
                0,
            );
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for VariableBitCPUEnable {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.complex_gate.connect_output(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.complex_gate.simple_gate.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        self.complex_gate.input_gates.clone()
    }
}

//Note that this is not a 'real' logic gate. Its purpose is to stop the signal from propagating to
// increase performance of the simulation.
pub struct SignalGatekeeper {
    complex_gate: ComplexGateMembers,
}

#[allow(dead_code)]
impl SignalGatekeeper {
    pub fn new(number_bits: usize) -> SharedMutex<Self> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(single_gate_keeper)
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        for i in 0..number_bits {
            connect_gates(
                self.complex_gate.input_gates[i].clone(),
                0,
                output_gates[i].clone(),
                0,
            );
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for SignalGatekeeper {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.complex_gate.connect_output(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();

        let e_output = e_input_gate.lock().unwrap().fetch_output_signals()?;

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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.complex_gate.simple_gate.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        self.complex_gate.input_gates.clone()
    }
}

pub struct MasterSlaveJKFlipFlop {
    complex_gate: ComplexGateMembers,
    j_input_nand: SharedMutex<Nand>,
    k_input_nand: SharedMutex<Nand>,
    q1_output_nand: SharedMutex<Nand>,
    q1_not_output_nand: SharedMutex<Nand>,
    q1_input_nand: SharedMutex<Nand>,
    q1_not_input_nand: SharedMutex<Nand>,
    q_output_nand: SharedMutex<Nand>,
    q_not_output_nand: SharedMutex<Nand>,
    not_gate: SharedMutex<Not>,
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

    pub fn new() -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        input_gates.push(SimpleInput::new(1, MasterSlaveJKFlipFlop::J));
        input_gates.push(SimpleInput::new(3, MasterSlaveJKFlipFlop::CLK_IN));
        input_gates.push(SimpleInput::new(1, MasterSlaveJKFlipFlop::K));

        let mut store_output = |gate: SharedMutex<SimpleOutput>| {
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
            q1_not_output_nand: Nand::new(2, 2),
            q1_input_nand: Nand::new(2, 1),
            q1_not_input_nand: Nand::new(2, 1),
            q_output_nand: Nand::new(2, 3),
            q_not_output_nand: Nand::new(2, 3),
            not_gate: Not::new(2),
        };

        flip_flop.j_input_nand.lock().unwrap().set_tag("j_input_nand");
        flip_flop.k_input_nand.lock().unwrap().set_tag("k_input_nand");
        flip_flop.q1_output_nand.lock().unwrap().set_tag("q1_output_nand");
        flip_flop.q1_not_output_nand.lock().unwrap().set_tag("q1_not_output_nand");
        flip_flop.q1_input_nand.lock().unwrap().set_tag("q1_input_nand");
        flip_flop.q1_not_input_nand.lock().unwrap().set_tag("q1_not_input_nand");
        flip_flop.q_output_nand.lock().unwrap().set_tag("q_output_nand");
        flip_flop.q_not_output_nand.lock().unwrap().set_tag("q_not_output_nand");
        flip_flop.not_gate.lock().unwrap().set_tag("not_gate");

        flip_flop.build_and_prime_circuit(output_gates_logic);

        new_shared_mutex(flip_flop)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {

        //CLK_IN input
        let clk_input_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::CLK_IN);
        let clk_input_gate = self.complex_gate.input_gates[clk_input_index].clone();

        connect_gates(
            clk_input_gate.clone(),
            0,
            self.j_input_nand.clone(),
            2,
        );

        connect_gates(
            clk_input_gate.clone(),
            1,
            self.k_input_nand.clone(),
            0,
        );

        connect_gates(
            clk_input_gate.clone(),
            2,
            self.not_gate.clone(),
            0,
        );

        //J input
        let j_input_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::J);
        let j_input_gate = self.complex_gate.input_gates[j_input_index].clone();

        connect_gates(
            j_input_gate.clone(),
            0,
            self.j_input_nand.clone(),
            1,
        );

        //K input
        let k_input_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::K);
        let k_input_gate = self.complex_gate.input_gates[k_input_index].clone();

        connect_gates(
            k_input_gate.clone(),
            0,
            self.k_input_nand.clone(),
            1,
        );

        connect_gates(
            self.j_input_nand.clone(),
            0,
            self.q1_output_nand.clone(),
            0,
        );

        connect_gates(
            self.k_input_nand.clone(),
            0,
            self.q1_not_output_nand.clone(),
            1,
        );

        //q1 output nand
        connect_gates(
            self.q1_output_nand.clone(),
            0,
            self.q1_input_nand.clone(),
            0,
        );

        connect_gates(
            self.q1_output_nand.clone(),
            1,
            self.q1_not_output_nand.clone(),
            0,
        );

        //q1 not output nand
        connect_gates(
            self.q1_not_output_nand.clone(),
            0,
            self.q1_not_input_nand.clone(),
            1,
        );

        connect_gates(
            self.q1_not_output_nand.clone(),
            1,
            self.q1_output_nand.clone(),
            1,
        );

        //not gate
        connect_gates(
            self.not_gate.clone(),
            0,
            self.q1_input_nand.clone(),
            1,
        );

        connect_gates(
            self.not_gate.clone(),
            1,
            self.q1_not_input_nand.clone(),
            0,
        );

        //q1 input nand
        connect_gates(
            self.q1_input_nand.clone(),
            0,
            self.q_output_nand.clone(),
            0,
        );

        //q1 not input nand
        connect_gates(
            self.q1_not_input_nand.clone(),
            0,
            self.q_not_output_nand.clone(),
            1,
        );

        //q output nand
        let output_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::Q);
        connect_gates(
            self.q_output_nand.clone(),
            0,
            output_gates[output_index].clone(),
            0,
        );

        connect_gates(
            self.q_output_nand.clone(),
            1,
            self.q_not_output_nand.clone(),
            0,
        );

        connect_gates(
            self.q_output_nand.clone(),
            2,
            self.k_input_nand.clone(),
            2,
        );

        let output_index = self.get_index_from_tag(MasterSlaveJKFlipFlop::NOT_Q);
        connect_gates(
            self.q_not_output_nand.clone(),
            0,
            output_gates[output_index].clone(),
            0,
        );

        connect_gates(
            self.q_not_output_nand.clone(),
            1,
            self.q_output_nand.clone(),
            1,
        );

        connect_gates(
            self.q_not_output_nand.clone(),
            2,
            self.j_input_nand.clone(),
            0,
        );

        clk_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for MasterSlaveJKFlipFlop {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.complex_gate.connect_output(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.complex_gate.simple_gate.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        self.complex_gate.input_gates.clone()
    }
}

pub struct FourCycleClockHookup {
    complex_gate: ComplexGateMembers,
    flip_flop: SharedMutex<MasterSlaveJKFlipFlop>,
    q_splitter: SharedMutex<Splitter>,
    q_input_and: SharedMutex<And>,
    q_not_input_nand: SharedMutex<Nand>,
    flip_flop_clk_in_not: SharedMutex<Not>,
}

#[allow(dead_code)]
impl FourCycleClockHookup {
    //Inputs
    pub const CLK_IN: &'static str = "CLK_IN";

    //Outputs
    pub const CLK_OUT: &'static str = "CLK_OUT";
    pub const CLKS: &'static str = "CLKS";
    pub const CLKE: &'static str = "CLKE";

    pub fn new() -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        let input_gate = SimpleInput::new(3, FourCycleClockHookup::CLK_IN);

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(input_gate.clone());

        let mut store_output = |gate: SharedMutex<SimpleOutput>| {
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

        clock_hookup.flip_flop.lock().unwrap().set_tag("flip_flop");
        clock_hookup.q_splitter.lock().unwrap().set_tag("q_splitter");
        clock_hookup.q_input_and.lock().unwrap().set_tag("q_input_and");
        clock_hookup.q_not_input_nand.lock().unwrap().set_tag("q_not_input_nand");
        clock_hookup.flip_flop_clk_in_not.lock().unwrap().set_tag("flip_flop_clk_in_not");

        clock_hookup.build_and_prime_circuit(
            output_gates_logic
        );

        new_shared_mutex(clock_hookup)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {

        //Set J to high
        let j_input_index = self.flip_flop.lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::J);
        self.flip_flop.lock().unwrap().update_input_signal(
            GateInput::new(
                j_input_index,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        //Set K to high
        let k_input_index = self.flip_flop.lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::K);
        self.flip_flop.lock().unwrap().update_input_signal(
            GateInput::new(
                k_input_index,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        //CLK_IN input
        let clk_input_index = self.get_index_from_tag(FourCycleClockHookup::CLK_IN);
        let clk_input_gate = self.complex_gate.input_gates[clk_input_index].clone();
        connect_gates(
            clk_input_gate.clone(),
            0,
            self.q_input_and.clone(),
            0,
        );

        connect_gates(
            clk_input_gate.clone(),
            1,
            self.q_not_input_nand.clone(),
            1,
        );

        connect_gates(
            clk_input_gate.clone(),
            2,
            self.flip_flop_clk_in_not.clone(),
            0,
        );

        let clk_input_index = self.flip_flop.lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::CLK_IN);
        connect_gates(
            self.flip_flop_clk_in_not.clone(),
            0,
            self.flip_flop.clone(),
            clk_input_index,
        );

        let q_output_index = self.flip_flop.lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::Q);
        connect_gates(
            self.flip_flop.clone(),
            q_output_index,
            self.q_splitter.clone(),
            0,
        );

        let output_index = self.q_splitter.lock().unwrap().get_index_for_output(0, 0);
        connect_gates(
            self.q_splitter.clone(),
            output_index,
            self.q_input_and.clone(),
            1,
        );

        let output_index = self.get_index_from_tag(FourCycleClockHookup::CLK_OUT);
        let splitter_output_index = self.q_splitter.lock().unwrap().get_index_for_output(0, 1);
        connect_gates(
            self.q_splitter.clone(),
            splitter_output_index,
            output_gates[output_index].clone(),
            1,
        );

        let not_q_output_index = self.flip_flop.lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::NOT_Q);
        connect_gates(
            self.flip_flop.clone(),
            not_q_output_index,
            self.q_not_input_nand.clone(),
            0,
        );

        let output_index = self.get_index_from_tag(FourCycleClockHookup::CLKS);
        connect_gates(
            self.q_input_and.clone(),
            0,
            output_gates[output_index].clone(),
            0,
        );

        let output_index = self.get_index_from_tag(FourCycleClockHookup::CLKE);
        connect_gates(
            self.q_not_input_nand.clone(),
            0,
            output_gates[output_index].clone(),
            0,
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

            clk_input_gate.lock().unwrap().update_input_signal(
                GateInput::new(
                    0,
                    input_signal.clone(),
                    UniqueID::zero_id(),
                )
            );

            self.complex_gate.calculate_output_from_inputs_and_set_child_count(
                if i == 0 { true } else { false },
            );
        }

        clk_input_gate.lock().unwrap().update_input_signal(
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

        //J and K on the flip flop are tied high
        // check_output(&self.flip_flop.lock().unwrap().complex_gate.simple_gate);
        check_output(&self.q_splitter.lock().unwrap().members);
        check_output(&self.q_input_and.lock().unwrap().members);
        check_output(&self.q_not_input_nand.lock().unwrap().members);
        check_output(&self.flip_flop_clk_in_not.lock().unwrap().members);
    }
}

impl LogicGate for FourCycleClockHookup {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.complex_gate.connect_output(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.complex_gate.simple_gate.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        self.complex_gate.input_gates.clone()
    }
}

pub struct VariableBitMultiplexer {
    complex_gate: ComplexGateMembers,
    input_and_gates: Vec<SharedMutex<And>>,
    input_or_gates: Vec<SharedMutex<Or>>,
    control_lines: Vec<SharedMutex<Not>>,
}

#[allow(dead_code)]
impl VariableBitMultiplexer {
    pub fn new(bus_size: usize, number_inputs: usize) -> SharedMutex<Self> {
        assert_ne!(bus_size, 0);
        assert_ne!(number_inputs, 0);

        let num_control_lines = number_inputs.ilog2() as usize;

        assert_eq!(usize::pow(2, num_control_lines as u32), number_inputs);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        let mut input_and_gates: Vec<SharedMutex<And>> = Vec::new();
        let mut input_or_gates: Vec<SharedMutex<Or>> = Vec::new();
        let mut control_lines: Vec<SharedMutex<Not>> = Vec::new();


        let loop_num = bus_size * number_inputs;
        for i in 0..loop_num {
            let and_gate = And::new(num_control_lines + 1, 1);
            and_gate.lock().unwrap().set_tag(format!("and_{}", i).as_str());
            input_and_gates.push(and_gate);
        }

        for i in 0..bus_size {
            let or_gate = Or::new(number_inputs, 1);
            or_gate.lock().unwrap().set_tag(format!("or_{}", i).as_str());
            input_or_gates.push(or_gate);
        }

        for i in 0..number_inputs {
            for j in 0..bus_size {
                let input_tag = format!("I_{}_bit_{}", i, j);
                input_gates.push(SimpleInput::new(1, input_tag.as_str()));
            }
        }

        for i in 0..num_control_lines {
            let not_gate = Not::new((number_inputs * bus_size) / 2);
            not_gate.lock().unwrap().set_tag(format!("not_{}", i).as_str());
            control_lines.push(not_gate);

            let input_tag = format!("C_{}", i);
            input_gates.push(SimpleInput::new((number_inputs * bus_size) / 2 + 1, input_tag.as_str()));
        }

        let mut store_output = |gate: SharedMutex<SimpleOutput>| {
            output_gates.push(gate.clone());
            output_gates_logic.push(gate.clone());
        };

        for i in 0..bus_size {
            let output_tag = format!("o_{}", i);
            store_output(SimpleOutput::new(output_tag.as_str()));
        }

        let mut multiplexer = VariableBitMultiplexer {
            complex_gate: ComplexGateMembers::new(
                num_control_lines + bus_size * number_inputs,
                bus_size,
                GateType::VariableBitMultiplexerType,
                input_gates,
                output_gates,
            ),
            input_and_gates,
            input_or_gates,
            control_lines,
        };

        multiplexer.build_and_prime_circuit(
            bus_size,
            number_inputs,
            num_control_lines,
            output_gates_logic,
        );

        new_shared_mutex(multiplexer)
    }

    fn build_and_prime_circuit(
        &mut self,
        bus_size: usize,
        number_inputs: usize,
        num_control_lines: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let mut not_current_index = vec![0; num_control_lines];
        let mut normal_current_index = vec![0; num_control_lines];

        //Connect input control lines to not gates.
        for i in 0..num_control_lines {
            let control_input_tag = format!("C_{}", i);
            let control_index = self.get_index_from_tag(control_input_tag.as_str());
            let control_input_gate = self.complex_gate.input_gates[control_index].clone();

            connect_gates(
                control_input_gate.clone(),
                normal_current_index[i],
                self.control_lines[i].clone(),
                0,
            );

            normal_current_index[i] += 1;
        }

        //Connect inputs, control lines, and not gates to and gates.
        for i in 0..number_inputs {
            let binary_number = format!("{:0width$b}", i, width = num_control_lines);
            assert_eq!(binary_number.len(), num_control_lines);
            for j in 0..bus_size {
                let input_tag = format!("I_{}_bit_{}", i, j);
                let input_index = self.get_index_from_tag(input_tag.as_str());
                let input_gate = self.complex_gate.input_gates[input_index].clone();

                let and_gate_index = j * number_inputs + i;
                connect_gates(
                    input_gate.clone(),
                    0,
                    self.input_and_gates[and_gate_index].clone(),
                    0,
                );

                for (k, c) in binary_number.chars().enumerate() {
                    if c == '0' {
                        //Not input to and gate.
                        connect_gates(
                            self.control_lines[k].clone(),
                            not_current_index[k],
                            self.input_and_gates[and_gate_index].clone(),
                            k + 1,
                        );
                        not_current_index[k] += 1;
                    } else {
                        //Normal input to and gate
                        let control_input_tag = format!("C_{}", k);
                        let control_index = self.get_index_from_tag(control_input_tag.as_str());
                        let control_input_gate = self.complex_gate.input_gates[control_index].clone();

                        connect_gates(
                            control_input_gate.clone(),
                            normal_current_index[k],
                            self.input_and_gates[and_gate_index].clone(),
                            k + 1,
                        );

                        normal_current_index[k] += 1;
                    }
                }
            }
        }

        for i in 0..self.input_and_gates.len() {
            let or_gate_index = i / number_inputs;
            let next_gate_input_key = i % number_inputs;

            connect_gates(
                self.input_and_gates[i].clone(),
                0,
                self.input_or_gates[or_gate_index].clone(),
                next_gate_input_key,
            );
        };

        for i in 0..bus_size {
            let output_tag = format!("o_{}", i);
            let output_index = self.get_index_from_tag(output_tag.as_str());
            connect_gates(
                self.input_or_gates[i].clone(),
                0,
                output_gates[output_index].clone(),
                0,
            );
        }

        #[cfg(feature = "high_restriction")]
        self.check_output();

        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
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

        for i in self.input_and_gates.iter() {
            check_output(&i.lock().unwrap().members);
        }

        for i in self.input_or_gates.iter() {
            check_output(&i.lock().unwrap().members);
        }

        for i in self.control_lines.iter() {
            check_output(&i.lock().unwrap().members);
        }
    }
}

impl LogicGate for VariableBitMultiplexer {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.complex_gate.connect_output(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.complex_gate.simple_gate.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        self.complex_gate.input_gates.clone()
    }
}

pub struct VariableBitCounter {
    complex_gate: ComplexGateMembers,
    flip_flops: Vec<SharedMutex<MasterSlaveJKFlipFlop>>,
    flip_flop_clk_in_not_gates: Vec<SharedMutex<Not>>,
    flip_flop_output_splitter: SharedMutex<Splitter>,
}

#[allow(dead_code)]
impl VariableBitCounter {
    //Inputs
    pub const CLK_IN: &'static str = "CLK_IN";

    pub fn new(num_output_pins: usize) -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        let input_gate = SimpleInput::new(1, Self::CLK_IN);

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(input_gate.clone());

        let mut flip_flops = Vec::new();
        let mut flip_flop_clk_in_not_gates = Vec::new();

        let mut store_output = |gate: SharedMutex<SimpleOutput>| {
            output_gates.push(gate.clone());
            output_gates_logic.push(gate.clone());
        };

        for i in 0..num_output_pins {
            let output_tag = format!("o_{}", i);
            store_output(SimpleOutput::new(output_tag.as_str()));

            flip_flops.push(MasterSlaveJKFlipFlop::new());
            flip_flop_clk_in_not_gates.push(Not::new(1));
        }

        let mut clock_hookup = VariableBitCounter {
            complex_gate: ComplexGateMembers::new(
                1,
                num_output_pins,
                GateType::VariableBitCounterType,
                input_gates,
                output_gates,
            ),
            flip_flops,
            flip_flop_clk_in_not_gates,
            flip_flop_output_splitter: Splitter::new(num_output_pins, 2),
        };

        for i in 0..num_output_pins {
            clock_hookup.flip_flops[i].lock().unwrap().set_tag(format!("flip_flop_{}", i).as_str());
            clock_hookup.flip_flop_clk_in_not_gates[i].lock().unwrap().set_tag(format!("flip_flop_clk_in_not_{}", i).as_str());
        }

        clock_hookup.build_and_prime_circuit(
            output_gates_logic,
            num_output_pins,
        );

        new_shared_mutex(clock_hookup)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
        num_output_pins: usize,
    ) {
        for i in 0..num_output_pins {
            //Set J to high
            let j_input_index = self.flip_flops[i].lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::J);
            self.flip_flops[i].lock().unwrap().update_input_signal(
                GateInput::new(
                    j_input_index,
                    HIGH,
                    UniqueID::zero_id(),
                )
            );

            //Set K to high
            let k_input_index = self.flip_flops[i].lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::K);
            self.flip_flops[i].lock().unwrap().update_input_signal(
                GateInput::new(
                    k_input_index,
                    HIGH,
                    UniqueID::zero_id(),
                )
            );

            let not_q_output_index = self.flip_flops[i].lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::NOT_Q);
            connect_gates(
                self.flip_flops[i].clone(),
                not_q_output_index,
                self.flip_flop_output_splitter.clone(),
                i,
            );

            if i == 0 {
                //CLK_IN input
                let clk_input_index = self.get_index_from_tag(Self::CLK_IN);
                let clk_input_gate = self.complex_gate.input_gates[clk_input_index].clone();

                let clk_input_index = self.flip_flops[i].lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::CLK_IN);
                connect_gates(
                    clk_input_gate.clone(),
                    0,
                    self.flip_flops[i].clone(),
                    clk_input_index,
                );
            } else {
                let clk_input_index = self.flip_flops[i].lock().unwrap().get_index_from_tag(MasterSlaveJKFlipFlop::CLK_IN);
                let splitter_output_index = self.flip_flop_output_splitter.lock().unwrap().get_index_for_output(i - 1, 1);
                connect_gates(
                    self.flip_flop_output_splitter.clone(),
                    splitter_output_index,
                    self.flip_flops[i].clone(),
                    clk_input_index,
                );
            }

            let splitter_output_index = self.flip_flop_output_splitter.lock().unwrap().get_index_for_output(i, 0);
            let output_tag = format!("o_{}", i);
            let output_index = self.get_index_from_tag(output_tag.as_str());
            connect_gates(
                self.flip_flop_output_splitter.clone(),
                splitter_output_index,
                output_gates[output_index].clone(),
                0,
            );
        }

        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
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
                    panic!("A gate input for type {} id {} tag {} was empty", simple_gate.gate_type, simple_gate.unique_id.id(), simple_gate.tag);
                } else if i.len() == 1 {
                    let (id, _signal) = i.iter().next().unwrap();
                    if id.id() == 0 {
                        panic!("A gate input for type {} id {} tag {} was empty\n{:#?}", simple_gate.gate_type, simple_gate.unique_id.id(), simple_gate.tag, simple_gate.input_signals);
                    }
                } else {
                    panic!("Multiple inputs connected for the same gate")
                }
            }
        }

        for i in 0..self.flip_flops.len() {
            check_output(&self.flip_flops[i].lock().unwrap().complex_gate.simple_gate);
            check_output(&self.flip_flop_clk_in_not_gates[i].lock().unwrap().members);
        }
    }
}

impl LogicGate for VariableBitCounter {
    fn internal_connect_output(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: SharedMutex<dyn LogicGate>) -> Signal {
        self.complex_gate.connect_output(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        )
    }

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.complex_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals(
            &self.get_tag(),
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.complex_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    fn num_children_gates(&self) -> usize {
        self.complex_gate.simple_gate.number_child_gates
    }

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>> {
        self.complex_gate.input_gates.clone()
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

        let output = cpu_enable.lock().unwrap().fetch_output_signals().unwrap();

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

        let clock_output = flip_flop.lock().unwrap().fetch_output_signals().unwrap();

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
                vec![HIGH, LOW_], //Q ~Q
                vec![LOW_, HIGH], //Q ~Q
                vec![LOW_, HIGH], //Q ~Q
                vec![HIGH, LOW_], //Q ~Q
                vec![HIGH, LOW_], //Q ~Q
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

        let clock_output = clock.lock().unwrap().fetch_output_signals().unwrap();

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
        clock_hookup.lock().unwrap().toggle_output_printing(true);
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

    #[test]
    fn multiplexer_init() {
        let multiplexer = VariableBitMultiplexer::new(3, 2);

        let multiplexer_output = multiplexer.lock().unwrap().fetch_output_signals().unwrap();

        let mut output_signals = Vec::new();
        for out in multiplexer_output {
            match out {
                GateOutputState::NotConnected(signal) => {
                    output_signals.push(signal);
                }
                GateOutputState::Connected(_) => panic!("Clock should not be connected to anything")
            }
        }

        assert_eq!(
            output_signals,
            vec![LOW_, LOW_, LOW_],
        );
    }

    #[test]
    fn multiplexer_run() {
        run_multi_input_output_logic_gate(
            vec![],
            vec![
                vec![HIGH, HIGH, HIGH],
                vec![LOW_, HIGH, LOW_],
                vec![HIGH, HIGH, HIGH],
                vec![LOW_, HIGH, LOW_],
                vec![LOW_, HIGH, LOW_],
            ],
            HashMap::from(
                [
                    ("C_0", vec![
                        vec![LOW_],
                        vec![HIGH],
                        vec![LOW_],
                        vec![HIGH],
                        vec![HIGH],
                    ]),
                    ("I_0_bit", vec![
                        vec![HIGH, HIGH, HIGH],
                        vec![HIGH, HIGH, HIGH],
                        vec![HIGH, HIGH, HIGH],
                        vec![HIGH, HIGH, HIGH],
                        vec![HIGH, HIGH, HIGH],
                    ]
                    ),
                    ("I_1_bit", vec![
                        vec![LOW_, HIGH, LOW_],
                        vec![LOW_, HIGH, LOW_],
                        vec![LOW_, HIGH, LOW_],
                        vec![LOW_, HIGH, LOW_],
                        vec![LOW_, HIGH, LOW_],
                    ]
                    ),
                ]
            ),
            VariableBitMultiplexer::new(3, 2),
        );
    }

    #[test]
    #[should_panic]
    fn multiplexer_invalid_num_input() {
        let multi = VariableBitMultiplexer::new(2, 3);
        multi.lock().unwrap().toggle_output_printing(true);
    }

    #[test]
    fn variable_bit_counter_initialization() {
        let counter = VariableBitCounter::new(4);

        let counter_output = counter.lock().unwrap().fetch_output_signals().unwrap();

        let mut output_signals = Vec::new();
        for out in counter_output {
            match out {
                GateOutputState::NotConnected(signal) => {
                    output_signals.push(signal);
                }
                GateOutputState::Connected(_) => panic!("Clock should not be connected to anything")
            }
        }

        assert_eq!(
            output_signals,
            vec![LOW_, LOW_, LOW_, LOW_],
        );
    }

    #[test]
    fn variable_bit_counter_run() {
        let num_output_pins = rand::thread_rng().gen_range(1..8);

        let possible_numbers = usize::pow(2, num_output_pins as u32);

        let mut output_signals: Vec<Vec<Signal>> = Vec::new();
        let mut clk_input_signals: Vec<Vec<Signal>> = Vec::new();

        //Starts with 0.
        output_signals.push(vec![LOW_; num_output_pins]);
        clk_input_signals.push(vec![HIGH]);

        for i in 1..possible_numbers {
            let binary_number = format!("{:0width$b}", i, width = num_output_pins);

            let mut output = Vec::new();
            for c in binary_number.chars().rev() {
                output.push(
                    if c == '0' {
                        LOW_
                    } else {
                        HIGH
                    }
                );
            }

            output_signals.push(output.clone());
            output_signals.push(output);

            clk_input_signals.push(vec![LOW_]);
            clk_input_signals.push(vec![HIGH]);
        }

        //Ends with 0.
        output_signals.push(vec![LOW_; num_output_pins]);
        clk_input_signals.push(vec![LOW_]);

        let counter = VariableBitCounter::new(num_output_pins);
        run_multi_input_output_logic_gate(
            vec![],
            output_signals,
            HashMap::from(
                [
                    (VariableBitCounter::CLK_IN, clk_input_signals),
                ]
            ),
            counter,
        );
    }
}
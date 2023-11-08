use crate::logic::basic_gates::{Nand, Nor};
use crate::logic::foundations::{ComplexGateMembers, connect_gates, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, push_reg_outputs_to_output_gates, Signal, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW_, HIGH};
use crate::shared_mutex::{new_shared_mutex, SharedMutex};

pub struct SRLatch {
    complex_gate: ComplexGateMembers,
    top_nor_gate: SharedMutex<Nor>,
    bottom_nor_gate: SharedMutex<Nor>,
}

#[allow(dead_code)]
impl SRLatch {
    pub fn new() -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_clone: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        let reset_input_gate = SimpleInput::new(1, "R");
        let set_input_gate = SimpleInput::new(1, "S");
        let q_output_gate = SimpleOutput::new("Q");
        let nq_output_gate = SimpleOutput::new("~Q");

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

        sr_latch.top_nor_gate.lock().unwrap().set_tag("TOP_NOR_GATE");
        sr_latch.bottom_nor_gate.lock().unwrap().set_tag("BOTTOM_NOR_GATE");

        //Force the ~Q to be low and Q to be high.
        reset_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        sr_latch.build_and_prime_circuit(output_gates_clone);

        reset_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                LOW_,
                UniqueID::zero_id(),
            )
        );

        new_shared_mutex(sr_latch)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("R")].clone();
        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();

        let q_output_gate = output_gates[self.get_index_from_tag("Q")].clone();
        let not_q_output_gate = output_gates[self.get_index_from_tag("~Q")].clone();

        connect_gates(
            r_input_gate.clone(),
            0,
            self.top_nor_gate.clone(),
            0,
        );

        connect_gates(
            s_input_gate.clone(),
            0,
            self.bottom_nor_gate.clone(),
            1,
        );

        connect_gates(
            self.top_nor_gate.clone(),
            0,
            q_output_gate.clone(),
            0,
        );

        connect_gates(
            self.top_nor_gate.clone(),
            1,
            self.bottom_nor_gate.clone(),
            0,
        );

        connect_gates(
            self.bottom_nor_gate.clone(),
            0,
            not_q_output_gate.clone(),
            0,
        );

        connect_gates(
            self.bottom_nor_gate.clone(),
            1,
            self.top_nor_gate.clone(),
            1,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for SRLatch {
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
        //SRLatch has an `invalid` state of HIGH HIGH. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_calculate(
            &self.get_tag(),
        )
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_no_calculate(
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
        self.complex_gate.simple_gate.tag = tag.to_string()
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

pub struct ActiveLowSRLatch {
    complex_gate: ComplexGateMembers,
    top_nand_gate: SharedMutex<Nand>,
    bottom_nand_gate: SharedMutex<Nand>,
}

#[allow(dead_code)]
impl ActiveLowSRLatch {
    pub fn new() -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_clone: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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
            GateInput::new(
                sr_latch.get_index_from_tag("R"),
                HIGH,
                UniqueID::zero_id(),
            )
        );

        sr_latch.update_input_signal(
            GateInput::new(
                sr_latch.get_index_from_tag("S"),
                HIGH,
                UniqueID::zero_id(),
            )
        );

        sr_latch.build_and_prime_circuit(output_gates_clone);

        new_shared_mutex(sr_latch)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("R")].clone();
        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();

        let q_output_gate = output_gates[self.get_index_from_tag("Q")].clone();
        let not_q_output_gate = output_gates[self.get_index_from_tag("~Q")].clone();

        connect_gates(
            s_input_gate.clone(),
            0,
            self.top_nand_gate.clone(),
            0,
        );

        connect_gates(
            r_input_gate.clone(),
            0,
            self.bottom_nand_gate.clone(),
            1,
        );

        connect_gates(
            self.top_nand_gate.clone(),
            0,
            q_output_gate.clone(),
            0,
        );

        connect_gates(
            self.top_nand_gate.clone(),
            1,
            self.bottom_nand_gate.clone(),
            0,
        );

        connect_gates(
            self.bottom_nand_gate.clone(),
            0,
            not_q_output_gate.clone(),
            0,
        );

        connect_gates(
            self.bottom_nand_gate.clone(),
            1,
            self.top_nand_gate.clone(),
            1,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for ActiveLowSRLatch {
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

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_calculate(
            &self.get_tag(),
        )
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_no_calculate(
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

pub struct OneBitMemoryCell {
    complex_gate: ComplexGateMembers,
    set_enable_nand_gate: SharedMutex<Nand>,
    enable_nand_gate: SharedMutex<Nand>,
    sr_top_nand_gate: SharedMutex<Nand>,
    sr_bottom_nand_gate: SharedMutex<Nand>,
}

#[allow(dead_code)]
impl OneBitMemoryCell {
    pub fn new(output_num: usize) -> SharedMutex<Self> {
        assert_ne!(output_num, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        let enable_input_gate = SimpleInput::new(2, "E");
        let set_input_gate = SimpleInput::new(1, "S");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(enable_input_gate.clone());
        input_gates.push(set_input_gate.clone());

        let q_output_gate = SimpleOutput::new("Q");
        output_gates.push(q_output_gate.clone());
        output_gates_logic.push(q_output_gate);

        for i in 1..output_num {
            let q_tag = format!("Q_{}", i);
            let q_output_gate = SimpleOutput::new(q_tag.as_str());
            output_gates.push(q_output_gate.clone());
            output_gates_logic.push(q_output_gate);
        }

        //todo: delete
        println!("OneBitMemoryCellGate ids");
        println!("enable_input_gate {}", input_gates[0].lock().unwrap().get_unique_id().id());
        println!("set_input_gate {}", input_gates[1].lock().unwrap().get_unique_id().id());
        println!("output_gate {}", output_gates[0].lock().unwrap().get_unique_id().id());

        let mut one_bit_memory_cell = OneBitMemoryCell {
            complex_gate: ComplexGateMembers::new(
                2,
                output_num,
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
                2, 1 + output_num,
            ),
            sr_bottom_nand_gate: Nand::new(
                2, 1,
            ),
        };

        //todo: delete

        println!("set_enable_nand_gate {}", one_bit_memory_cell.set_enable_nand_gate.lock().unwrap().get_unique_id().id());
        println!("enable_nand_gate {}", one_bit_memory_cell.enable_nand_gate.lock().unwrap().get_unique_id().id());
        println!("sr_top_nand_gate {}", one_bit_memory_cell.sr_top_nand_gate.lock().unwrap().get_unique_id().id());
        println!("sr_bottom_nand_gate {}", one_bit_memory_cell.sr_bottom_nand_gate.lock().unwrap().get_unique_id().id());

        //This will allow the circuit to be primed to the LOW output state.
        enable_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        one_bit_memory_cell.build_and_prime_circuit(
            output_num,
            output_gates_logic,
        );

        enable_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                LOW_,
                UniqueID::zero_id(),
            )
        );

        new_shared_mutex(one_bit_memory_cell)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_num: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();

        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();

        connect_gates(
            s_input_gate.clone(),
            0,
            self.set_enable_nand_gate.clone(),
            0,
        );

        connect_gates(
            e_input_gate.clone(),
            0,
            self.set_enable_nand_gate.clone(),
            1,
        );

        connect_gates(
            e_input_gate.clone(),
            1,
            self.enable_nand_gate.clone(),
            1,
        );

        connect_gates(
            self.set_enable_nand_gate.clone(),
            0,
            self.sr_top_nand_gate.clone(),
            0,
        );

        connect_gates(
            self.set_enable_nand_gate.clone(),
            1,
            self.enable_nand_gate.clone(),
            0,
        );

        connect_gates(
            self.enable_nand_gate.clone(),
            0,
            self.sr_bottom_nand_gate.clone(),
            1,
        );

        connect_gates(
            self.sr_top_nand_gate.clone(),
            0,
            output_gates[0].clone(),
            0,
        );

        connect_gates(
            self.sr_top_nand_gate.clone(),
            1,
            self.sr_bottom_nand_gate.clone(),
            0,
        );

        for i in 1..output_num {
            connect_gates(
                self.sr_top_nand_gate.clone(),
                i + 1,
                output_gates[i].clone(),
                0,
            );
        }

        connect_gates(
            self.sr_bottom_nand_gate.clone(),
            0,
            self.sr_top_nand_gate.clone(),
            1,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for OneBitMemoryCell {
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

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_calculate(
            &self.get_tag(),
        )
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_no_calculate(
            &self.get_tag(),
        )
    }

    fn get_gate_type(&self) -> GateType {
        self.complex_gate.simple_gate.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        println!("OneBitMemoryCell get_unique_id"); //todo delete
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

pub struct VariableBitMemoryCell {
    complex_gate: ComplexGateMembers,
    one_bit_memory_cells: Vec<SharedMutex<OneBitMemoryCell>>,
}

#[allow(dead_code)]
impl VariableBitMemoryCell {
    pub fn new(number_bits: usize) -> SharedMutex<Self> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut one_bit_memory_cells: Vec<SharedMutex<OneBitMemoryCell>> = Vec::new();

        for i in 0..number_bits {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            one_bit_memory_cells.push(
                OneBitMemoryCell::new(2)
            );
        }

        push_reg_outputs_to_output_gates(
            number_bits,
            &mut output_gates,
            &mut output_gates_logic,
        );

        let set_input_gate = SimpleInput::new(number_bits, "S");

        //Order of input gates is important here to force the circuit into a deterministic state.
        input_gates.push(set_input_gate.clone());

        let mut one_bit_memory_cell = VariableBitMemoryCell {
            complex_gate: ComplexGateMembers::new(
                number_bits + 1,
                2 * number_bits,
                GateType::VariableBitMemoryCellType,
                input_gates,
                output_gates,
            ),
            one_bit_memory_cells,
        };

        one_bit_memory_cell.build_and_prime_circuit(number_bits, output_gates_logic);

        new_shared_mutex(one_bit_memory_cell)
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();

        for i in 0..number_bits {
            let enable_gate_index = self.one_bit_memory_cells[i].lock().unwrap().get_index_from_tag("E");
            let set_gate_index = self.one_bit_memory_cells[i].lock().unwrap().get_index_from_tag("S");

            connect_gates(
                self.complex_gate.input_gates[i].clone(),
                0,
                self.one_bit_memory_cells[i].clone(),
                set_gate_index,
            );

            connect_gates(
                s_input_gate.clone(),
                i,
                self.one_bit_memory_cells[i].clone(),
                enable_gate_index,
            );

            connect_gates(
                self.one_bit_memory_cells[i].clone(),
                0,
                output_gates[i].clone(),
                0,
            );

            let reg_tag = format!("reg_{}", i);
            let reg_idx = self.get_index_from_tag(reg_tag.as_str());
            connect_gates(
                self.one_bit_memory_cells[i].clone(),
                1,
                output_gates[reg_idx].clone(),
                0,
            );
        }

        drop(s_input_gate);

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for VariableBitMemoryCell {
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

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_calculate(
            &self.get_tag(),
        )
    }

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.complex_gate.fetch_output_signals_no_calculate(
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
    use crate::globals::CLOCK_TICK_NUMBER;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW_};
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
        sr_latch: SharedMutex<dyn LogicGate>,
    ) {
        let r_input_gate = AutomaticInput::new(r_input_signal, 1, "Start_R");
        let s_input_gate = AutomaticInput::new(s_input_signal, 1, "Start_S");

        let q_output_gate = SimpleOutput::new("End_Q");
        let not_q_output_gate = SimpleOutput::new("End_~Q");

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(r_input_gate.clone());
        input_gates.push(s_input_gate.clone());
        output_gates.push(q_output_gate.clone());
        output_gates.push(not_q_output_gate.clone());

        let r_index = sr_latch.lock().unwrap().get_index_from_tag("R");
        connect_gates(
            r_input_gate.clone(),
            0,
            sr_latch.clone(),
            r_index,
        );

        let s_index = sr_latch.lock().unwrap().get_index_from_tag("S");
        connect_gates(
            s_input_gate.clone(),
            0,
            sr_latch.clone(),
            s_index,
        );

        let q_output_idx = sr_latch.lock().unwrap().get_index_from_tag("Q");
        connect_gates(
            sr_latch.clone(),
            q_output_idx,
            q_output_gate.clone(),
            0,
        );

        let not_q_output_idx = sr_latch.lock().unwrap().get_index_from_tag("~Q");
        connect_gates(
            sr_latch.clone(),
            not_q_output_idx,
            not_q_output_gate.clone(),
            0,
        );

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
                &mut |_clock_tick_inputs, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
                    assert_eq!(output_gates.len(), 2);

                    let mut q_output = output_gates[0].lock().unwrap();
                    let mut not_q_output = output_gates[1].lock().unwrap();

                    let q_output = q_output.fetch_output_signals_calculate().unwrap();
                    let not_q_output = not_q_output.fetch_output_signals_calculate().unwrap();

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
        let second_q_output_gate = SimpleOutput::new("End_Q_2");

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(e_input_gate.clone());
        input_gates.push(s_input_gate.clone());
        output_gates.push(q_output_gate.clone());
        output_gates.push(second_q_output_gate.clone());

        let one_bit_memory_cell = OneBitMemoryCell::new(2);

        let e_index = one_bit_memory_cell.lock().unwrap().get_index_from_tag("E");
        connect_gates(
            e_input_gate.clone(),
            0,
            one_bit_memory_cell.clone(),
            e_index,
        );

        let s_index = one_bit_memory_cell.lock().unwrap().get_index_from_tag("S");
        connect_gates(
            s_input_gate.clone(),
            0,
            one_bit_memory_cell.clone(),
            s_index,
        );

        let q_output_idx = one_bit_memory_cell.lock().unwrap().get_index_from_tag("Q");
        connect_gates(
            one_bit_memory_cell.clone(),
            q_output_idx,
            q_output_gate.clone(),
            0,
        );

        let second_q_output_idx = one_bit_memory_cell.lock().unwrap().get_index_from_tag("Q_1");
        connect_gates(
            one_bit_memory_cell.clone(),
            second_q_output_idx,
            second_q_output_gate.clone(),
            0,
        );

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
                &mut |_clock_tick_inputs, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
                    assert_eq!(output_gates.len(), 2);

                    for out in output_gates.iter() {
                        let mut q_output = out.lock().unwrap();

                        let q_output = q_output.fetch_output_signals_calculate().unwrap();

                        assert_eq!(q_output.len(), 1);

                        let q_output_state = &q_output[0];

                        match q_output_state {
                            GateOutputState::NotConnected(signal) => {
                                collected_output.push(signal.clone());
                            }
                            GateOutputState::Connected(_) => panic!("Final output gate should not be connected"),
                        }
                    }
                },
            );

            propagate_signal_through_circuit = false;
        }

        let mut output_signals = Vec::new();
        for s in q_output_signal.into_iter() {
            output_signals.push(s.clone());
            output_signals.push(s);
        }
        assert_eq!(collected_output, output_signals);
    }

    #[test]
    fn sr_gate_initialization() {
        let sr_latch = SRLatch::new();

        let output = sr_latch.lock().unwrap().fetch_output_signals_calculate().unwrap();

        assert_eq!(output.len(), 2);

        let first_output = output.get(0).unwrap();
        let second_output = output.get(1).unwrap();

        match first_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, LOW_),
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
            vec![LOW_, LOW_],
            vec![HIGH, LOW_],
            vec![HIGH, HIGH],
            vec![LOW_, LOW_],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_low_low_after_high_low() {
        run_sr_latch(
            vec![HIGH, LOW_],
            vec![LOW_, LOW_],
            vec![LOW_, LOW_],
            vec![HIGH, HIGH],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_low_high() {
        run_sr_latch(
            vec![LOW_],
            vec![HIGH],
            vec![HIGH],
            vec![LOW_],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_high_low() {
        run_sr_latch(
            vec![HIGH],
            vec![LOW_],
            vec![LOW_],
            vec![HIGH],
            SRLatch::new(),
        );
    }

    #[test]
    fn sr_gate_high_high() {
        run_sr_latch(
            vec![HIGH],
            vec![HIGH],
            vec![LOW_],
            vec![LOW_],
            SRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_initialization() {
        let sr_latch = ActiveLowSRLatch::new();

        let output = sr_latch.lock().unwrap().fetch_output_signals_calculate().unwrap();

        assert_eq!(output.len(), 2);

        let first_output = output.get(0).unwrap();
        let second_output = output.get(1).unwrap();

        match first_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, HIGH),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }

        match second_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, LOW_),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }
    }

    #[test]
    fn active_low_sr_gate_high_high_after_low_high() {
        run_sr_latch(
            vec![LOW_, HIGH],
            vec![HIGH, HIGH],
            vec![LOW_, LOW_],
            vec![HIGH, HIGH],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_high_high_after_high_low() {
        run_sr_latch(
            vec![HIGH, HIGH],
            vec![LOW_, HIGH],
            vec![HIGH, HIGH],
            vec![LOW_, LOW_],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_low_high() {
        run_sr_latch(
            vec![LOW_],
            vec![HIGH],
            vec![LOW_],
            vec![HIGH],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_high_low() {
        run_sr_latch(
            vec![HIGH],
            vec![LOW_],
            vec![HIGH],
            vec![LOW_],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn active_low_sr_gate_low_low() {
        run_sr_latch(
            vec![LOW_],
            vec![LOW_],
            vec![HIGH],
            vec![HIGH],
            ActiveLowSRLatch::new(),
        );
    }

    #[test]
    fn one_bit_mem_initialization() {
        //initialization
        let one_bit_memory_cell = OneBitMemoryCell::new(1);

        let output = one_bit_memory_cell.lock().unwrap().fetch_output_signals_calculate().unwrap();

        assert_eq!(output.len(), 1);

        let first_output = output.first().unwrap();

        match first_output {
            GateOutputState::NotConnected(signal) => assert_eq!(*signal, LOW_),
            GateOutputState::Connected(_) => panic!("Output should never be connected"),
        }
    }

    #[test]
    fn one_bit_mem_low_high() {
        run_one_bit_memory_cell(
            vec![LOW_],
            vec![HIGH],
            vec![LOW_],
        );
    }

    #[test]
    fn one_bit_mem_high_low() {
        run_one_bit_memory_cell(
            vec![HIGH],
            vec![LOW_],
            vec![LOW_],
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
            vec![HIGH, LOW_],
            vec![HIGH, LOW_],
            vec![HIGH, HIGH],
        );
    }

    #[test]
    fn one_bit_mem_low_high_to_low_low() {
        run_one_bit_memory_cell(
            vec![LOW_, LOW_],
            vec![HIGH, LOW_],
            vec![LOW_, LOW_],
        );
    }

    #[test]
    fn one_bit_mem_high_low_to_low_low() {
        run_one_bit_memory_cell(
            vec![HIGH, LOW_],
            vec![LOW_, LOW_],
            vec![LOW_, LOW_],
        );
    }

    #[test]
    fn one_bit_mem_low_low_to_low_low() {
        run_one_bit_memory_cell(
            vec![LOW_, LOW_],
            vec![LOW_, LOW_],
            vec![LOW_, LOW_],
        );
    }

    #[test]
    fn one_bit_mem_high_high_to_high_low() {
        run_one_bit_memory_cell(
            vec![HIGH, HIGH],
            vec![HIGH, LOW_],
            vec![HIGH, LOW_],
        );
    }

    #[test]
    fn one_bit_mem_high_low_to_high_high() {
        run_one_bit_memory_cell(
            vec![HIGH, HIGH],
            vec![LOW_, HIGH],
            vec![LOW_, HIGH],
        );
    }

    #[test]
    fn variable_bit_mem_initialization() {
        let num_bits = rand::thread_rng().gen_range(2..=16);
        let variable_bit_memory_cell = VariableBitMemoryCell::new(num_bits);

        let output = variable_bit_memory_cell.lock().unwrap().fetch_output_signals_calculate().unwrap();

        assert_eq!(output.len(), 2 * num_bits);
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
    fn variable_bit_signal_high() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW_, HIGH],
            ],
            vec![
                vec![HIGH, LOW_, HIGH, HIGH, LOW_, HIGH],
            ],
            HashMap::from(
                [("S", vec![vec![HIGH]])]
            ),
            VariableBitMemoryCell::new(3),
        );
    }

    #[test]
    fn variable_bit_signal_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW_],
            ],
            vec![
                vec![LOW_, LOW_, LOW_, LOW_],
            ],
            HashMap::from(
                [("S", vec![vec![LOW_]])]
            ),
            VariableBitMemoryCell::new(2),
        );
    }

    #[test]
    fn variable_bit_saved_states() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH, LOW_],
                vec![HIGH, HIGH, LOW_],
                vec![HIGH, HIGH, LOW_],
                vec![LOW_, LOW_, HIGH],
            ],
            vec![
                vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_],
                vec![HIGH, HIGH, LOW_, HIGH, HIGH, LOW_],
                vec![HIGH, HIGH, LOW_, HIGH, HIGH, LOW_],
                vec![LOW_, LOW_, HIGH, LOW_, LOW_, HIGH],
            ],
            HashMap::from(
                [("S", vec![vec![LOW_], vec![HIGH], vec![LOW_], vec![HIGH]])]
            ),
            VariableBitMemoryCell::new(3),
        );
    }
}

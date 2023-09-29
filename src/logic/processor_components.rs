use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::{And, ControlledBuffer, Not, Or, Splitter};
use crate::logic::complex_logic::VariableBitCPUEnable;
use crate::logic::foundations::{build_simple_inputs_and_outputs, build_simple_inputs_and_outputs_with_and, ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, push_reg_outputs_to_output_gates, Signal, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW_, HIGH};
use crate::logic::memory_gates::VariableBitMemoryCell;

pub struct VariableBitRegister {
    complex_gate: ComplexGateMembers,
    memory: Rc<RefCell<VariableBitMemoryCell>>,
    enable: Rc<RefCell<VariableBitCPUEnable>>,
    controlled_buffer: Rc<RefCell<ControlledBuffer>>,
}

#[allow(dead_code)]
impl VariableBitRegister {
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

        push_reg_outputs_to_output_gates(
            number_bits,
            &mut output_gates,
            &mut output_gates_logic,
        );

        let set_input_gate = SimpleInput::new(1, "S");
        let enable_input_gate = SimpleInput::new(2, "E");

        input_gates.push(set_input_gate.clone());
        input_gates.push(enable_input_gate.clone());

        let mut bit_register = VariableBitRegister {
            complex_gate: ComplexGateMembers::new(
                number_bits + 2,
                2 * number_bits,
                GateType::VariableBitRegisterType,
                input_gates,
                output_gates,
            ),
            memory: VariableBitMemoryCell::new(number_bits),
            enable: VariableBitCPUEnable::new(number_bits),
            controlled_buffer: ControlledBuffer::new(number_bits),
        };

        bit_register.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(bit_register))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..number_bits {
            let mut input_gate = self.complex_gate.input_gates[i].borrow_mut();

            input_gate.connect_output_to_next_gate(
                0,
                i,
                self.memory.clone(),
            );

            self.memory.borrow_mut().connect_output_to_next_gate(
                i,
                i,
                self.enable.clone(),
            );

            let reg_tag = format!("reg_{}", i);
            let mem_reg_index = self.memory.borrow_mut().get_index_from_tag(reg_tag.as_str());
            let self_reg_index = self.get_index_from_tag(reg_tag.as_str());
            self.memory.borrow_mut().connect_output_to_next_gate(
                mem_reg_index,
                0,
                output_gates[self_reg_index].clone(),
            );

            self.enable.borrow_mut().connect_output_to_next_gate(
                i,
                i,
                self.controlled_buffer.clone(),
            );

            self.controlled_buffer.borrow_mut().connect_output_to_next_gate(
                i,
                0,
                output_gates[i].clone(),
            );
        }

        let s_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("S")].clone();
        let memory_set_index = self.memory.borrow_mut().get_index_from_tag("S");
        s_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            memory_set_index,
            self.memory.clone(),
        );

        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();
        let memory_enable_index = self.enable.borrow_mut().get_index_from_tag("E");
        e_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            memory_enable_index,
            self.enable.clone(),
        );

        let controlled_buffer_enable_index = self.controlled_buffer.borrow_mut().get_index_from_tag("E");
        e_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            controlled_buffer_enable_index,
            self.controlled_buffer.clone(),
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for VariableBitRegister {
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

pub struct VariableDecoder {
    pub complex_gate: ComplexGateMembers,
    and_gates: Vec<Rc<RefCell<And>>>,
    not_gates: Vec<Rc<RefCell<Not>>>,
}

#[allow(dead_code)]
impl VariableDecoder {
    pub fn new(number_inputs: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_inputs, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let number_outputs = usize::pow(2, number_inputs as u32);

        for i in 0..number_inputs {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(number_outputs / 2 + 1, input_tag.as_str()));
        }

        for i in 0..number_outputs {
            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);
        }

        let mut and_gates = Vec::with_capacity(number_outputs);
        for _ in 0..number_outputs {
            and_gates.push(And::new(number_inputs, 1));
        }

        let mut not_gates = Vec::with_capacity(number_inputs);
        for _ in 0..number_inputs {
            not_gates.push(Not::new(number_outputs / 2));
        }

        let mut decoder = VariableDecoder {
            complex_gate: ComplexGateMembers::new(
                number_inputs,
                number_outputs,
                GateType::VariableDecoderType,
                input_gates,
                output_gates,
            ),
            and_gates,
            not_gates,
        };

        decoder.build_and_prime_circuit(number_inputs, number_outputs, output_gates_logic);

        Rc::new(RefCell::new(decoder))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_inputs: usize,
        number_outputs: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..number_inputs {
            self.complex_gate.input_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                self.not_gates[i].clone(),
            );
        }

        let mut input_gate_index = vec![1; number_inputs];
        let mut not_gate_index = vec![0; number_inputs];

        for i in 0..number_outputs {
            //This will make a binary number formatted as a String with `number_inputs` digits.
            let binary_number = format!("{:0width$b}", i, width = number_inputs);

            for (j, c) in binary_number.chars().enumerate() {
                if c == '0' { // '0' means connects from output.
                    let next_index = not_gate_index[j];
                    not_gate_index[j] += 1;
                    self.not_gates[j].borrow_mut().connect_output_to_next_gate(
                        next_index,
                        j,
                        self.and_gates[i].clone(),
                    );
                } else { // '1' means connects from input.
                    let next_index = input_gate_index[j];
                    input_gate_index[j] += 1;
                    self.complex_gate.input_gates[j].borrow_mut().connect_output_to_next_gate(
                        next_index,
                        j,
                        self.and_gates[i].clone(),
                    );
                }
            }

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

impl LogicGate for VariableDecoder {
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

pub struct SingleRAMCell {
    complex_gate: ComplexGateMembers,
    register: Rc<RefCell<VariableBitRegister>>,
    h_v_and_gate: Rc<RefCell<And>>,
    set_and_gate: Rc<RefCell<And>>,
    enable_and_gate: Rc<RefCell<And>>,
    reset_or_gate: Rc<RefCell<Or>>,
    controlled_buffer: Rc<RefCell<ControlledBuffer>>,
}

#[allow(dead_code)]
impl SingleRAMCell {
    pub fn new(number_inputs_outputs: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_inputs_outputs, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        build_simple_inputs_and_outputs(
            number_inputs_outputs,
            &mut input_gates,
            &mut output_gates,
            &mut output_gates_logic,
        );

        push_reg_outputs_to_output_gates(
            number_inputs_outputs,
            &mut output_gates,
            &mut output_gates_logic,
        );

        let horizontal_input_gate = SimpleInput::new(1, "H");
        let vertical_input_gate = SimpleInput::new(1, "V");
        let set_input_gate = SimpleInput::new(1, "S");
        let enable_input_gate = SimpleInput::new(1, "E");
        let reset_input_gate = SimpleInput::new(1, "R");

        input_gates.push(horizontal_input_gate);
        input_gates.push(vertical_input_gate);
        input_gates.push(set_input_gate);
        input_gates.push(enable_input_gate);
        input_gates.push(reset_input_gate);

        let mut ram_cell = SingleRAMCell {
            complex_gate: ComplexGateMembers::new(
                number_inputs_outputs + 5,
                2 * number_inputs_outputs,
                GateType::VariableSingleRAMCellType,
                input_gates,
                output_gates,
            ),
            register: VariableBitRegister::new(number_inputs_outputs),
            h_v_and_gate: And::new(2, 2),
            set_and_gate: And::new(2, 1),
            enable_and_gate: And::new(2, 2),
            reset_or_gate: Or::new(2, 1),
            controlled_buffer: ControlledBuffer::new(number_inputs_outputs),
        };

        ram_cell.build_and_prime_circuit(
            number_inputs_outputs,
            output_gates_logic,
        );

        Rc::new(RefCell::new(ram_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_input_outputs: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let horizontal_input_index = self.get_index_from_tag("H");
        let vertical_input_index = self.get_index_from_tag("V");
        let set_input_index = self.get_index_from_tag("S");
        let enable_input_index = self.get_index_from_tag("E");
        let reset_input_index = self.get_index_from_tag("R");

        let horizontal_input_gate = self.complex_gate.input_gates[horizontal_input_index].clone();
        let vertical_input_gate = self.complex_gate.input_gates[vertical_input_index].clone();
        let set_input_gate = self.complex_gate.input_gates[set_input_index].clone();
        let enable_input_gate = self.complex_gate.input_gates[enable_input_index].clone();
        let reset_input_gate = self.complex_gate.input_gates[reset_input_index].clone();

        horizontal_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.h_v_and_gate.clone(),
        );

        vertical_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.h_v_and_gate.clone(),
        );

        set_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.set_and_gate.clone(),
        );

        enable_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.enable_and_gate.clone(),
        );

        reset_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.reset_or_gate.clone(),
        );

        self.h_v_and_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.set_and_gate.clone(),
        );

        self.h_v_and_gate.borrow_mut().connect_output_to_next_gate(
            1,
            0,
            self.enable_and_gate.clone(),
        );

        self.set_and_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.reset_or_gate.clone(),
        );

        let mem_set_index = self.register.borrow_mut().get_index_from_tag("S");
        self.reset_or_gate.borrow_mut().connect_output_to_next_gate(
            0,
            mem_set_index,
            self.register.clone(),
        );

        let mem_enable_index = self.register.borrow_mut().get_index_from_tag("E");
        self.enable_and_gate.borrow_mut().connect_output_to_next_gate(
            0,
            mem_enable_index,
            self.register.clone(),
        );

        let buffer_enable_index = self.controlled_buffer.borrow_mut().get_index_from_tag("E");
        self.enable_and_gate.borrow_mut().connect_output_to_next_gate(
            1,
            buffer_enable_index,
            self.controlled_buffer.clone(),
        );

        for i in 0..number_input_outputs {
            let register_tag = format!("reg_{}", i);
            let self_reg_index = self.get_index_from_tag(register_tag.as_str());
            let register_reg_index = self.register.borrow_mut().get_index_from_tag(register_tag.as_str());

            self.register.borrow_mut().connect_output_to_next_gate(
                register_reg_index,
                0,
                output_gates[self_reg_index].clone(),
            );

            self.complex_gate.input_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                i,
                self.register.clone(),
            );

            self.register.borrow_mut().connect_output_to_next_gate(
                i,
                i,
                self.controlled_buffer.clone(),
            );

            self.controlled_buffer.borrow_mut().connect_output_to_next_gate(
                i,
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

impl LogicGate for SingleRAMCell {
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

pub struct RAMUnit {
    complex_gate: ComplexGateMembers,
    memory_address_register: Rc<RefCell<VariableBitMemoryCell>>,
    horizontal_decoder: Rc<RefCell<VariableDecoder>>,
    horizontal_decoder_splitter: Rc<RefCell<Splitter>>,
    vertical_decoder: Rc<RefCell<VariableDecoder>>,
    vertical_decoder_splitter: Rc<RefCell<Splitter>>,
    controlled_buffer: Rc<RefCell<ControlledBuffer>>,
    ram_cells: Vec<Rc<RefCell<SingleRAMCell>>>,
}

#[allow(dead_code)]
impl RAMUnit {

    pub fn get_ram_output_string(ram_cell_index: usize, bit_index: usize) -> String {
        format!("cell_{}_bit_{}", ram_cell_index, bit_index)
    }

    pub fn new(bus_size_in_bits: usize, decoder_input_size: usize) -> Rc<RefCell<Self>> {
        assert_ne!(bus_size_in_bits, 0);
        assert_ne!(decoder_input_size, 0);

        let num_ram_cells_in_row = usize::pow(2, decoder_input_size as u32);
        let num_ram_cells = usize::pow(num_ram_cells_in_row, 2);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        for i in 0..bus_size_in_bits {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(num_ram_cells, input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);
        }

        for i in 0..(decoder_input_size * 2) {
            let input_tag = format!("addr_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));
            //todo d
            // input_gates.last().unwrap().borrow_mut().toggle_output_printing(true);
        }

        let set_address_input_gate = SimpleInput::new(1, "SA");
        let reset_input_gate = SimpleInput::new(num_ram_cells, "R");
        let set_input_gate = SimpleInput::new(num_ram_cells, "S");
        let enable_input_gate = SimpleInput::new(num_ram_cells + 1, "E");

        input_gates.push(set_address_input_gate);
        input_gates.push(set_input_gate);
        input_gates.push(enable_input_gate);
        input_gates.push(reset_input_gate);

        let mut ram_cells: Vec<Rc<RefCell<SingleRAMCell>>> = Vec::new();

        for i in 0..num_ram_cells {
            let ram_cell = SingleRAMCell::new(bus_size_in_bits);
            let ram_cell_tag = format!("ram_cell_{}", i);
            ram_cell.borrow_mut().set_tag(ram_cell_tag.as_str());
            ram_cells.push(ram_cell);
        }

        let total_bits = num_ram_cells * bus_size_in_bits;
        for i in 0..total_bits {
            let ram_cell_index = i / bus_size_in_bits;
            let bit_num = i % bus_size_in_bits;
            let output_tag = Self::get_ram_output_string(ram_cell_index, bit_num);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);
        }

        let mut ram_cell = RAMUnit {
            complex_gate: ComplexGateMembers::new(
                bus_size_in_bits + decoder_input_size * 2 + 4,
                total_bits + bus_size_in_bits,
                GateType::RAMUnitType,
                input_gates,
                output_gates,
            ),
            memory_address_register: VariableBitMemoryCell::new(decoder_input_size * 2),
            horizontal_decoder: VariableDecoder::new(decoder_input_size),
            horizontal_decoder_splitter: Splitter::new(num_ram_cells_in_row, num_ram_cells_in_row),
            vertical_decoder: VariableDecoder::new(decoder_input_size),
            vertical_decoder_splitter: Splitter::new(num_ram_cells_in_row, num_ram_cells_in_row),
            controlled_buffer: ControlledBuffer::new(bus_size_in_bits),
            ram_cells,
        };

        ram_cell.build_and_prime_circuit(
            bus_size_in_bits,
            decoder_input_size,
            num_ram_cells_in_row,
            num_ram_cells,
            output_gates_logic,
        );

        Rc::new(RefCell::new(ram_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        bus_size_in_bits: usize,
        decoder_input_size: usize,
        num_ram_cells_in_row: usize,
        num_ram_cells: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let set_address_input_index = self.get_index_from_tag("SA");
        let reset_input_index = self.get_index_from_tag("R");
        let set_input_index = self.get_index_from_tag("S");
        let enable_input_index = self.get_index_from_tag("E");

        let set_address_input_gate = self.complex_gate.input_gates[set_address_input_index].clone();
        let set_input_gate = self.complex_gate.input_gates[set_input_index].clone();
        let enable_input_gate = self.complex_gate.input_gates[enable_input_index].clone();
        let reset_input_gate = self.complex_gate.input_gates[reset_input_index].clone();

        let memory_address_reg_set_index = self.memory_address_register.borrow_mut().get_index_from_tag("S");
        set_address_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            memory_address_reg_set_index,
            self.memory_address_register.clone(),
        );

        for i in 0..(2 * decoder_input_size) {
            let input_tag = format!("addr_{}", i);
            let input_index = self.get_index_from_tag(input_tag.as_str());

            self.complex_gate.input_gates[input_index].borrow_mut().connect_output_to_next_gate(
                0,
                i,
                self.memory_address_register.clone(),
            );
        }

        for i in 0..decoder_input_size {
            self.memory_address_register.borrow_mut().connect_output_to_next_gate(
                i,
                i,
                self.vertical_decoder.clone(),
            );
        }

        for i in decoder_input_size..(2 * decoder_input_size) {
            self.memory_address_register.borrow_mut().connect_output_to_next_gate(
                i,
                i - decoder_input_size,
                self.horizontal_decoder.clone(),
            );
        }

        for i in 0..num_ram_cells_in_row {
            self.horizontal_decoder.borrow_mut().connect_output_to_next_gate(
                i,
                i,
                self.horizontal_decoder_splitter.clone(),
            );

            self.vertical_decoder.borrow_mut().connect_output_to_next_gate(
                i,
                i,
                self.vertical_decoder_splitter.clone(),
            );
        }

        for i in 0..num_ram_cells_in_row {
            for j in 0..num_ram_cells_in_row {
                let ram_cell_idx = i * num_ram_cells_in_row + j;
                let ram_cell_horizontal_index = self.ram_cells[ram_cell_idx].borrow_mut().get_index_from_tag("H");
                let decoder_idx = self.horizontal_decoder_splitter.borrow_mut().get_index_for_output(
                    i, j,
                );

                self.horizontal_decoder_splitter.borrow_mut().connect_output_to_next_gate(
                    decoder_idx,
                    ram_cell_horizontal_index,
                    self.ram_cells[ram_cell_idx].clone(),
                );

                let ram_cell_idx = j * num_ram_cells_in_row + i;
                let ram_cell_vertical_index = self.ram_cells[ram_cell_idx].borrow_mut().get_index_from_tag("V");
                let decoder_idx = self.vertical_decoder_splitter.borrow_mut().get_index_for_output(
                    i, j,
                );

                self.vertical_decoder_splitter.borrow_mut().connect_output_to_next_gate(
                    decoder_idx,
                    ram_cell_vertical_index,
                    self.ram_cells[ram_cell_idx].clone(),
                );
            }
        }

        for i in 0..num_ram_cells {
            let ram_cell_enable_index = self.ram_cells[i].borrow_mut().get_index_from_tag("E");
            let ram_cell_set_index = self.ram_cells[i].borrow_mut().get_index_from_tag("S");
            let ram_cell_reset_index = self.ram_cells[i].borrow_mut().get_index_from_tag("R");

            enable_input_gate.borrow_mut().connect_output_to_next_gate(
                i,
                ram_cell_enable_index,
                self.ram_cells[i].clone(),
            );

            set_input_gate.borrow_mut().connect_output_to_next_gate(
                i,
                ram_cell_set_index,
                self.ram_cells[i].clone(),
            );

            reset_input_gate.borrow_mut().connect_output_to_next_gate(
                i,
                ram_cell_reset_index,
                self.ram_cells[i].clone(),
            );

            for j in 0..bus_size_in_bits {
                self.complex_gate.input_gates[j].borrow_mut().connect_output_to_next_gate(
                    i,
                    j,
                    self.ram_cells[i].clone(),
                );

                self.ram_cells[i].borrow_mut().connect_output_to_next_gate(
                    j,
                    j,
                    self.controlled_buffer.clone(),
                );

                let output_tag = Self::get_ram_output_string(i, j);
                let output_index = self.get_index_from_tag(output_tag.as_str());
                let reg_output_tag = format!("reg_{}", j);
                let reg_output_index = self.ram_cells[i].borrow_mut().get_index_from_tag(reg_output_tag.as_str());
                self.ram_cells[i].borrow_mut().connect_output_to_next_gate(
                    reg_output_index,
                    0,
                    output_gates[output_index].clone(),
                );
            }
        }

        let controlled_buffer_enable_index = self.controlled_buffer.borrow_mut().get_index_from_tag("E");
        enable_input_gate.borrow_mut().connect_output_to_next_gate(
            num_ram_cells,
            controlled_buffer_enable_index,
            self.controlled_buffer.clone(),
        );

        for j in 0..bus_size_in_bits {
            self.controlled_buffer.borrow_mut().connect_output_to_next_gate(
                j,
                0,
                output_gates[j].clone(),
            );
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            Some(GateType::VariableSingleRAMCellType),
        );
    }
}

impl LogicGate for RAMUnit {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //todo d
        if self.get_tag() == "ram" {
            println!("input: {:?}", input);
        }
        //ActiveLowSRLatch has an `invalid` state of LOW LOW. However, this is not being enforced by
        // assertions because it may be an intermediate state.
        self.complex_gate.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        //The second gate_type parameter will guarantee that all Single RAM cells run on the same
        // clock tick for efficiency.
        let result = self.complex_gate.fetch_output_signals(
            &self.get_tag(),
            Some(GateType::VariableSingleRAMCellType),
        );

        // if self.simple_gate.should_print_output {
        //     if tag.is_empty() {
        //         println!(
        //             "{} gate id {}\ninput is {:#?}\noutput is {:#?}",
        //             gate_type,
        //             unique_id.id(),
        //             input,
        //             output,
        //         );
        //     } else {
        //         println!(
        //             "{} gate tag {} id {}\ninput is {:#?}\noutput is {:#?}",
        //             gate_type,
        //             tag,
        //             unique_id.id(),
        //             input,
        //             output,
        //         );
        //     }
        // }

        result
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

//This is a higher level thing for the CPU to connect to and add one.
pub struct VariableBitBusOne {
    complex_gate: ComplexGateMembers,
    and_gates: Vec<Rc<RefCell<And>>>,
    or_gate: Rc<RefCell<Or>>,
    not_gate: Rc<RefCell<Not>>,
}

#[allow(dead_code)]
impl VariableBitBusOne {
    pub fn new(number_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let mut and_gates = Vec::new();

        build_simple_inputs_and_outputs_with_and(
            number_bits,
            &mut input_gates,
            &mut output_gates,
            &mut output_gates_logic,
            &mut and_gates,
        );

        and_gates.pop();

        let set_input_gate = SimpleInput::new(2, "BUS_1");

        input_gates.push(set_input_gate.clone());

        let mut bit_register = VariableBitBusOne {
            complex_gate: ComplexGateMembers::new(
                number_bits + 1,
                number_bits,
                GateType::VariableBitBusOneType,
                input_gates,
                output_gates,
            ),
            and_gates,
            or_gate: Or::new(2, 1),
            not_gate: Not::new(number_bits - 1),
        };

        bit_register.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(bit_register))
    }

    fn build_and_prime_circuit(
        &mut self,
        number_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let bus_one_input = self.complex_gate.input_gates[self.get_index_from_tag("BUS_1")].clone();

        bus_one_input.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.not_gate.clone(),
        );

        for i in 0..number_bits {
            let mut input_gate = self.complex_gate.input_gates[i].borrow_mut();

            if i == 0 {
                input_gate.connect_output_to_next_gate(
                    0,
                    0,
                    self.or_gate.clone(),
                );

                bus_one_input.borrow_mut().connect_output_to_next_gate(
                    1,
                    1,
                    self.or_gate.clone(),
                );

                self.or_gate.borrow_mut().connect_output_to_next_gate(
                    0,
                    0,
                    output_gates[i].clone(),
                );
            } else {
                input_gate.connect_output_to_next_gate(
                    0,
                    0,
                    self.and_gates[i - 1].clone(),
                );

                self.not_gate.borrow_mut().connect_output_to_next_gate(
                    i - 1,
                    1,
                    self.and_gates[i - 1].clone(),
                );

                self.and_gates[i - 1].borrow_mut().connect_output_to_next_gate(
                    0,
                    0,
                    output_gates[i].clone(),
                );
            }
        }

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for VariableBitBusOne {
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
    use crate::logic::foundations::Signal::{HIGH, LOW_, NONE};
    use rand::Rng;
    use crate::test_stuff::run_multi_input_output_logic_gate;
    use super::*;

    #[test]
    fn processor_register_initialization() {
        let num_bits = rand::thread_rng().gen_range(1..=16);
        let register = VariableBitRegister::new(num_bits);

        let output = register.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), 2 * num_bits);
        for (i, out) in output.into_iter().enumerate() {
            match out {
                GateOutputState::NotConnected(signal) => {
                    if i < num_bits {
                        assert_eq!(signal, NONE);
                    } else {
                        assert_eq!(signal, LOW_);
                    }
                }
                GateOutputState::Connected(_) => panic!("Final output gate should never be connected.")
            }
        }
    }

    #[test]
    fn processor_register_simple_test() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, HIGH, LOW_]
            ],
            vec![
                vec![HIGH, HIGH, LOW_, HIGH, HIGH, LOW_],
            ],
            HashMap::from(
                [
                    ("S", vec![vec![HIGH]]),
                    ("E", vec![vec![HIGH]])
                ],
            ),
            VariableBitRegister::new(3),
        );
    }

    #[test]
    fn processor_register_set_bit_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW_, HIGH],
                vec![HIGH, HIGH, LOW_],
            ],
            vec![
                vec![HIGH, LOW_, HIGH, HIGH, LOW_, HIGH],
                vec![HIGH, LOW_, HIGH, HIGH, LOW_, HIGH],
            ],
            HashMap::from(
                [
                    ("S", vec![vec![HIGH], vec![LOW_]]),
                    ("E", vec![vec![HIGH], vec![HIGH]])
                ],
            ),
            VariableBitRegister::new(3),
        );
    }

    #[test]
    fn processor_register_enable_bit_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW_, HIGH, HIGH],
                vec![HIGH, LOW_, HIGH, HIGH],
                vec![HIGH, HIGH, LOW_, LOW_],
            ],
            vec![
                vec![HIGH, LOW_, HIGH, HIGH, HIGH, LOW_, HIGH, HIGH],
                vec![NONE, NONE, NONE, NONE, HIGH, LOW_, HIGH, HIGH],
                vec![NONE, NONE, NONE, NONE, HIGH, LOW_, HIGH, HIGH],
            ],
            HashMap::from(
                [
                    ("S", vec![
                        vec![HIGH],
                        vec![HIGH],
                        vec![LOW_],
                    ]),
                    ("E", vec![
                        vec![HIGH],
                        vec![LOW_],
                        vec![LOW_],
                    ])
                ],
            ),
            VariableBitRegister::new(4),
        );
    }

    #[test]
    fn decoder_initialization() {
        let num_bits = rand::thread_rng().gen_range(1..=16);
        let register = VariableDecoder::new(num_bits);

        let output = register.borrow_mut().fetch_output_signals().unwrap();

        assert_eq!(output.len(), usize::pow(2, num_bits as u32));
        for (i, out) in output.into_iter().enumerate() {
            match out {
                GateOutputState::NotConnected(signal) => {
                    if i == 0 {
                        assert_eq!(signal, HIGH);
                    } else {
                        assert_eq!(signal, LOW_);
                    }
                }
                GateOutputState::Connected(_) => panic!("Final output gate should never be connected.")
            }
        }
    }

    #[test]
    fn decoder_all_numbers() {
        let number_inputs = rand::thread_rng().gen_range(1..=5);
        let number_outputs = usize::pow(2, number_inputs as u32);
        let decoder = VariableDecoder::new(number_inputs);

        let mut input_vector = Vec::new();
        let mut output_vector = Vec::new();
        for i in 0..number_outputs {
            let binary_input_number = format!("{:0width$b}", i, width = number_inputs);

            let mut i_vector = Vec::with_capacity(number_inputs);
            for c in binary_input_number.chars() {
                if c == '0' {
                    i_vector.push(LOW_);
                } else {
                    i_vector.push(HIGH);
                }
            }
            input_vector.push(i_vector);

            let mut o_vector = vec![LOW_; number_outputs];
            o_vector[i] = HIGH;

            output_vector.push(o_vector);
        }

        run_multi_input_output_logic_gate(
            input_vector,
            output_vector,
            HashMap::new(),
            decoder.clone(),
        );
    }

    fn single_ram_cell_low_v_h(
        v_signal: Signal,
        h_signal: Signal,
    ) {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH],
            ],
            vec![
                vec![NONE, LOW_],
            ],
            HashMap::from(
                [
                    ("R", vec![vec![LOW_]]),
                    ("H", vec![vec![h_signal]]),
                    ("V", vec![vec![v_signal]]),
                    ("S", vec![vec![HIGH]]),
                    ("E", vec![vec![HIGH]]),
                ]
            ),
            SingleRAMCell::new(1),
        );
    }

    #[test]
    fn single_ram_cell_low_v() {
        single_ram_cell_low_v_h(LOW_, HIGH);
    }

    #[test]
    fn single_ram_cell_low_h() {
        single_ram_cell_low_v_h(HIGH, LOW_);
    }

    fn vec_with_values<T: Clone>(val1: T, x: usize, val2: T, y: usize) -> Vec<T> {
        std::iter::repeat(val1).take(x).chain(std::iter::repeat(val2).take(y)).collect()
    }

    #[test]
    fn single_ram_cell_reset() {
        let num_bits = 8;
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH; num_bits],
                vec![HIGH; num_bits],
                vec![LOW_; num_bits],
            ],
            vec![
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(HIGH, num_bits, HIGH, num_bits),
            ],
            HashMap::from(
                [
                    ("R", vec![vec![HIGH], vec![LOW_], vec![LOW_]]),
                    ("H", vec![vec![LOW_], vec![LOW_], vec![HIGH]]),
                    ("V", vec![vec![LOW_], vec![LOW_], vec![HIGH]]),
                    ("S", vec![vec![LOW_], vec![LOW_], vec![LOW_]]),
                    ("E", vec![vec![LOW_], vec![LOW_], vec![HIGH]]),
                ]
            ),
            SingleRAMCell::new(num_bits),
        );
    }

    #[test]
    fn single_ram_cell_set() {
        let num_bits = 6;
        run_multi_input_output_logic_gate(
            vec![
                vec![LOW_; num_bits],
                vec![HIGH; num_bits],
                vec![HIGH; num_bits],
                vec![LOW_; num_bits],
            ],
            vec![
                vec_with_values(NONE, num_bits, LOW_, num_bits),
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(HIGH, num_bits, HIGH, num_bits),
            ],
            HashMap::from(
                [
                    ("R", vec![vec![LOW_], vec![LOW_], vec![LOW_], vec![LOW_]]),
                    ("H", vec![vec![HIGH], vec![HIGH], vec![LOW_], vec![HIGH]]),
                    ("V", vec![vec![HIGH], vec![HIGH], vec![LOW_], vec![HIGH]]),
                    ("S", vec![vec![HIGH], vec![HIGH], vec![LOW_], vec![LOW_]]),
                    ("E", vec![vec![LOW_], vec![LOW_], vec![LOW_], vec![HIGH]]),
                ]
            ),
            SingleRAMCell::new(num_bits),
        );
    }

    #[test]
    fn ram_unit_test() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW_, HIGH], // Change SA bit so address can be written
                vec![HIGH, LOW_, HIGH], // Update address to RAM 3.
                vec![HIGH, LOW_, HIGH], // Change S bit so input is saved to RAM address location.
                vec![HIGH, LOW_, HIGH], // Change S bit so input is no longer saved to RAM address location.
                vec![LOW_, LOW_, LOW_], // Get output at RAM 1
                vec![LOW_, HIGH, LOW_], // Get output at RAM 2
                vec![LOW_, HIGH, HIGH], // Get output at RAM 3
                vec![HIGH, HIGH, HIGH], // Get output at RAM 4
            ],
            vec![
                vec![NONE, NONE, NONE],
                vec![NONE, NONE, NONE],
                vec![NONE, NONE, NONE],
                vec![NONE, NONE, NONE],
                vec![LOW_, LOW_, LOW_],
                vec![LOW_, LOW_, LOW_],
                vec![HIGH, LOW_, HIGH],
                vec![LOW_, LOW_, LOW_],
            ],
            HashMap::from(
                [
                    ("addr", vec![
                        vec![LOW_, LOW_], //0b00
                        vec![HIGH, LOW_], //0b10
                        vec![HIGH, LOW_], //0b10
                        vec![HIGH, LOW_], //0b10
                        vec![LOW_, LOW_], //0b00
                        vec![LOW_, HIGH], //0b01
                        vec![HIGH, LOW_], //0b10
                        vec![HIGH, HIGH], //0b11
                    ]),
                    ("SA", vec![vec![HIGH]; 8]),
                    ("R", vec![vec![LOW_]; 8]),
                    ("S", vec![
                        vec![LOW_],
                        vec![LOW_],
                        vec![HIGH],
                        vec![LOW_],
                        vec![LOW_],
                        vec![LOW_],
                        vec![LOW_],
                        vec![LOW_],
                    ]),
                    ("E", vec_with_values(vec![LOW_], 4, vec![HIGH], 4), )
                ]
            ),
            RAMUnit::new(3, 1),
        );
    }

    #[test]
    fn variable_bit_bus_one_test() {
        //If the BUS_1 input is HIGH, the output returns one. Otherwise, it passes the input
        // through.
        let num_bits = 8;
        let mut one_signals = vec![LOW_; num_bits];
        one_signals[0] = HIGH;
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH; num_bits],
                vec![HIGH; num_bits],
            ],
            vec![
                vec![HIGH; num_bits],
                one_signals,
            ],
            HashMap::from(
                [
                    ("BUS_1", vec![vec![LOW_], vec![HIGH]]),
                ]
            ),
            VariableBitBusOne::new(num_bits),
        );
    }
}
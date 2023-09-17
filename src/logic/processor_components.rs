use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::{And, ControlledBuffer, Not, Or, Splitter};
use crate::logic::complex_logic::VariableBitCPUEnable;
use crate::logic::foundations::{build_simple_inputs_and_outputs, ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, push_reg_outputs_to_output_gates, Signal, UniqueID};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW, HIGH};
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
        self.complex_gate.calculate_output_from_inputs(true);
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
    complex_gate: ComplexGateMembers,
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
        self.complex_gate.calculate_output_from_inputs(true);
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
        self.complex_gate.calculate_output_from_inputs(true);
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

        for _ in 0..num_ram_cells {
            ram_cells.push(SingleRAMCell::new(bus_size_in_bits));
        }

        let mut ram_cell = RAMUnit {
            complex_gate: ComplexGateMembers::new(
                bus_size_in_bits + decoder_input_size * 2 + 4,
                bus_size_in_bits,
                GateType::VariableSingleRAMCellType,
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
                self.horizontal_decoder.clone(),
            );
        }

        for i in decoder_input_size..(2 * decoder_input_size) {
            self.memory_address_register.borrow_mut().connect_output_to_next_gate(
                i,
                i - decoder_input_size,
                self.vertical_decoder.clone(),
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

        //TODO: can I make this one loop with the loop below?
        for i in 0..num_ram_cells_in_row {
            for j in 0..num_ram_cells_in_row {
                let ram_cell_idx = i*num_ram_cells_in_row + j;
                let ram_cell_horizontal_index = self.ram_cells[ram_cell_idx].borrow_mut().get_index_from_tag("H");
                let decoder_idx = self.horizontal_decoder_splitter.borrow_mut().get_index_for_output(
                     i, j
                );
                self.horizontal_decoder_splitter.borrow_mut().connect_output_to_next_gate(
                    decoder_idx,
                    ram_cell_horizontal_index,
                    self.ram_cells[ram_cell_idx].clone(),
                );

                let ram_cell_idx = j*num_ram_cells_in_row + i;
                let ram_cell_vertical_index = self.ram_cells[ram_cell_idx].borrow_mut().get_index_from_tag("V");
                let decoder_idx = self.vertical_decoder_splitter.borrow_mut().get_index_for_output(
                    j, i
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
        self.complex_gate.calculate_output_from_inputs(true);
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
    use crate::logic::foundations::Signal::{HIGH, LOW, NONE};
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
                        assert_eq!(signal, LOW);
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
                vec![HIGH, HIGH, LOW]
            ],
            vec![
                vec![HIGH, HIGH, LOW, HIGH, HIGH, LOW],
            ],
            HashMap::from(
                [
                    ("S", vec![HIGH]),
                    ("E", vec![HIGH])
                ],
            ),
            VariableBitRegister::new(3),
        );
    }

    #[test]
    fn processor_register_set_bit_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW, HIGH],
                vec![HIGH, HIGH, LOW],
            ],
            vec![
                vec![HIGH, LOW, HIGH, HIGH, LOW, HIGH],
                vec![HIGH, LOW, HIGH, HIGH, LOW, HIGH],
            ],
            HashMap::from(
                [
                    ("S", vec![HIGH, LOW]),
                    ("E", vec![HIGH, HIGH])
                ],
            ),
            VariableBitRegister::new(3),
        );
    }

    #[test]
    fn processor_register_enable_bit_low() {
        run_multi_input_output_logic_gate(
            vec![
                vec![HIGH, LOW, HIGH, HIGH],
                vec![HIGH, LOW, HIGH, HIGH],
                vec![HIGH, HIGH, LOW, LOW],
            ],
            vec![
                vec![HIGH, LOW, HIGH, HIGH, HIGH, LOW, HIGH, HIGH],
                vec![NONE, NONE, NONE, NONE, HIGH, LOW, HIGH, HIGH],
                vec![NONE, NONE, NONE, NONE, HIGH, LOW, HIGH, HIGH],
            ],
            HashMap::from(
                [
                    ("S", vec![HIGH, HIGH, LOW]),
                    ("E", vec![HIGH, LOW, LOW])
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

        println!("output_ {:#?}", output);
        assert_eq!(output.len(), usize::pow(2, num_bits as u32));
        for (i, out) in output.into_iter().enumerate() {
            match out {
                GateOutputState::NotConnected(signal) => {
                    if i == 0 {
                        assert_eq!(signal, HIGH);
                    } else {
                        assert_eq!(signal, LOW);
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
                    i_vector.push(LOW);
                } else {
                    i_vector.push(HIGH);
                }
            }
            input_vector.push(i_vector);

            let mut o_vector = vec![LOW; number_outputs];
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
                vec![NONE, LOW],
            ],
            HashMap::from(
                [
                    ("R", vec![LOW]),
                    ("H", vec![h_signal]),
                    ("V", vec![v_signal]),
                    ("S", vec![HIGH]),
                    ("E", vec![HIGH]),
                ]
            ),
            SingleRAMCell::new(1),
        );
    }

    #[test]
    fn single_ram_cell_low_v() {
        single_ram_cell_low_v_h(LOW, HIGH);
    }

    #[test]
    fn single_ram_cell_low_h() {
        single_ram_cell_low_v_h(HIGH, LOW);
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
                vec![LOW; num_bits],
            ],
            vec![
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(HIGH, num_bits, HIGH, num_bits),
            ],
            HashMap::from(
                [
                    ("R", vec![HIGH, LOW, LOW]),
                    ("H", vec![LOW, LOW, HIGH]),
                    ("V", vec![LOW, LOW, HIGH]),
                    ("S", vec![LOW, LOW, LOW]),
                    ("E", vec![LOW, LOW, HIGH]),
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
                vec![LOW; num_bits],
                vec![HIGH; num_bits],
                vec![HIGH; num_bits],
                vec![LOW; num_bits],
            ],
            vec![
                vec_with_values(NONE, num_bits, LOW, num_bits),
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(NONE, num_bits, HIGH, num_bits),
                vec_with_values(HIGH, num_bits, HIGH, num_bits),
            ],
            HashMap::from(
                [
                    ("R", vec![LOW, LOW, LOW, LOW]),
                    ("H", vec![HIGH, HIGH, LOW, HIGH]),
                    ("V", vec![HIGH, HIGH, LOW, HIGH]),
                    ("S", vec![HIGH, HIGH, LOW, LOW]),
                    ("E", vec![LOW, LOW, LOW, HIGH]),
                ]
            ),
            SingleRAMCell::new(num_bits),
        );
    }

    //TODO: RAMUnit tests
    #[test]
    fn ram_unit_test() {
        //TODO: Honestly, this is pretty complex, I should probably draw the gates out and print
        // the ids to make sure the connections are correct.
        run_multi_input_output_logic_gate(
            vec![
                vec![LOW],
            ],
            vec![
                vec![LOW]
            ],
            HashMap::from(
                [
                    ("SA", vec![HIGH]),
                    ("R", vec![LOW]),
                    ("S", vec![HIGH]),
                    ("E", vec![HIGH]),
                ]
            ),
            RAMUnit::new(1, 1),
        );
    }
}
use crate::logic::arithmetic_gates::ArithmeticLogicUnit;
use crate::logic::basic_gates::{And, ControlledBuffer, Not, Splitter};
use crate::logic::complex_logic::{FourCycleClockHookup, VariableBitCounter, VariableBitMultiplexer};
use crate::logic::control_section::ControlSection;

use crate::logic::foundations::{ComplexGateMembers, connect_gates, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW_, HIGH};
use crate::logic::input_gates::{Clock, SimpleInput};
use crate::logic::memory_gates::{OneBitMemoryCell, VariableBitMemoryCell};
use crate::logic::processor_components::{RAMUnit, VariableBitBusOne, VariableBitRegister};
use crate::shared_mutex::{new_shared_mutex, SharedMutex};

#[allow(dead_code)]
#[derive(Clone)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
}

#[allow(dead_code)]
impl Register {
    fn binary(reg: Register) -> &'static str {
        match reg {
            Register::R0 => "00",
            Register::R1 => "01",
            Register::R2 => "10",
            Register::R3 => "11",
        }
    }

    fn get_variable_bit_tag(&self) -> &'static str {
        match self {
            Register::R0 => VariableBitCPU::R0,
            Register::R1 => VariableBitCPU::R1,
            Register::R2 => VariableBitCPU::R2,
            Register::R3 => VariableBitCPU::R3,
        }
    }
}

#[allow(dead_code)]
pub enum ALUInstruction {
    ADD,
    SHR,
    SHL,
    NOT,
    AND,
    OR,
    XOR,
    CMP,
}

#[allow(dead_code)]
impl ALUInstruction {
    fn binary(opt: ALUInstruction) -> &'static str {
        match opt {
            ALUInstruction::ADD => "000",
            ALUInstruction::SHR => "001",
            ALUInstruction::SHL => "010",
            ALUInstruction::NOT => "011",
            ALUInstruction::AND => "100",
            ALUInstruction::OR => "101",
            ALUInstruction::XOR => "110",
            ALUInstruction::CMP => "111", //Not hooked up
        }
    }
}

#[allow(dead_code)]
//8 bit instructions.
pub enum Instructions {
    End,
    ALU { opt: ALUInstruction, reg_a: Register, reg_b: Register },
    // Load contents of register reg_b to RAM address inside reg_a.
    Store { reg_a: Register, reg_b: Register },
    // Store contents of register reg_b to RAM address inside reg_a.
    Load { reg_a: Register, reg_b: Register },
    // Loads contents of register reg_b to RAM address inside reg_a.
    Data { reg: Register },
    // Loads data at next RAM address into reg.
    JumpRegister { reg: Register },
    // Jumps to address inside reg.
    JumpAddress,
    // Jumps to address inside next RAM cell.
    JumpIf { carry: bool, a_larger: bool, equal: bool, zero: bool },
    // Jumps to address inside next RAM cell if flags are true.
    ClearFlags, //Clears flags.
}

#[allow(dead_code)]
impl Instructions {
    fn binary(instruction: Self) -> String {
        let binary_string =
            match instruction {
                Instructions::End => "11001111".to_string(),
                Instructions::Data { reg } => {
                    format!("001000{}", Register::binary(reg))
                }
                Instructions::ALU { opt, reg_a, reg_b } => {
                    format!("1{}{}{}", ALUInstruction::binary(opt), Register::binary(reg_a), Register::binary(reg_b))
                }
                Instructions::Store { reg_a, reg_b } => {
                    format!("0001{}{}", Register::binary(reg_a), Register::binary(reg_b))
                }
                Instructions::Load { reg_a, reg_b } => {
                    format!("0000{}{}", Register::binary(reg_a), Register::binary(reg_b))
                }
                Instructions::JumpRegister { reg } => {
                    format!("001100{}", Register::binary(reg))
                }
                Instructions::JumpAddress => {
                    format!("01000000")
                }
                Instructions::JumpIf { carry, a_larger, equal, zero } => {
                    fn bool_char(b: bool) -> char {
                        match b {
                            true => '1',
                            false => '0',
                        }
                    }
                    format!(
                        "0101{}{}{}{}",
                        bool_char(carry),
                        bool_char(a_larger),
                        bool_char(equal),
                        bool_char(zero)
                    )
                }
                Instructions::ClearFlags => {
                    format!("01100000")
                }
            };

        binary_string
    }
}

pub struct VariableBitCPU {
    complex_gate: ComplexGateMembers,
    four_cycle_clock_hookup: SharedMutex<FourCycleClockHookup>,
    four_cycle_clock_clk_splitter: SharedMutex<Splitter>,
    four_cycle_clock_clke_splitter: SharedMutex<Splitter>,
    four_cycle_clock_clks_splitter: SharedMutex<Splitter>,
    control_section: SharedMutex<ControlSection>,
    temp_s_splitter: SharedMutex<Splitter>,
    bus: SharedMutex<Splitter>,
    register_0: SharedMutex<VariableBitRegister>,
    register_1: SharedMutex<VariableBitRegister>,
    register_2: SharedMutex<VariableBitRegister>,
    register_3: SharedMutex<VariableBitRegister>,
    instruction_address_register: SharedMutex<VariableBitRegister>,
    instruction_register: SharedMutex<VariableBitMemoryCell>,
    ram: SharedMutex<RAMUnit>,
    alu: SharedMutex<ArithmeticLogicUnit>,
    bus_1: SharedMutex<VariableBitBusOne>,
    tmp: SharedMutex<VariableBitMemoryCell>,
    c_tmp: SharedMutex<OneBitMemoryCell>,
    c_tmp_and: SharedMutex<And>,
    acc: SharedMutex<VariableBitRegister>,
    flags: SharedMutex<VariableBitMemoryCell>,
    flags_c_out_splitter: SharedMutex<Splitter>,
    end_input_and_gate: SharedMutex<And>,
    end_input_not_gate: SharedMutex<Not>,
    load_multiplexer: SharedMutex<VariableBitMultiplexer>,
    load_counter: SharedMutex<VariableBitCounter>,
    counter_controlled_buffer: SharedMutex<ControlledBuffer>,
    counter_and: SharedMutex<And>,
    load_input_splitter: SharedMutex<Splitter>,
    reset_controlled_buffer: SharedMutex<ControlledBuffer>,
}

#[allow(dead_code)]
impl VariableBitCPU {
    //Inputs
    pub const LOAD: &'static str = "LOAD";
    pub const RESET: &'static str = "RESET";
    pub const MARS: &'static str = "MARS";
    pub const RAM: &'static str = "RAM";
    pub const CLK_IN: &'static str = "CLK_IN";

    //Outputs
    pub const R0: &'static str = "R0";
    pub const R1: &'static str = "R1";
    pub const R2: &'static str = "R2";
    pub const R3: &'static str = "R3";
    pub const IR: &'static str = "IR";
    pub const IAR: &'static str = "IAR";
    pub const ACC: &'static str = "ACC";
    pub const TMP: &'static str = "TMP";
    pub const BUS: &'static str = "BUS";
    pub const CLK_OUT: &'static str = "CLK_OUT";
    pub const CLKE: &'static str = "CLKE";
    pub const CLKS: &'static str = "CLKS";
    pub const IO: &'static str = "IO";
    pub const DA: &'static str = "DA";
    pub const END: &'static str = "END";
    pub const IO_CLK_S: &'static str = "IO_CLK_S";
    pub const IO_CLK_E: &'static str = "IO_CLK_E";
    //RAM Cells as well RAMUnit::get_ram_output_string()

    pub fn new(number_bits: usize, ram_cells_decoder_input: usize) -> SharedMutex<Self> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        for i in 0..number_bits {
            let input_tag = format!("{}_{}", Self::RAM, i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));
        }

        input_gates.push(SimpleInput::new(1, VariableBitCPU::LOAD));
        input_gates.push(SimpleInput::new(3, VariableBitCPU::RESET));
        input_gates.push(SimpleInput::new(1, VariableBitCPU::MARS));

        //This must be the last input.
        input_gates.push(SimpleInput::new(1, VariableBitCPU::CLK_IN));

        let mut store_output = |multi_bit_output: bool, tag: &str| {
            if multi_bit_output {
                for i in 0..number_bits {
                    let output_tag = format!("{}_{}", tag, i);
                    let gate = SimpleOutput::new(output_tag.as_str());

                    output_gates.push(gate.clone());
                    output_gates_logic.push(gate.clone());
                }
            } else {
                let gate = SimpleOutput::new(tag);
                output_gates.push(gate.clone());
                output_gates_logic.push(gate.clone());
            }
        };

        store_output(true, VariableBitCPU::R0);
        store_output(true, VariableBitCPU::R1);
        store_output(true, VariableBitCPU::R2);
        store_output(true, VariableBitCPU::R3);
        store_output(true, VariableBitCPU::IR);
        store_output(true, VariableBitCPU::IAR);
        store_output(true, VariableBitCPU::ACC);
        store_output(true, VariableBitCPU::TMP);
        store_output(true, VariableBitCPU::BUS);

        store_output(false, VariableBitCPU::CLK_OUT);
        store_output(false, VariableBitCPU::CLKE);
        store_output(false, VariableBitCPU::CLKS);
        store_output(false, VariableBitCPU::IO);
        store_output(false, VariableBitCPU::DA);
        store_output(false, VariableBitCPU::END);
        store_output(false, VariableBitCPU::IO_CLK_E);
        store_output(false, VariableBitCPU::IO_CLK_S);

        let num_ram_cells = usize::pow(2, (2 * ram_cells_decoder_input) as u32);
        for i in 0..num_ram_cells {
            for j in 0..number_bits {
                let output_tag = RAMUnit::get_ram_output_string(i, j);
                let output_gate = SimpleOutput::new(output_tag.as_str());
                output_gates.push(output_gate.clone());
                output_gates_logic.push(output_gate);
            }
        }

        let mut cpu = VariableBitCPU {
            complex_gate: ComplexGateMembers::new(
                number_bits + 4,
                num_ram_cells * number_bits + 9 * number_bits + 8,
                GateType::VariableBitCPUType,
                input_gates,
                output_gates,
            ),
            four_cycle_clock_hookup: FourCycleClockHookup::new(),
            four_cycle_clock_clk_splitter: Splitter::new(1, 3),
            four_cycle_clock_clke_splitter: Splitter::new(1, 2),
            four_cycle_clock_clks_splitter: Splitter::new(1, 2),
            control_section: ControlSection::new(number_bits),
            temp_s_splitter: Splitter::new(1, 2),
            bus: Splitter::new(number_bits, 11),
            register_0: VariableBitRegister::new(number_bits),
            register_1: VariableBitRegister::new(number_bits),
            register_2: VariableBitRegister::new(number_bits),
            register_3: VariableBitRegister::new(number_bits),
            instruction_address_register: VariableBitRegister::new(number_bits),
            instruction_register: VariableBitMemoryCell::new(number_bits),
            ram: RAMUnit::new(number_bits, ram_cells_decoder_input),
            alu: ArithmeticLogicUnit::new(number_bits),
            bus_1: VariableBitBusOne::new(number_bits),
            tmp: VariableBitMemoryCell::new(number_bits),
            c_tmp: OneBitMemoryCell::new(1),
            c_tmp_and: And::new(2, 1),
            acc: VariableBitRegister::new(number_bits),
            flags: VariableBitMemoryCell::new(4), //size 4 for the alu outputs
            flags_c_out_splitter: Splitter::new(1, 2),
            end_input_and_gate: And::new(2, 1),
            end_input_not_gate: Not::new(1),
            load_multiplexer: VariableBitMultiplexer::new(number_bits, 2),
            load_counter: VariableBitCounter::new(2 * ram_cells_decoder_input), //This is done so load will properly complete with the counter at 0.
            counter_controlled_buffer: ControlledBuffer::new(2 * ram_cells_decoder_input),
            counter_and: And::new(2, 1),
            load_input_splitter: Splitter::new(1, 4),
            reset_controlled_buffer: ControlledBuffer::new(number_bits),
        };

        cpu.four_cycle_clock_hookup.lock().unwrap().set_tag("four_cycle_clock_hookup");
        cpu.four_cycle_clock_clk_splitter.lock().unwrap().set_tag("four_cycle_clock_clk_splitter");
        cpu.four_cycle_clock_clke_splitter.lock().unwrap().set_tag("four_cycle_clock_clke_splitter");
        cpu.four_cycle_clock_clks_splitter.lock().unwrap().set_tag("four_cycle_clock_clks_splitter");
        cpu.control_section.lock().unwrap().set_tag("control_section");
        cpu.temp_s_splitter.lock().unwrap().set_tag("temp_s_splitter");
        cpu.bus.lock().unwrap().set_tag("bus");
        cpu.register_0.lock().unwrap().set_tag("register_0");
        cpu.register_1.lock().unwrap().set_tag("register_1");
        cpu.register_2.lock().unwrap().set_tag("register_2");
        cpu.register_3.lock().unwrap().set_tag("register_3");
        cpu.instruction_address_register.lock().unwrap().set_tag("instruction_address_register");
        cpu.instruction_register.lock().unwrap().set_tag("instruction_register");
        cpu.ram.lock().unwrap().set_tag("ram");
        cpu.alu.lock().unwrap().set_tag("alu");
        cpu.bus_1.lock().unwrap().set_tag("bus_1");
        cpu.tmp.lock().unwrap().set_tag("tmp");
        cpu.c_tmp.lock().unwrap().set_tag("c_tmp");
        cpu.c_tmp_and.lock().unwrap().set_tag("c_tmp_and");
        cpu.acc.lock().unwrap().set_tag("acc");
        cpu.flags.lock().unwrap().set_tag("flags");
        cpu.flags_c_out_splitter.lock().unwrap().set_tag("flags_c_out_splitter");
        cpu.end_input_and_gate.lock().unwrap().set_tag("end_input_and_gate");
        cpu.end_input_not_gate.lock().unwrap().set_tag("end_input_not_gate");
        cpu.load_multiplexer.lock().unwrap().set_tag("load_multiplexer");
        cpu.load_counter.lock().unwrap().set_tag("load_counter");
        cpu.counter_and.lock().unwrap().set_tag("counter_and");
        cpu.counter_controlled_buffer.lock().unwrap().set_tag("counter_controlled_buffer");
        cpu.load_input_splitter.lock().unwrap().set_tag("load_input_splitter");
        cpu.reset_controlled_buffer.lock().unwrap().set_tag("reset_controlled_buffer");

        cpu.four_cycle_clock_hookup.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.control_section.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.register_0.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.register_1.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.register_2.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.register_3.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.instruction_address_register.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.instruction_register.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.ram.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.alu.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.tmp.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.acc.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.flags.lock().unwrap().toggle_print_each_input_output_gate(false);
        cpu.load_counter.lock().unwrap().toggle_print_each_input_output_gate(false);

        cpu.build_and_prime_circuit(
            number_bits,
            ram_cells_decoder_input,
            num_ram_cells,
            output_gates_logic,
        );

        new_shared_mutex(cpu)
    }

    fn build_and_prime_circuit(
        &mut self,
        bus_size: usize,
        ram_cells_decoder_input: usize,
        num_ram_cells: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        self.connect_inputs(bus_size);

        self.connect_control_section(&output_gates);
        self.connect_four_cycle_clock_hookup();
        self.connect_four_cycle_clock_clk_splitter(&output_gates);
        self.connect_four_cycle_clock_clke_splitter(&output_gates);
        self.connect_four_cycle_clock_clks_splitter(&output_gates);
        self.connect_bus(bus_size, ram_cells_decoder_input, &output_gates);
        self.connect_register_0(bus_size, &output_gates);
        self.connect_register_1(bus_size, &output_gates);
        self.connect_register_2(bus_size, &output_gates);
        self.connect_register_3(bus_size, &output_gates);
        self.connect_instruction_address_register(bus_size, &output_gates);
        self.connect_instruction_register(bus_size, &output_gates);
        self.connect_ram(bus_size, num_ram_cells, &output_gates);
        self.connect_alu(bus_size);
        self.connect_bus_1(bus_size);
        self.connect_tmp(bus_size, &output_gates);
        self.connect_c_tmp();
        self.connect_c_tmp_and();
        self.connect_acc(bus_size, &output_gates);
        self.connect_flags();
        self.connect_end_input_and_gate();
        self.connect_end_input_not_gate();
        self.connect_load_multiplexer(bus_size);
        self.connect_load_counter(ram_cells_decoder_input);
        self.connect_counter_controlled_buffer(ram_cells_decoder_input);
        self.connect_counter_and();
        self.connect_load_input_splitter();
        self.connect_reset_controlled_buffer(bus_size);

        //The clock input gate should not be seeded by priming the system.
        let clock_input_gate = self.complex_gate.input_gates.pop().unwrap();

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );

        self.complex_gate.input_gates.push(clock_input_gate);
    }

    fn connect_input_to_output(
        bus_size: usize,
        start_gate: SharedMutex<dyn LogicGate>,
        end_gate: SharedMutex<dyn LogicGate>,
        input_val: &str,
    ) {
        for i in 0..bus_size {
            let input_tag = format!("{}_{}", input_val, i);
            let output_tag = format!("o_{}", i);
            let input_index = end_gate.lock().unwrap().get_index_from_tag(input_tag.as_str());
            let output_index = start_gate.lock().unwrap().get_index_from_tag(output_tag.as_str());
            connect_gates(
                start_gate.clone(),
                output_index,
                end_gate.clone(),
                input_index,
            );
        }
    }

    fn connect_multi_bit_output(
        &mut self,
        bus_size: usize,
        start_gate: SharedMutex<dyn LogicGate>,
        input_val: &str,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        for i in 0..bus_size {
            let input_tag = format!("{}_{}", input_val, i);
            let output_tag = format!("reg_{}", i);
            let output_gate_index = self.get_index_from_tag(input_tag.as_str());
            let output_index = start_gate.lock().unwrap().get_index_from_tag(output_tag.as_str());
            connect_gates(
                start_gate.clone(),
                output_index,
                output_gates[output_gate_index].clone(),
                0,
            );
        }
    }

    fn connect_inputs(&mut self, bus_size: usize) {
        for i in 0..bus_size {
            let input_tag = format!("{}_{}", Self::RAM, i);
            let input_index = self.get_index_from_tag(input_tag.as_str());
            let input_gate = self.complex_gate.input_gates[input_index].clone();

            let multiplexer_tag = format!("I_1_bit_{}", i);
            let ram_input_index = self.load_multiplexer.lock().unwrap().get_index_from_tag(multiplexer_tag.as_str());
            connect_gates(
                input_gate.clone(),
                0,
                self.load_multiplexer.clone(),
                ram_input_index,
            );
        }

        let reset_index = self.get_index_from_tag(VariableBitCPU::RESET);
        let reset_input_gate = self.complex_gate.input_gates[reset_index].clone();

        let control_section_reset_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::HIGH_LVL_RESET);
        connect_gates(
            reset_input_gate.clone(),
            0,
            self.control_section.clone(),
            control_section_reset_index,
        );

        let ram_reset = self.ram.lock().unwrap().get_index_from_tag("R");
        connect_gates(
            reset_input_gate.clone(),
            1,
            self.ram.clone(),
            ram_reset,
        );

        let controlled_buffer_enable = self.reset_controlled_buffer.lock().unwrap().get_index_from_tag("E");
        connect_gates(
            reset_input_gate.clone(),
            2,
            self.reset_controlled_buffer.clone(),
            controlled_buffer_enable,
        );

        let mars_index = self.get_index_from_tag(VariableBitCPU::MARS);
        let mars_input_gate = self.complex_gate.input_gates[mars_index].clone();

        let control_section_mars_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::HIGH_LVL_MARS);
        connect_gates(
            mars_input_gate.clone(),
            0,
            self.control_section.clone(),
            control_section_mars_index,
        );

        let load_index = self.get_index_from_tag(VariableBitCPU::LOAD);
        let load_input_gate = self.complex_gate.input_gates[load_index].clone();

        connect_gates(
            load_input_gate.clone(),
            0,
            self.load_input_splitter.clone(),
            0,
        );

        let clk_in_index = self.get_index_from_tag(VariableBitCPU::CLK_IN);
        let clk_in_input_gate = self.complex_gate.input_gates[clk_in_index].clone();

        connect_gates(
            clk_in_input_gate.clone(),
            0,
            self.end_input_and_gate.clone(),
            0,
        );
    }

    fn connect_control_section(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let input_index = self.alu.lock().unwrap().get_index_from_tag("C");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::ALU_0);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.alu.clone(),
            input_index,
        );

        let input_index = self.alu.lock().unwrap().get_index_from_tag("B");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::ALU_1);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.alu.clone(),
            input_index,
        );

        let input_index = self.alu.lock().unwrap().get_index_from_tag("A");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::ALU_2);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.alu.clone(),
            input_index,
        );

        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::C_OUT);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.c_tmp_and.clone(),
            0,
        );

        let input_index = self.flags.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::FLAG_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.flags.clone(),
            input_index,
        );

        let input_index = self.acc.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::ACC_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.acc.clone(),
            input_index,
        );

        let input_index = self.acc.lock().unwrap().get_index_from_tag("E");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::ACC_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.acc.clone(),
            input_index,
        );

        let input_index = self.instruction_address_register.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::IAR_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.instruction_address_register.clone(),
            input_index,
        );

        let input_index = self.instruction_address_register.lock().unwrap().get_index_from_tag("E");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::IAR_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.instruction_address_register.clone(),
            input_index,
        );

        let input_index = self.instruction_register.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::IR_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.instruction_register.clone(),
            input_index,
        );

        let input_index = self.ram.lock().unwrap().get_index_from_tag("E");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::RAM_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.ram.clone(),
            input_index,
        );

        let input_index = self.ram.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::RAM_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.ram.clone(),
            input_index,
        );

        let input_index = self.ram.lock().unwrap().get_index_from_tag("SA");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::MAR_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.ram.clone(),
            input_index,
        );

        let input_index = self.register_0.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R0_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_0.clone(),
            input_index,
        );

        let input_index = self.register_0.lock().unwrap().get_index_from_tag("E");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R0_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_0.clone(),
            input_index,
        );

        let input_index = self.register_1.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R1_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_1.clone(),
            input_index,
        );

        let input_index = self.register_1.lock().unwrap().get_index_from_tag("E");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R1_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_1.clone(),
            input_index,
        );

        let input_index = self.register_2.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R2_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_2.clone(),
            input_index,
        );

        let input_index = self.register_2.lock().unwrap().get_index_from_tag("E");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R2_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_2.clone(),
            input_index,
        );

        let input_index = self.register_3.lock().unwrap().get_index_from_tag("S");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R3_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_3.clone(),
            input_index,
        );

        let input_index = self.register_3.lock().unwrap().get_index_from_tag("E");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::R3_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.register_3.clone(),
            input_index,
        );

        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::TMP_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.temp_s_splitter.clone(),
            0,
        );

        let input_index = self.tmp.lock().unwrap().get_index_from_tag("S");
        let output_index = self.temp_s_splitter.lock().unwrap().get_index_for_output(0, 0);
        connect_gates(
            self.temp_s_splitter.clone(),
            output_index,
            self.tmp.clone(),
            input_index,
        );

        let input_index = self.c_tmp.lock().unwrap().get_index_from_tag("S");
        let output_index = self.temp_s_splitter.lock().unwrap().get_index_for_output(0, 1);
        connect_gates(
            self.temp_s_splitter.clone(),
            output_index,
            self.c_tmp.clone(),
            input_index,
        );

        let input_index = self.bus_1.lock().unwrap().get_index_from_tag("BUS_1");
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::BUS_1);
        connect_gates(
            self.control_section.clone(),
            output_index,
            self.bus_1.clone(),
            input_index,
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::IO);
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::IO);
        connect_gates(
            self.control_section.clone(),
            output_index,
            output_gates[output_gate_index].clone(),
            0,
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::DA);
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::DA);
        connect_gates(
            self.control_section.clone(),
            output_index,
            output_gates[output_gate_index].clone(),
            0,
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::END);
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::END);
        connect_gates(
            self.control_section.clone(),
            output_index,
            output_gates[output_gate_index].clone(),
            0,
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::IO_CLK_E);
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::IO_CLK_E);
        connect_gates(
            self.control_section.clone(),
            output_index,
            output_gates[output_gate_index].clone(),
            0,
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::IO_CLK_S);
        let output_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::IO_CLK_S);
        connect_gates(
            self.control_section.clone(),
            output_index,
            output_gates[output_gate_index].clone(),
            0,
        );
    }

    fn connect_four_cycle_clock_hookup(&mut self) {
        let cycle_block_output = self.four_cycle_clock_hookup.lock().unwrap().get_index_from_tag(FourCycleClockHookup::CLK_OUT);
        connect_gates(
            self.four_cycle_clock_hookup.clone(),
            cycle_block_output,
            self.four_cycle_clock_clk_splitter.clone(),
            0,
        );

        let cycle_block_output = self.four_cycle_clock_hookup.lock().unwrap().get_index_from_tag(FourCycleClockHookup::CLKE);
        connect_gates(
            self.four_cycle_clock_hookup.clone(),
            cycle_block_output,
            self.four_cycle_clock_clke_splitter.clone(),
            0,
        );

        let cycle_block_output = self.four_cycle_clock_hookup.lock().unwrap().get_index_from_tag(FourCycleClockHookup::CLKS);
        connect_gates(
            self.four_cycle_clock_hookup.clone(),
            cycle_block_output,
            self.four_cycle_clock_clks_splitter.clone(),
            0,
        );
    }

    fn connect_four_cycle_clock_clk_splitter(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let cycle_block_output = self.four_cycle_clock_clk_splitter.lock().unwrap().get_index_for_output(
            0, 0,
        );
        let clock_input = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::CLOCK);
        connect_gates(
            self.four_cycle_clock_clk_splitter.clone(),
            cycle_block_output,
            self.control_section.clone(),
            clock_input,
        );

        let cycle_block_output = self.four_cycle_clock_clk_splitter.lock().unwrap().get_index_for_output(
            0, 1,
        );
        connect_gates(
            self.four_cycle_clock_clk_splitter.clone(),
            cycle_block_output,
            self.counter_and.clone(),
            1,
        );

        let output_index = self.get_index_from_tag(VariableBitCPU::CLK_OUT);
        let cycle_block_output = self.four_cycle_clock_clk_splitter.lock().unwrap().get_index_for_output(
            0, 2,
        );
        connect_gates(
            self.four_cycle_clock_clk_splitter.clone(),
            cycle_block_output,
            output_gates[output_index].clone(),
            0,
        );
    }

    fn connect_four_cycle_clock_clke_splitter(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let cycle_block_output = self.four_cycle_clock_clke_splitter.lock().unwrap().get_index_for_output(
            0, 0,
        );
        let clock_enable_input = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::CLOCK_ENABLE);
        connect_gates(
            self.four_cycle_clock_clke_splitter.clone(),
            cycle_block_output,
            self.control_section.clone(),
            clock_enable_input,
        );

        let output_index = self.get_index_from_tag(VariableBitCPU::CLKE);
        let cycle_block_output = self.four_cycle_clock_clke_splitter.lock().unwrap().get_index_for_output(
            0, 1,
        );
        connect_gates(
            self.four_cycle_clock_clke_splitter.clone(),
            cycle_block_output,
            output_gates[output_index].clone(),
            0,
        );
    }

    fn connect_four_cycle_clock_clks_splitter(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let cycle_block_output = self.four_cycle_clock_clks_splitter.lock().unwrap().get_index_for_output(
            0, 0,
        );
        let clock_set_input = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::CLOCK_SET);
        connect_gates(
            self.four_cycle_clock_clks_splitter.clone(),
            cycle_block_output,
            self.control_section.clone(),
            clock_set_input,
        );

        let output_index = self.get_index_from_tag(VariableBitCPU::CLKS);
        let cycle_block_output = self.four_cycle_clock_clks_splitter.lock().unwrap().get_index_for_output(
            0, 1,
        );
        connect_gates(
            self.four_cycle_clock_clks_splitter.clone(),
            cycle_block_output,
            output_gates[output_index].clone(),
            0,
        );
    }

    fn connect_bus(
        &mut self,
        bus_size: usize,
        ram_cells_decoder_input: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        //This is here to help with the reset. In case the reset goes high and sets all the pins,
        // need to make sure NONE is not passed into any of the inputs.
        self.bus.lock().unwrap().pull_output(LOW_);

        for i in 0..bus_size {
            let input_tag = format!("i_{}", i);

            //reg_0
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 0);
            let input_index = self.register_0.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.register_0.clone(),
                input_index,
            );

            //reg_1
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 1);
            let input_index = self.register_1.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.register_1.clone(),
                input_index,
            );

            //reg_2
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 2);
            let input_index = self.register_2.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.register_2.clone(),
                input_index,
            );

            //reg_3
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 3);
            let input_index = self.register_3.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.register_3.clone(),
                input_index,
            );

            //memory address register
            if i < ram_cells_decoder_input * 2 {
                let address_input_tag = format!("addr_{}", i);
                let output_index = self.bus.lock().unwrap().get_index_for_output(i, 4);
                let input_index = self.ram.lock().unwrap().get_index_from_tag(address_input_tag.as_str());
                connect_gates(
                    self.bus.clone(),
                    output_index,
                    self.ram.clone(),
                    input_index,
                );
            }

            //ram input (multiplexer)
            let multiplexer_input_tag = format!("I_0_bit_{}", i);
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 5);
            let input_index = self.load_multiplexer.lock().unwrap().get_index_from_tag(multiplexer_input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.load_multiplexer.clone(),
                input_index,
            );

            //ir
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 6);
            let input_index = self.instruction_register.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.instruction_register.clone(),
                input_index,
            );

            //iar
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 7);
            let input_index = self.instruction_address_register.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.instruction_address_register.clone(),
                input_index,
            );

            //tmp
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 8);
            let input_index = self.tmp.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.tmp.clone(),
                input_index,
            );

            //alu a
            let a_input_tag = format!("a_{}", i);
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 9);
            let input_index = self.alu.lock().unwrap().get_index_from_tag(a_input_tag.as_str());
            connect_gates(
                self.bus.clone(),
                output_index,
                self.alu.clone(),
                input_index,
            );

            let input_tag = format!("{}_{}", Self::BUS, i);
            let output_gate_index = self.get_index_from_tag(input_tag.as_str());
            let output_index = self.bus.lock().unwrap().get_index_for_output(i, 10);
            connect_gates(
                self.bus.clone(),
                output_index,
                output_gates[output_gate_index].clone(),
                0,
            );
        }
    }

    fn connect_register_0(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_0.clone(),
            self.bus.clone(),
            "i",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.register_0.clone(),
            Self::R0,
            output_gates,
        );
    }

    fn connect_register_1(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_1.clone(),
            self.bus.clone(),
            "i",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.register_1.clone(),
            Self::R1,
            output_gates,
        );
    }

    fn connect_register_2(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_2.clone(),
            self.bus.clone(),
            "i",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.register_2.clone(),
            Self::R2,
            output_gates,
        );
    }

    fn connect_register_3(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_3.clone(),
            self.bus.clone(),
            "i",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.register_3.clone(),
            Self::R3,
            output_gates,
        );
    }

    fn connect_instruction_address_register(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.instruction_address_register.clone(),
            self.bus.clone(),
            "i",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.instruction_address_register.clone(),
            Self::IAR,
            output_gates,
        );
    }

    fn connect_instruction_register(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.instruction_register.clone(),
            self.control_section.clone(),
            "IR",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.instruction_register.clone(),
            Self::IR,
            output_gates,
        );
    }

    fn connect_ram(
        &mut self,
        bus_size: usize,
        num_ram_cells: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.ram.clone(),
            self.bus.clone(),
            "i",
        );

        for i in 0..num_ram_cells {
            for j in 0..bus_size {
                let output_tag = RAMUnit::get_ram_output_string(i, j);
                let output_index = self.get_index_from_tag(output_tag.as_str());
                let ram_output_index = self.ram.lock().unwrap().get_index_from_tag(output_tag.as_str());
                connect_gates(
                    self.ram.clone(),
                    ram_output_index,
                    output_gates[output_index].clone(),
                    0,
                );
            }
        }
    }

    fn connect_alu(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.alu.clone(),
            self.acc.clone(),
            "i",
        );

        let input_index = self.flags.lock().unwrap().get_index_from_tag("i_0");
        let output_index = self.alu.lock().unwrap().get_index_from_tag("C_OUT");
        connect_gates(
            self.alu.clone(),
            output_index,
            self.flags.clone(),
            input_index,
        );

        let input_index = self.flags.lock().unwrap().get_index_from_tag("i_1");
        let output_index = self.alu.lock().unwrap().get_index_from_tag("A_L");
        connect_gates(
            self.alu.clone(),
            output_index,
            self.flags.clone(),
            input_index,
        );

        let input_index = self.flags.lock().unwrap().get_index_from_tag("i_2");
        let output_index = self.alu.lock().unwrap().get_index_from_tag("EQ");
        connect_gates(
            self.alu.clone(),
            output_index,
            self.flags.clone(),
            input_index,
        );

        let input_index = self.flags.lock().unwrap().get_index_from_tag("i_3");
        let output_index = self.alu.lock().unwrap().get_index_from_tag("Z");
        connect_gates(
            self.alu.clone(),
            output_index,
            self.flags.clone(),
            input_index,
        );
    }

    fn connect_bus_1(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.bus_1.clone(),
            self.alu.clone(),
            "b",
        );
    }

    fn connect_tmp(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.tmp.clone(),
            self.bus_1.clone(),
            "i",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.tmp.clone(),
            Self::TMP,
            output_gates,
        );
    }

    fn connect_c_tmp(&mut self) {
        let output_index = self.c_tmp.lock().unwrap().get_index_from_tag("Q");
        connect_gates(
            self.c_tmp.clone(),
            output_index,
            self.c_tmp_and.clone(),
            1,
        );
    }

    fn connect_c_tmp_and(&mut self) {
        let input_index = self.alu.lock().unwrap().get_index_from_tag("C_IN");
        connect_gates(
            self.c_tmp_and.clone(),
            0,
            self.alu.clone(),
            input_index,
        );
    }

    fn connect_acc(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.acc.clone(),
            self.bus.clone(),
            "i",
        );

        self.connect_multi_bit_output(
            bus_size,
            self.acc.clone(),
            Self::ACC,
            output_gates,
        );
    }

    fn connect_flags(&mut self) {
        let output_index = self.flags.lock().unwrap().get_index_from_tag("o_0");
        connect_gates(
            self.flags.clone(),
            output_index,
            self.flags_c_out_splitter.clone(),
            0,
        );

        let input_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::C_IN);
        let output_index = self.flags_c_out_splitter.lock().unwrap().get_index_for_output(0, 0);
        connect_gates(
            self.flags_c_out_splitter.clone(),
            output_index,
            self.control_section.clone(),
            input_index,
        );

        let input_index = self.c_tmp.lock().unwrap().get_index_from_tag("E");
        let output_index = self.flags_c_out_splitter.lock().unwrap().get_index_for_output(0, 1);
        connect_gates(
            self.flags_c_out_splitter.clone(),
            output_index,
            self.c_tmp.clone(),
            input_index,
        );

        let input_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::A_L);
        let output_index = self.flags.lock().unwrap().get_index_from_tag("o_1");
        connect_gates(
            self.flags.clone(),
            output_index,
            self.control_section.clone(),
            input_index,
        );

        let input_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::EQ);
        let output_index = self.flags.lock().unwrap().get_index_from_tag("o_2");
        connect_gates(
            self.flags.clone(),
            output_index,
            self.control_section.clone(),
            input_index,
        );

        let input_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::Z);
        let output_index = self.flags.lock().unwrap().get_index_from_tag("o_3");
        connect_gates(
            self.flags.clone(),
            output_index,
            self.control_section.clone(),
            input_index,
        );
    }

    fn connect_end_input_and_gate(&mut self) {
        let clk_input_index = self.four_cycle_clock_hookup.lock().unwrap().get_index_from_tag(FourCycleClockHookup::CLK_IN);
        connect_gates(
            self.end_input_and_gate.clone(),
            0,
            self.four_cycle_clock_hookup.clone(),
            clk_input_index,
        );
    }

    fn connect_end_input_not_gate(&mut self) {
        connect_gates(
            self.end_input_not_gate.clone(),
            0,
            self.end_input_and_gate.clone(),
            1,
        );
    }

    fn connect_load_multiplexer(
        &mut self,
        bus_size: usize,
    ) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.load_multiplexer.clone(),
            self.ram.clone(),
            "i",
        );
    }

    fn connect_load_counter(
        &mut self,
        ram_cells_decoder_input: usize,
    ) {
        VariableBitCPU::connect_input_to_output(
            ram_cells_decoder_input * 2,
            self.load_counter.clone(),
            self.counter_controlled_buffer.clone(),
            "i",
        );
    }

    fn connect_counter_controlled_buffer(
        &mut self,
        ram_cells_decoder_input: usize,
    ) {
        VariableBitCPU::connect_input_to_output(
            ram_cells_decoder_input * 2,
            self.counter_controlled_buffer.clone(),
            self.bus.clone(),
            "i",
        );
    }

    fn connect_counter_and(&mut self) {
        let clock_input = self.load_counter.lock().unwrap().get_index_from_tag(VariableBitCounter::CLK_IN);
        connect_gates(
            self.counter_and.clone(),
            0,
            self.load_counter.clone(),
            clock_input,
        );
    }

    fn connect_load_input_splitter(&mut self) {
        let splitter_output_index = self.load_input_splitter.lock().unwrap().get_index_for_output(
            0, 0,
        );
        let load_input_index = self.control_section.lock().unwrap().get_index_from_tag(ControlSection::HIGH_LVL_LOAD);
        connect_gates(
            self.load_input_splitter.clone(),
            splitter_output_index,
            self.control_section.clone(),
            load_input_index,
        );

        let splitter_output_index = self.load_input_splitter.lock().unwrap().get_index_for_output(
            0, 1,
        );
        let enable_index = self.counter_controlled_buffer.lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.load_input_splitter.clone(),
            splitter_output_index,
            self.counter_controlled_buffer.clone(),
            enable_index,
        );

        let splitter_output_index = self.load_input_splitter.lock().unwrap().get_index_for_output(
            0, 2,
        );
        connect_gates(
            self.load_input_splitter.clone(),
            splitter_output_index,
            self.counter_and.clone(),
            0,
        );

        let splitter_output_index = self.load_input_splitter.lock().unwrap().get_index_for_output(
            0, 3,
        );
        let multiplexed_control_index = self.load_multiplexer.lock().unwrap().get_index_from_tag("C_0");
        connect_gates(
            self.load_input_splitter.clone(),
            splitter_output_index,
            self.load_multiplexer.clone(),
            multiplexed_control_index,
        );
    }

    fn connect_reset_controlled_buffer(&mut self, bus_size: usize) {
        for i in 0..bus_size {
            let input_tag = format!("i_{}", i);
            let input_index = self.reset_controlled_buffer.lock().unwrap().get_index_from_tag(input_tag.as_str());
            self.reset_controlled_buffer.lock().unwrap().update_input_signal(
                GateInput::new(
                    input_index,
                    LOW_,
                    UniqueID::zero_id(),
                )
            );
        }

        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.reset_controlled_buffer.clone(),
            self.bus.clone(),
            "i",
        );
    }

    pub fn get_clock_synced_with_cpu(
        &self,
        clock: &SharedMutex<Clock>,
    ) {
        let input_index = self.get_index_from_tag(VariableBitCPU::CLK_IN);
        let output_signals = self.complex_gate.input_gates[input_index].lock().unwrap().fetch_output_signals().unwrap();
        //SimpleInput outputs all have the same value.
        let output_signal =
            match output_signals.first().unwrap() {
                GateOutputState::NotConnected(_) => panic!("SimpleInput for VariableBitCPU CLK_IN should always be connected."),
                GateOutputState::Connected(connected_output) => {
                    connected_output.throughput.signal.clone()
                }
            };
        clock.lock().unwrap().set_clock_state(output_signal);
    }

    pub fn get_complex_gate(&self) -> &ComplexGateMembers {
        &self.complex_gate
    }
}

impl LogicGate for VariableBitCPU {
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
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use rand::Rng;
    use crate::logic::foundations::{connect_gates, LogicGate, Signal};
    use crate::logic::foundations::Signal::{HIGH, LOW_};
    use crate::logic::input_gates::{AutomaticInput};
    use crate::logic::processor_components::RAMUnit;
    use crate::logic::variable_bit_cpu::{ALUInstruction, Instructions, Register, VariableBitCPU};
    use crate::run_circuit::{collect_signals_from_logic_gate, compare_generate_and_collected_output, generate_default_output, load_values_into_ram, run_circuit, run_instructions};
    use crate::shared_mutex::SharedMutex;
    use crate::test_stuff::{run_test_with_timeout};

    fn store_in_output(
        cpu: &SharedMutex<VariableBitCPU>,
        default_output: &mut Vec<Signal>,
        i: usize,
        signal: Signal,
        tag: &str,
    ) {
        let acc_tag = format!("{}_{}", tag, i);

        let acc_index = cpu.lock().unwrap().get_index_from_tag(acc_tag.as_str());

        default_output[acc_index] = signal.clone();
    }

    fn convert_bytes_to_signals<F>(byte_string: &str, mut task: F)
        where
            F: FnMut(usize, Signal),
    {
        let stored_data_bytes = byte_string.as_bytes().to_vec();
        for (i, b) in stored_data_bytes.iter().rev().enumerate() {
            let signal =
                if *b == b'0' {
                    LOW_
                } else {
                    HIGH
                };

            task(i, signal);
        }
    }

    fn generate_ram_output(
        cpu: &SharedMutex<VariableBitCPU>,
        binary_strings: &Vec<&str>,
        output: &mut Vec<Signal>,
    ) {
        for (i, str) in binary_strings.iter().enumerate() {
            convert_bytes_to_signals(
                str,
                |j, signal| {
                    let mut ram_cell_string = RAMUnit::get_ram_output_string(i, 0);
                    ram_cell_string.pop();
                    ram_cell_string.pop();
                    store_in_output(
                        cpu,
                        output,
                        j,
                        signal,
                        ram_cell_string.as_str(),
                    )
                },
            );
        }
    }

    fn generate_end_output(
        cpu: &SharedMutex<VariableBitCPU>,
        number_bits: usize,
        end_instruction_ram_cell_index: usize,
        output: &mut Vec<Signal>,
    ) {
        convert_bytes_to_signals(
            Instructions::binary(Instructions::End).as_str(),
            |i, signal| {
                store_in_output(
                    cpu,
                    output,
                    i,
                    signal.clone(),
                    VariableBitCPU::IR,
                );

                store_in_output(
                    cpu,
                    output,
                    i,
                    signal,
                    VariableBitCPU::BUS,
                );
            },
        );

        convert_bytes_to_signals(
            format!("{:0width$b}", end_instruction_ram_cell_index, width = number_bits).as_str(),
            |i, signal| {
                store_in_output(
                    cpu,
                    output,
                    i,
                    signal,
                    VariableBitCPU::IAR,
                );
            },
        );

        let mut acc_number_binary_string = format!("{:0width$b}", end_instruction_ram_cell_index + 1, width = number_bits);
        if acc_number_binary_string.len() > number_bits {
            acc_number_binary_string.remove(0);
        }

        convert_bytes_to_signals(
            acc_number_binary_string.as_str(),
            |i, signal| {
                store_in_output(
                    cpu,
                    output,
                    i,
                    signal,
                    VariableBitCPU::ACC,
                );
            },
        );

        let clk_out_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::CLK_OUT);
        let clks_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::CLKS);
        let io_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::IO);
        let da_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::DA);
        let end_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::END);

        output[clk_out_index] = HIGH;
        output[clks_index] = HIGH;
        output[io_index] = HIGH;
        output[da_index] = HIGH;
        output[end_index] = HIGH;
    }

    fn generate_basic_output(
        cpu: &SharedMutex<VariableBitCPU>,
        number_bits: usize,
        binary_strings: &Vec<&str>,
        end_instruction_ram_cell_index: usize,
    ) -> Vec<Signal> {
        let mut default_output = generate_default_output(cpu);

        generate_ram_output(
            cpu,
            binary_strings,
            &mut default_output,
        );

        generate_end_output(
            cpu,
            number_bits,
            end_instruction_ram_cell_index,
            &mut default_output,
        );

        default_output
    }

    fn reset_cpu_values(cpu: &SharedMutex<VariableBitCPU>) {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        let reset_input = AutomaticInput::new(
            vec![HIGH],
            1,
            "RESET",
        );

        let reset_cpu_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::RESET);
        connect_gates(
            reset_input.clone(),
            0,
            cpu.clone(),
            reset_cpu_index,
        );

        input_gates.push(reset_input.clone());

        cpu.lock().unwrap().bus.lock().unwrap().toggle_output_printing(true);

        run_circuit(
            &input_gates,
            &Vec::new(),
            false,
            &mut |_clock_tick_inputs, _output_gates| {},
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let generated_output = generate_default_output(&cpu);

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_output,
            collected_signals,
        );

        assert!(!failed);

        reset_input.lock().unwrap().disconnect_gate(0);
    }

    fn run_alu_instruction(
        number_bits: usize,
        a_num: usize,
        b_num: usize,
        result: usize,
        opt: ALUInstruction,
        reg_a: Register,
        reg_b: Register,
    ) {
        let decoder_input_size = 2;

        let data_a_num = Instructions::binary(
            Instructions::Data { reg: reg_a.clone() }
        );
        let a_num_data = format!("{:0width$b}", a_num, width = number_bits);
        let data_b_num = Instructions::binary(
            Instructions::Data { reg: reg_b.clone() }
        );
        let b_num_data = format!("{:0width$b}", b_num, width = number_bits);
        let shift_right_instruction = Instructions::binary(
            Instructions::ALU {
                opt,
                reg_a: reg_a.clone(),
                reg_b: reg_b.clone(),
            }
        );
        let end_instruction = Instructions::binary(Instructions::End);

        let binary_strings = vec![
            data_a_num.as_str(),
            a_num_data.as_str(),
            data_b_num.as_str(),
            b_num_data.as_str(),
            shift_right_instruction.as_str(),
            end_instruction.as_str(),
        ];

        let end_instruction_index = binary_strings.len() - 1;
        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let mut generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        let first_tag = reg_a.get_variable_bit_tag();

        convert_bytes_to_signals(
            a_num_data.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    first_tag,
                );
            },
        );

        convert_bytes_to_signals(
            b_num_data.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::TMP,
                );
            },
        );

        let second_tag = reg_b.get_variable_bit_tag();
        let mut result_string = format!("{:0width$b}", result, width = number_bits);
        //If result is larger, chop off leading digits.
        while result_string.len() > number_bits {
            result_string.remove(0);
        }
        convert_bytes_to_signals(
            result_string.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal.clone(),
                    second_tag,
                );
            },
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);
    }

    fn run_jump_if_test(
        number_bits: usize,
        num_a: i32,
        num_b: i32,
        result: i32,
        carry: bool,
        a_larger: bool,
        equal: bool,
        zero: bool,
        opt: ALUInstruction,
        successful_jump: bool,
    ) {
        let decoder_input_size = 2;

        let jump_to_address_num = 9;

        let store_data_a_instruction = Instructions::binary(
            Instructions::Data {
                reg: Register::R0
            }
        );
        let num_a_data = format!("{:0width$b}", num_a, width = number_bits);
        let store_data_b_instruction = Instructions::binary(
            Instructions::Data {
                reg: Register::R1
            }
        );
        let add_instruction = Instructions::binary(
            Instructions::ALU {
                opt,
                reg_a: Register::R0,
                reg_b: Register::R1,
            }
        );
        let num_b_data = format!("{:0width$b}", num_b, width = number_bits);
        let jump_if_carry_instruction = Instructions::binary(
            Instructions::JumpIf {
                carry,
                a_larger,
                equal,
                zero,
            }
        );
        let jump_to_address_data = format!("{:0width$b}", jump_to_address_num, width = number_bits);
        let end_instruction = Instructions::binary(Instructions::End);

        let binary_strings = vec![
            store_data_a_instruction.as_str(), //0
            num_a_data.as_str(), //1
            store_data_b_instruction.as_str(), //2
            num_b_data.as_str(), //3
            add_instruction.as_str(), //4
            jump_if_carry_instruction.as_str(), //5
            jump_to_address_data.as_str(), //6
            end_instruction.as_str(), //7
            "00000000", //8 Dummy data
            end_instruction.as_str(), //9
        ];

        let end_instruction_index =
            if successful_jump {
                binary_strings.len() - 1
            } else {
                binary_strings.len() - 3
            };

        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let mut generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        convert_bytes_to_signals(
            num_a_data.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::R0,
                );
            },
        );

        convert_bytes_to_signals(
            num_b_data.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::TMP,
                );
            },
        );

        let mut sum_string = format!("{:0width$b}", result, width = number_bits);
        //If result is larger, chop off leading digits.
        while sum_string.len() > number_bits {
            sum_string.remove(0);
        }
        convert_bytes_to_signals(
            sum_string.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal.clone(),
                    VariableBitCPU::R1,
                );
            },
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);
    }

    #[test]
    fn initialization() {
        let cpu = VariableBitCPU::new(8, 4);

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());

        let generated_signals = generate_default_output(&cpu);

        let failed = compare_generate_and_collected_output(&cpu, generated_signals, collected_signals);

        assert!(!failed);
    }

    #[test]
    fn load_to_ram() {
        let number_bits = 8;
        let decoder_input_size = 2;
        let cpu = VariableBitCPU::new(number_bits, decoder_input_size);

        let binary_strings = vec![
            "11110000",
            "00110011",
            "01010110",
            "01110111",
            "01000011",
            "10011001",
        ];

        let num_ram_cells = usize::pow(2, (decoder_input_size * 2) as u32);
        assert!(binary_strings.len() <= num_ram_cells);
        if !binary_strings.is_empty() {
            assert_eq!(binary_strings[0].len(), number_bits);
        }

        load_values_into_ram(
            &cpu,
            &binary_strings,
            num_ram_cells,
        );
    }

    #[test]
    fn reset_cpu() {
        let number_bits = 8;
        let decoder_input_size = 1;
        let cpu = VariableBitCPU::new(number_bits, decoder_input_size);

        let binary_strings = vec![
            "11111111",
        ];

        let num_ram_cells = usize::pow(2, (decoder_input_size * 2) as u32);
        assert!(binary_strings.len() <= num_ram_cells);
        if !binary_strings.is_empty() {
            assert_eq!(binary_strings[0].len(), number_bits);
        }

        load_values_into_ram(
            &cpu,
            &binary_strings,
            num_ram_cells,
        );

        reset_cpu_values(&cpu);
    }

    #[test]
    fn end_instruction() {
        run_test_with_timeout(
            Duration::from_millis(500),
            || {
                let number_bits = 8;
                let decoder_input_size = 1;

                let end_instruction = Instructions::binary(Instructions::End);
                let binary_strings = vec![
                    end_instruction.as_str(),
                ];

                let end_instruction_index = binary_strings.len() - 1;
                let cpu = run_instructions(
                    number_bits,
                    decoder_input_size,
                    &binary_strings,
                );

                let collected_signals = collect_signals_from_logic_gate(cpu.clone());
                // let mut generated_signals = generate_default_output(&cpu);

                let generated_signals = generate_basic_output(
                    &cpu,
                    number_bits,
                    &binary_strings,
                    end_instruction_index,
                );

                let failed = compare_generate_and_collected_output(
                    &cpu,
                    generated_signals,
                    collected_signals,
                );

                assert!(!failed);
            },
        )
    }

    #[test]
    fn data_instruction() {
        let number_bits = 8;
        let decoder_input_size = 1;

        let data_instruction = Instructions::binary(
            Instructions::Data { reg: Register::R1 }
        );
        let stored_data = "11111010";
        let end_instruction = Instructions::binary(Instructions::End);

        let binary_strings = vec![
            data_instruction.as_str(),
            stored_data,
            end_instruction.as_str(),
        ];

        let end_instruction_index = binary_strings.len() - 1;
        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());

        let mut generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        convert_bytes_to_signals(
            stored_data,
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::R1,
                );
            },
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);
    }

    #[test]
    fn store_instruction() {
        let number_bits = 8;
        let decoder_input_size = 2;

        let data_instruction_first = Instructions::binary(
            Instructions::Data { reg: Register::R0 }
        );
        let stored_data_address = "00000110"; // 6
        let data_instruction_second = Instructions::binary(
            Instructions::Data { reg: Register::R3 }
        );
        let stored_data_value = "11111010";
        let store_instruction = Instructions::binary(
            Instructions::Store { reg_a: Register::R0, reg_b: Register::R3 }
        );
        let end_instruction = Instructions::binary(Instructions::End);

        //This should store the stored_data_value to memory address 6 (stored_data_address).
        let binary_strings = vec![
            data_instruction_first.as_str(), //0
            stored_data_address, //1
            data_instruction_second.as_str(), //2
            stored_data_value, //3
            store_instruction.as_str(), //4
            end_instruction.as_str(), //5
        ];

        let end_instruction_index = binary_strings.len() - 1;
        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let mut generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        convert_bytes_to_signals(
            stored_data_address,
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::R0,
                );
            },
        );

        convert_bytes_to_signals(
            stored_data_value,
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal.clone(),
                    VariableBitCPU::R3,
                );

                let mut output_tag = RAMUnit::get_ram_output_string(6, 0);
                output_tag.pop();
                output_tag.pop();
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    output_tag.as_str(),
                );
            },
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);
    }

    #[test]
    fn load_instruction() {
        let number_bits = 8;
        let decoder_input_size = 2;

        let data_instruction_first = Instructions::binary(
            Instructions::Data { reg: Register::R1 }
        );
        let stored_data_address = "00000100"; // 4
        let load_instruction = Instructions::binary(
            Instructions::Load { reg_a: Register::R1, reg_b: Register::R2 }
        );
        let end_instruction = Instructions::binary(Instructions::End);
        let stored_data_value = "10111010";

        //This should store the stored_data_value to memory address 6 (stored_data_address).
        let binary_strings = vec![
            data_instruction_first.as_str(), //0
            stored_data_address, //1
            load_instruction.as_str(), //2
            end_instruction.as_str(), //3
            stored_data_value, //4
        ];

        let end_instruction_index = binary_strings.len() - 2;
        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let mut generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        convert_bytes_to_signals(
            stored_data_address,
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::R1,
                );
            },
        );

        convert_bytes_to_signals(
            stored_data_value,
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal.clone(),
                    VariableBitCPU::R2,
                );

                let mut output_tag = RAMUnit::get_ram_output_string(4, 0);
                output_tag.pop();
                output_tag.pop();
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    output_tag.as_str(),
                );
            },
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);
    }

    #[test]
    fn add_instruction() {
        let number_bits = 8;

        let high_number_range = usize::pow(2, number_bits as u32);
        let a_num = rand::thread_rng().gen_range(0..high_number_range);
        let b_num = rand::thread_rng().gen_range(0..high_number_range);

        let sum = a_num + b_num;

        run_alu_instruction(
            number_bits,
            a_num,
            b_num,
            sum,
            ALUInstruction::ADD,
            Register::R0,
            Register::R1,
        );
    }

    #[test]
    fn shift_right_instruction() {
        let number_bits = 8;

        let high_number_range = usize::pow(2, number_bits as u32);
        let a_num = rand::thread_rng().gen_range(0..high_number_range);

        let shift_result = a_num >> 1;

        run_alu_instruction(
            number_bits,
            a_num,
            0,
            shift_result,
            ALUInstruction::SHR,
            Register::R0,
            Register::R1,
        );
    }

    #[test]
    fn shift_left_instruction() {
        let number_bits = 8;

        let high_number_range = usize::pow(2, number_bits as u32);
        let a_num = rand::thread_rng().gen_range(0..high_number_range);

        let shift_result = a_num << 1;

        run_alu_instruction(
            number_bits,
            a_num,
            0,
            shift_result,
            ALUInstruction::SHL,
            Register::R0,
            Register::R1,
        );
    }

    #[test]
    fn not_instruction() {
        let number_bits = 8;

        let high_number_range = usize::pow(2, number_bits as u32);
        let a_num = rand::thread_rng().gen_range(0..high_number_range);

        let not_result = !a_num;

        run_alu_instruction(
            number_bits,
            a_num,
            0,
            not_result,
            ALUInstruction::NOT,
            Register::R0,
            Register::R1,
        );
    }

    #[test]
    fn and_instruction() {
        let number_bits = 8;

        let high_number_range = usize::pow(2, number_bits as u32);
        let a_num = rand::thread_rng().gen_range(0..high_number_range);
        let b_num = rand::thread_rng().gen_range(0..high_number_range);

        let not_result = a_num & b_num;

        run_alu_instruction(
            number_bits,
            a_num,
            b_num,
            not_result,
            ALUInstruction::AND,
            Register::R0,
            Register::R1,
        );
    }

    #[test]
    fn or_instruction() {
        let number_bits = 8;

        let high_number_range = usize::pow(2, number_bits as u32);
        let a_num = rand::thread_rng().gen_range(0..high_number_range);
        let b_num = rand::thread_rng().gen_range(0..high_number_range);

        let not_result = a_num | b_num;

        run_alu_instruction(
            number_bits,
            a_num,
            b_num,
            not_result,
            ALUInstruction::OR,
            Register::R0,
            Register::R1,
        );
    }

    #[test]
    fn xor_instruction() {
        let number_bits = 8;

        let high_number_range = usize::pow(2, number_bits as u32);
        let a_num = rand::thread_rng().gen_range(0..high_number_range);
        let b_num = rand::thread_rng().gen_range(0..high_number_range);

        let not_result = a_num ^ b_num;

        run_alu_instruction(
            number_bits,
            a_num,
            b_num,
            not_result,
            ALUInstruction::XOR,
            Register::R0,
            Register::R1,
        );
    }

    #[test]
    fn jump_register_instruction() {
        let number_bits = 8;
        let decoder_input_size = 2;

        //Jump past the end at address 3 and use the jump at address 5. If the jump fails and it
        // ends early, the values in IAR and ACC will be wrong and the test will fail.
        let address_to_jump_to_num = 5;

        let data_a_num = Instructions::binary(
            Instructions::Data { reg: Register::R0 }
        );
        let address_to_jump_to_data = format!("{:0width$b}", address_to_jump_to_num, width = number_bits);
        let jump_register_instruction = Instructions::binary(
            Instructions::JumpRegister {
                reg: Register::R0,
            }
        );
        let end_instruction = Instructions::binary(Instructions::End);

        let binary_strings = vec![
            data_a_num.as_str(), //0
            address_to_jump_to_data.as_str(), //1
            jump_register_instruction.as_str(), //2
            end_instruction.as_str(), //3
            "00000000", //dummy data 4
            end_instruction.as_str(), //5
        ];

        let end_instruction_index = binary_strings.len() - 1;
        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let mut generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        convert_bytes_to_signals(
            address_to_jump_to_data.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::R0,
                );
            },
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);
    }

    #[test]
    fn jump_address_instruction() {
        let number_bits = 8;
        let decoder_input_size = 2;

        //Jump past the end at address 3 and use the jump at address 5. If the jump fails and it
        // ends early, the values in IAR and ACC will be wrong and the test will fail.
        let address_to_jump_to_num = 4;

        let jump_address_instruction = Instructions::binary(
            Instructions::JumpAddress
        );
        let address_to_jump_to_data = format!("{:0width$b}", address_to_jump_to_num, width = number_bits);
        let end_instruction = Instructions::binary(Instructions::End);

        let binary_strings = vec![
            jump_address_instruction.as_str(), //0
            address_to_jump_to_data.as_str(), //1
            end_instruction.as_str(), //2
            "00000000", //dummy data 3
            end_instruction.as_str(), //4
        ];

        let end_instruction_index = binary_strings.len() - 1;
        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);
    }

    #[test]
    fn jump_if_carry_instruction() {
        let number_bits = 8;

        //Want to make sure 3/4 of the flags are false.
        let num_a = 254;
        let num_b = 255;
        let sum = num_a + num_b;

        run_jump_if_test(
            number_bits,
            num_a,
            num_b,
            sum,
            true,
            false,
            false,
            false,
            ALUInstruction::ADD,
            true,
        );
    }

    #[test]
    fn jump_if_a_larger_instruction() {
        let number_bits = 8;

        //Want to make sure 3/4 of the flags are false.
        let num_a = 2;
        let num_b = 1;
        let sum = num_a + num_b;

        run_jump_if_test(
            number_bits,
            num_a,
            num_b,
            sum,
            false,
            true,
            false,
            false,
            ALUInstruction::ADD,
            true,
        );
    }

    #[test]
    fn jump_if_equal_instruction() {
        let number_bits = 8;

        //Want to make sure 3/4 of the flags are false.
        let num_a = 1;
        let num_b = 1;
        let sum = num_a + num_b;

        run_jump_if_test(
            number_bits,
            num_a,
            num_b,
            sum,
            false,
            false,
            true,
            false,
            ALUInstruction::ADD,
            true,
        );
    }

    #[test]
    fn jump_if_zero_instruction() {
        let number_bits = 8;

        //Want to make sure 3/4 of the flags are false.
        let num_a = 15;
        let num_b = 0;
        let result = num_a & num_b;

        run_jump_if_test(
            number_bits,
            num_a,
            num_b,
            result,
            false,
            false,
            false,
            true,
            ALUInstruction::AND,
            true,
        );
    }

    #[test]
    fn jump_if_none_instruction() {
        let number_bits = 8;

        //Want to make sure 3/4 of the flags are false.
        let num_a = 254;
        let num_b = 255;
        let result = num_a | num_b;

        run_jump_if_test(
            number_bits,
            num_a,
            num_b,
            result,
            false,
            false,
            false,
            false,
            ALUInstruction::OR,
            false,
        );
    }

    #[test]
    fn clear_flags_instruction() {
        let number_bits = 8;

        //Force a carry bit.
        let num_a = 255;
        let result = num_a << 1;

        let decoder_input_size = 2;

        let store_data_a_instruction = Instructions::binary(
            Instructions::Data {
                reg: Register::R0
            }
        );
        let num_a_data = format!("{:0width$b}", num_a, width = number_bits);
        let add_instruction = Instructions::binary(
            Instructions::ALU {
                opt: ALUInstruction::SHL,
                reg_a: Register::R0,
                reg_b: Register::R1,
            }
        );
        let clear_flags = Instructions::binary(
            Instructions::ClearFlags
        );
        let end_instruction = Instructions::binary(Instructions::End);

        let binary_strings = vec![
            store_data_a_instruction.as_str(),
            num_a_data.as_str(),
            add_instruction.as_str(),
            clear_flags.as_str(),
            end_instruction.as_str(),
        ];

        let end_instruction_index = binary_strings.len() - 1;
        let cpu = run_instructions(
            number_bits,
            decoder_input_size,
            &binary_strings,
        );

        let collected_signals = collect_signals_from_logic_gate(cpu.clone());
        let mut generated_signals = generate_basic_output(
            &cpu,
            number_bits,
            &binary_strings,
            end_instruction_index,
        );

        convert_bytes_to_signals(
            num_a_data.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal,
                    VariableBitCPU::R0,
                );
            },
        );

        let mut sum_string = format!("{:0width$b}", result, width = number_bits);
        //If result is larger, chop off leading digits.
        while sum_string.len() > number_bits {
            sum_string.remove(0);
        }
        convert_bytes_to_signals(
            sum_string.as_str(),
            |i, signal| {
                store_in_output(
                    &cpu,
                    &mut generated_signals,
                    i,
                    signal.clone(),
                    VariableBitCPU::R1,
                );
            },
        );

        let failed = compare_generate_and_collected_output(
            &cpu,
            generated_signals,
            collected_signals,
        );

        assert!(!failed);

        let collected_signals = collect_signals_from_logic_gate(
            cpu.lock().unwrap().flags.clone()
        );

        assert_eq!(
            collected_signals,
            vec![LOW_; 8],
        )
    }
}

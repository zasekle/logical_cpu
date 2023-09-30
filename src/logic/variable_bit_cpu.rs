use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::arithmetic_gates::ArithmeticLogicUnit;
use crate::logic::basic_gates::{And, ControlledBuffer, Not, Splitter};
use crate::logic::complex_logic::{FourCycleClockHookup, VariableBitCounter, VariableBitMultiplexer};
use crate::logic::control_section::ControlSection;

use crate::logic::foundations::{ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW_, HIGH};
use crate::logic::input_gates::{Clock, SimpleInput};
use crate::logic::memory_gates::{OneBitMemoryCell, VariableBitMemoryCell};
use crate::logic::processor_components::{RAMUnit, VariableBitBusOne, VariableBitRegister};

pub struct VariableBitCPU {
    complex_gate: ComplexGateMembers,
    four_cycle_clock_hookup: Rc<RefCell<FourCycleClockHookup>>,
    four_cycle_clock_clk_splitter: Rc<RefCell<Splitter>>,
    four_cycle_clock_clke_splitter: Rc<RefCell<Splitter>>,
    four_cycle_clock_clks_splitter: Rc<RefCell<Splitter>>,
    control_section: Rc<RefCell<ControlSection>>,
    temp_s_splitter: Rc<RefCell<Splitter>>,
    bus: Rc<RefCell<Splitter>>,
    register_0: Rc<RefCell<VariableBitRegister>>,
    register_1: Rc<RefCell<VariableBitRegister>>,
    register_2: Rc<RefCell<VariableBitRegister>>,
    register_3: Rc<RefCell<VariableBitRegister>>,
    instruction_address_register: Rc<RefCell<VariableBitRegister>>,
    instruction_register: Rc<RefCell<VariableBitMemoryCell>>,
    ram: Rc<RefCell<RAMUnit>>,
    alu: Rc<RefCell<ArithmeticLogicUnit>>,
    bus_1: Rc<RefCell<VariableBitBusOne>>,
    tmp: Rc<RefCell<VariableBitMemoryCell>>,
    c_tmp: Rc<RefCell<OneBitMemoryCell>>,
    c_tmp_and: Rc<RefCell<And>>,
    acc: Rc<RefCell<VariableBitRegister>>,
    flags: Rc<RefCell<VariableBitMemoryCell>>,
    flags_c_out_splitter: Rc<RefCell<Splitter>>,
    end_input_and_gate: Rc<RefCell<And>>,
    end_input_not_gate: Rc<RefCell<Not>>,
    load_multiplexer: Rc<RefCell<VariableBitMultiplexer>>,
    load_counter: Rc<RefCell<VariableBitCounter>>,
    counter_controlled_buffer: Rc<RefCell<ControlledBuffer>>,
    counter_and: Rc<RefCell<And>>,
    load_input_splitter: Rc<RefCell<Splitter>>,
    reset_controlled_buffer: Rc<RefCell<ControlledBuffer>>,
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

    //8 bit instructions.
    pub const END_INSTRUCTION: &'static str = "11001111";

    pub fn new(number_bits: usize, ram_cells_decoder_input: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        for i in 0..number_bits {
            let input_tag = format!("{}_{}", Self::RAM, i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));
        }

        input_gates.push(SimpleInput::new(1, VariableBitCPU::LOAD));
        input_gates.push(SimpleInput::new(3, VariableBitCPU::RESET));
        input_gates.push(SimpleInput::new(1, VariableBitCPU::MARS));
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

        cpu.four_cycle_clock_hookup.borrow_mut().set_tag("four_cycle_clock_hookup");
        cpu.four_cycle_clock_clk_splitter.borrow_mut().set_tag("four_cycle_clock_clk_splitter");
        cpu.four_cycle_clock_clke_splitter.borrow_mut().set_tag("four_cycle_clock_clke_splitter");
        cpu.four_cycle_clock_clks_splitter.borrow_mut().set_tag("four_cycle_clock_clks_splitter");
        cpu.control_section.borrow_mut().set_tag("control_section");
        cpu.temp_s_splitter.borrow_mut().set_tag("temp_s_splitter");
        cpu.bus.borrow_mut().set_tag("bus");
        cpu.register_0.borrow_mut().set_tag("register_0");
        cpu.register_1.borrow_mut().set_tag("register_1");
        cpu.register_2.borrow_mut().set_tag("register_2");
        cpu.register_3.borrow_mut().set_tag("register_3");
        cpu.instruction_address_register.borrow_mut().set_tag("instruction_address_register");
        cpu.instruction_register.borrow_mut().set_tag("instruction_register");
        cpu.ram.borrow_mut().set_tag("ram");
        cpu.alu.borrow_mut().set_tag("alu");
        cpu.bus_1.borrow_mut().set_tag("bus_1");
        cpu.tmp.borrow_mut().set_tag("tmp");
        cpu.c_tmp.borrow_mut().set_tag("c_tmp");
        cpu.c_tmp_and.borrow_mut().set_tag("c_tmp_and");
        cpu.acc.borrow_mut().set_tag("acc");
        cpu.flags.borrow_mut().set_tag("flags");
        cpu.flags_c_out_splitter.borrow_mut().set_tag("flags_c_out_splitter");
        cpu.end_input_and_gate.borrow_mut().set_tag("end_input_and_gate");
        cpu.end_input_not_gate.borrow_mut().set_tag("end_input_not_gate");
        cpu.load_multiplexer.borrow_mut().set_tag("load_multiplexer");
        cpu.load_counter.borrow_mut().set_tag("load_counter");
        cpu.counter_and.borrow_mut().set_tag("counter_and");
        cpu.counter_controlled_buffer.borrow_mut().set_tag("counter_controlled_buffer");
        cpu.load_input_splitter.borrow_mut().set_tag("load_input_splitter");
        cpu.reset_controlled_buffer.borrow_mut().set_tag("reset_controlled_buffer");

        cpu.build_and_prime_circuit(
            number_bits,
            ram_cells_decoder_input,
            num_ram_cells,
            output_gates_logic,
        );

        Rc::new(RefCell::new(cpu))
    }

    fn build_and_prime_circuit(
        &mut self,
        bus_size: usize,
        ram_cells_decoder_input: usize,
        num_ram_cells: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
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

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }

    fn connect_input_to_output(
        bus_size: usize,
        start_gate: Rc<RefCell<dyn LogicGate>>,
        end_gate: Rc<RefCell<dyn LogicGate>>,
        input_val: &str,
    ) {
        for i in 0..bus_size {
            let input_tag = format!("{}_{}", input_val, i);
            let output_tag = format!("o_{}", i);
            let input_index = end_gate.borrow_mut().get_index_from_tag(input_tag.as_str());
            let output_index = start_gate.borrow_mut().get_index_from_tag(output_tag.as_str());
            start_gate.borrow_mut().connect_output_to_next_gate(
                output_index,
                input_index,
                end_gate.clone(),
            );
        }
    }

    fn connect_multi_bit_output(
        &mut self,
        bus_size: usize,
        start_gate: Rc<RefCell<dyn LogicGate>>,
        input_val: &str,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..bus_size {
            let input_tag = format!("{}_{}", input_val, i);
            let output_tag = format!("reg_{}", i);
            let output_gate_index = self.get_index_from_tag(input_tag.as_str());
            let output_index = start_gate.borrow_mut().get_index_from_tag(output_tag.as_str());
            start_gate.borrow_mut().connect_output_to_next_gate(
                output_index,
                0,
                output_gates[output_gate_index].clone(),
            );
        }
    }

    fn connect_inputs(&mut self, bus_size: usize) {
        for i in 0..bus_size {
            let input_tag = format!("{}_{}", Self::RAM, i);
            let input_index = self.get_index_from_tag(input_tag.as_str());
            let input_gate = self.complex_gate.input_gates[input_index].clone();

            let multiplexer_tag = format!("I_1_bit_{}", i);
            let ram_input_index = self.load_multiplexer.borrow_mut().get_index_from_tag(multiplexer_tag.as_str());
            input_gate.borrow_mut().connect_output_to_next_gate(
                0,
                ram_input_index,
                self.load_multiplexer.clone(),
            );
        }

        let reset_index = self.get_index_from_tag(VariableBitCPU::RESET);
        let reset_input_gate = self.complex_gate.input_gates[reset_index].clone();

        let control_section_reset_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::HIGH_LVL_RESET);
        reset_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            control_section_reset_index,
            self.control_section.clone(),
        );

        let ram_reset = self.ram.borrow_mut().get_index_from_tag("R");
        reset_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            ram_reset,
            self.ram.clone(),
        );

        let controlled_buffer_enable = self.reset_controlled_buffer.borrow_mut().get_index_from_tag("E");
        reset_input_gate.borrow_mut().connect_output_to_next_gate(
            2,
            controlled_buffer_enable,
            self.reset_controlled_buffer.clone(),
        );

        let mars_index = self.get_index_from_tag(VariableBitCPU::MARS);
        let mars_input_gate = self.complex_gate.input_gates[mars_index].clone();

        let control_section_mars_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::HIGH_LVL_MARS);
        mars_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            control_section_mars_index,
            self.control_section.clone(),
        );

        let load_index = self.get_index_from_tag(VariableBitCPU::LOAD);
        let load_input_gate = self.complex_gate.input_gates[load_index].clone();

        load_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.load_input_splitter.clone(),
        );

        let clk_in_index = self.get_index_from_tag(VariableBitCPU::CLK_IN);
        let clk_in_input_gate = self.complex_gate.input_gates[clk_in_index].clone();

        clk_in_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.end_input_and_gate.clone(),
        );

        //Tie this high so that the priming will work sequentially.
        clk_in_input_gate.borrow_mut().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );
    }

    fn connect_control_section(
        &mut self,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let input_index = self.alu.borrow_mut().get_index_from_tag("C");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::ALU_0);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.alu.clone(),
        );

        let input_index = self.alu.borrow_mut().get_index_from_tag("B");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::ALU_1);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.alu.clone(),
        );

        let input_index = self.alu.borrow_mut().get_index_from_tag("C");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::ALU_2);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.alu.clone(),
        );

        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::C_OUT);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            self.c_tmp_and.clone(),
        );

        let input_index = self.flags.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::FLAG_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.flags.clone(),
        );

        let input_index = self.acc.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::ACC_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.acc.clone(),
        );

        let input_index = self.acc.borrow_mut().get_index_from_tag("E");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::ACC_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.acc.clone(),
        );

        let input_index = self.instruction_address_register.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::ACC_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.instruction_address_register.clone(),
        );

        let input_index = self.instruction_address_register.borrow_mut().get_index_from_tag("E");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::ACC_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.instruction_address_register.clone(),
        );

        let input_index = self.instruction_register.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::IR_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.instruction_register.clone(),
        );

        let input_index = self.ram.borrow_mut().get_index_from_tag("E");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::RAM_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.ram.clone(),
        );

        let input_index = self.ram.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::RAM_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.ram.clone(),
        );

        let input_index = self.ram.borrow_mut().get_index_from_tag("SA");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::MAR_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.ram.clone(),
        );

        let input_index = self.register_0.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R0_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_0.clone(),
        );

        let input_index = self.register_0.borrow_mut().get_index_from_tag("E");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R0_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_0.clone(),
        );

        let input_index = self.register_1.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R1_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_1.clone(),
        );

        let input_index = self.register_1.borrow_mut().get_index_from_tag("E");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R1_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_1.clone(),
        );

        let input_index = self.register_2.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R2_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_2.clone(),
        );

        let input_index = self.register_2.borrow_mut().get_index_from_tag("E");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R2_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_2.clone(),
        );

        let input_index = self.register_3.borrow_mut().get_index_from_tag("S");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R3_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_3.clone(),
        );

        let input_index = self.register_3.borrow_mut().get_index_from_tag("E");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::R3_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.register_3.clone(),
        );

        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::TMP_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            self.temp_s_splitter.clone(),
        );

        let input_index = self.tmp.borrow_mut().get_index_from_tag("S");
        let output_index = self.temp_s_splitter.borrow_mut().get_index_for_output(0, 0);
        self.temp_s_splitter.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.tmp.clone(),
        );

        let input_index = self.c_tmp.borrow_mut().get_index_from_tag("S");
        let output_index = self.temp_s_splitter.borrow_mut().get_index_for_output(0, 1);
        self.temp_s_splitter.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.c_tmp.clone(),
        );

        let input_index = self.bus_1.borrow_mut().get_index_from_tag("BUS_1");
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::BUS_1);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.bus_1.clone(),
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::IO);
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::IO);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            output_gates[output_gate_index].clone(),
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::DA);
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::DA);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            output_gates[output_gate_index].clone(),
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::END);
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::END);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            output_gates[output_gate_index].clone(),
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::IO_CLK_E);
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::IO_CLK_E);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            output_gates[output_gate_index].clone(),
        );

        let output_gate_index = self.get_index_from_tag(VariableBitCPU::IO_CLK_S);
        let output_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::IO_CLK_S);
        self.control_section.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            output_gates[output_gate_index].clone(),
        );
    }

    fn connect_four_cycle_clock_hookup(&mut self) {
        let cycle_block_output = self.four_cycle_clock_hookup.borrow_mut().get_index_from_tag(FourCycleClockHookup::CLK_OUT);
        self.four_cycle_clock_hookup.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            0,
            self.four_cycle_clock_clk_splitter.clone(),
        );

        let cycle_block_output = self.four_cycle_clock_hookup.borrow_mut().get_index_from_tag(FourCycleClockHookup::CLKE);
        self.four_cycle_clock_hookup.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            0,
            self.four_cycle_clock_clke_splitter.clone(),
        );

        let cycle_block_output = self.four_cycle_clock_hookup.borrow_mut().get_index_from_tag(FourCycleClockHookup::CLKS);
        self.four_cycle_clock_hookup.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            0,
            self.four_cycle_clock_clks_splitter.clone(),
        );
    }

    fn connect_four_cycle_clock_clk_splitter(
        &mut self,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let cycle_block_output = self.four_cycle_clock_clk_splitter.borrow_mut().get_index_for_output(
            0, 0,
        );
        let clock_input = self.control_section.borrow_mut().get_index_from_tag(ControlSection::CLOCK);
        self.four_cycle_clock_clk_splitter.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            clock_input,
            self.control_section.clone(),
        );

        let cycle_block_output = self.four_cycle_clock_clk_splitter.borrow_mut().get_index_for_output(
            0, 1,
        );
        self.four_cycle_clock_clk_splitter.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            1,
            self.counter_and.clone(),
        );

        let output_index = self.get_index_from_tag(VariableBitCPU::CLK_OUT);
        let cycle_block_output = self.four_cycle_clock_clk_splitter.borrow_mut().get_index_for_output(
            0, 2,
        );
        self.four_cycle_clock_clk_splitter.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            0,
            output_gates[output_index].clone(),
        );
    }

    fn connect_four_cycle_clock_clke_splitter(
        &mut self,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let cycle_block_output = self.four_cycle_clock_clke_splitter.borrow_mut().get_index_for_output(
            0, 0,
        );
        let clock_enable_input = self.control_section.borrow_mut().get_index_from_tag(ControlSection::CLOCK_ENABLE);
        self.four_cycle_clock_clke_splitter.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            clock_enable_input,
            self.control_section.clone(),
        );

        let output_index = self.get_index_from_tag(VariableBitCPU::CLKE);
        let cycle_block_output = self.four_cycle_clock_clke_splitter.borrow_mut().get_index_for_output(
            0, 1,
        );
        self.four_cycle_clock_clke_splitter.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            0,
            output_gates[output_index].clone(),
        );
    }

    fn connect_four_cycle_clock_clks_splitter(
        &mut self,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let cycle_block_output = self.four_cycle_clock_clks_splitter.borrow_mut().get_index_for_output(
            0, 0,
        );
        let clock_set_input = self.control_section.borrow_mut().get_index_from_tag(ControlSection::CLOCK_SET);
        self.four_cycle_clock_clks_splitter.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            clock_set_input,
            self.control_section.clone(),
        );

        let output_index = self.get_index_from_tag(VariableBitCPU::CLKS);
        let cycle_block_output = self.four_cycle_clock_clks_splitter.borrow_mut().get_index_for_output(
            0, 1,
        );
        self.four_cycle_clock_clks_splitter.borrow_mut().connect_output_to_next_gate(
            cycle_block_output,
            0,
            output_gates[output_index].clone(),
        );
    }

    fn connect_bus(
        &mut self,
        bus_size: usize,
        ram_cells_decoder_input: usize,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let mut mut_bus = self.bus.borrow_mut();

        //TODO: The reset pin on the ControlUnit relies on all registers being set to pull down, so if
        // they get NONE as the input, they need the bits to be set low.
        // I think this needs to be a setting inside LogicGate itself. where I make a function called
        //  'pull down'.
        //TODO : Or maybe I could do it on the bus itself, where if it sends an input in, it will
        // bring them down to zero on the output.
        //This is here to help with the reset. In case the reset goes high and sets all the pins,
        // need to make sure NONE is not passed into any of the inputs.
        mut_bus.pull_output(LOW_);

        for i in 0..bus_size {
            let input_tag = format!("i_{}", i);

            //reg_0
            let output_index = mut_bus.get_index_for_output(i, 0);
            let input_index = self.register_0.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_0.clone(),
            );

            //reg_1
            let output_index = mut_bus.get_index_for_output(i, 1);
            let input_index = self.register_1.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_1.clone(),
            );

            //reg_2
            let output_index = mut_bus.get_index_for_output(i, 2);
            let input_index = self.register_2.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_2.clone(),
            );

            //reg_3
            let output_index = mut_bus.get_index_for_output(i, 3);
            let input_index = self.register_3.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_3.clone(),
            );

            //memory address register
            if i < ram_cells_decoder_input * 2 {
                let address_input_tag = format!("addr_{}", i);
                let output_index = mut_bus.get_index_for_output(i, 4);
                let input_index = self.ram.borrow_mut().get_index_from_tag(address_input_tag.as_str());
                mut_bus.connect_output_to_next_gate(
                    output_index,
                    input_index,
                    self.ram.clone(),
                );
            }

            //ram input (multiplexer)
            let multiplexer_input_tag = format!("I_0_bit_{}", i);
            let output_index = mut_bus.get_index_for_output(i, 5);
            let input_index = self.load_multiplexer.borrow_mut().get_index_from_tag(multiplexer_input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.load_multiplexer.clone(),
            );

            //ir
            let output_index = mut_bus.get_index_for_output(i, 6);
            let input_index = self.instruction_register.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.instruction_register.clone(),
            );

            //iar
            let output_index = mut_bus.get_index_for_output(i, 7);
            let input_index = self.instruction_address_register.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.instruction_address_register.clone(),
            );

            //tmp
            let output_index = mut_bus.get_index_for_output(i, 8);
            let input_index = self.tmp.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.tmp.clone(),
            );

            //alu a
            let a_input_tag = format!("a_{}", i);
            let output_index = mut_bus.get_index_for_output(i, 9);
            let input_index = self.alu.borrow_mut().get_index_from_tag(a_input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.alu.clone(),
            );

            let input_tag = format!("{}_{}", Self::BUS, i);
            let output_gate_index = self.get_index_from_tag(input_tag.as_str());
            let output_index = mut_bus.get_index_for_output(i, 10);
            mut_bus.connect_output_to_next_gate(
                output_index,
                0,
                output_gates[output_gate_index].clone(),
            );
        }
    }

    fn connect_register_0(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
                let ram_output_index = self.ram.borrow_mut().get_index_from_tag(output_tag.as_str());
                self.ram.borrow_mut().connect_output_to_next_gate(
                    ram_output_index,
                    0,
                    output_gates[output_index].clone(),
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

        let input_index = self.flags.borrow_mut().get_index_from_tag("i_0");
        let output_index = self.alu.borrow_mut().get_index_from_tag("C_OUT");
        self.alu.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.flags.clone(),
        );

        let input_index = self.flags.borrow_mut().get_index_from_tag("i_1");
        let output_index = self.alu.borrow_mut().get_index_from_tag("A_L");
        self.alu.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.flags.clone(),
        );

        let input_index = self.flags.borrow_mut().get_index_from_tag("i_2");
        let output_index = self.alu.borrow_mut().get_index_from_tag("EQ");
        self.alu.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.flags.clone(),
        );

        let input_index = self.flags.borrow_mut().get_index_from_tag("i_3");
        let output_index = self.alu.borrow_mut().get_index_from_tag("Z");
        self.alu.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.flags.clone(),
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
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        let output_index = self.c_tmp.borrow_mut().get_index_from_tag("Q");
        self.c_tmp.borrow_mut().connect_output_to_next_gate(
            output_index,
            1,
            self.c_tmp_and.clone(),
        );
    }

    fn connect_c_tmp_and(&mut self) {
        let input_index = self.alu.borrow_mut().get_index_from_tag("C_IN");
        self.c_tmp_and.borrow_mut().connect_output_to_next_gate(
            0,
            input_index,
            self.alu.clone(),
        );
    }

    fn connect_acc(
        &mut self,
        bus_size: usize,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
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
        let output_index = self.flags.borrow_mut().get_index_from_tag("o_0");
        self.flags.borrow_mut().connect_output_to_next_gate(
            output_index,
            0,
            self.flags_c_out_splitter.clone(),
        );

        let input_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::C_IN);
        let output_index = self.flags_c_out_splitter.borrow_mut().get_index_for_output(0, 0);
        self.flags_c_out_splitter.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.control_section.clone(),
        );

        let input_index = self.c_tmp.borrow_mut().get_index_from_tag("E");
        let output_index = self.flags_c_out_splitter.borrow_mut().get_index_for_output(0, 1);
        self.flags_c_out_splitter.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.c_tmp.clone(),
        );

        let input_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::A_L);
        let output_index = self.flags.borrow_mut().get_index_from_tag("o_1");
        self.flags.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.control_section.clone(),
        );

        let input_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::EQ);
        let output_index = self.flags.borrow_mut().get_index_from_tag("o_2");
        self.flags.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.control_section.clone(),
        );

        let input_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::Z);
        let output_index = self.flags.borrow_mut().get_index_from_tag("o_3");
        self.flags.borrow_mut().connect_output_to_next_gate(
            output_index,
            input_index,
            self.control_section.clone(),
        );
    }

    fn connect_end_input_and_gate(&mut self) {
        let clk_input_index = self.four_cycle_clock_hookup.borrow_mut().get_index_from_tag(FourCycleClockHookup::CLK_IN);
        self.end_input_and_gate.borrow_mut().connect_output_to_next_gate(
            0,
            clk_input_index,
            self.four_cycle_clock_hookup.clone(),
        );
    }

    fn connect_end_input_not_gate(&mut self) {
        self.end_input_not_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.end_input_and_gate.clone(),
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
        let clock_input = self.load_counter.borrow_mut().get_index_from_tag(VariableBitCounter::CLK_IN);
        self.counter_and.borrow_mut().connect_output_to_next_gate(
            0,
            clock_input,
            self.load_counter.clone(),
        );
    }

    fn connect_load_input_splitter(&mut self) {
        let splitter_output_index = self.load_input_splitter.borrow_mut().get_index_for_output(
            0, 0,
        );
        let load_input_index = self.control_section.borrow_mut().get_index_from_tag(ControlSection::HIGH_LVL_LOAD);
        self.load_input_splitter.borrow_mut().connect_output_to_next_gate(
            splitter_output_index,
            load_input_index,
            self.control_section.clone(),
        );

        let splitter_output_index = self.load_input_splitter.borrow_mut().get_index_for_output(
            0, 1,
        );
        let enable_index = self.counter_controlled_buffer.borrow_mut().get_index_from_tag("E");
        self.load_input_splitter.borrow_mut().connect_output_to_next_gate(
            splitter_output_index,
            enable_index,
            self.counter_controlled_buffer.clone(),
        );

        let splitter_output_index = self.load_input_splitter.borrow_mut().get_index_for_output(
            0, 2,
        );
        self.load_input_splitter.borrow_mut().connect_output_to_next_gate(
            splitter_output_index,
            0,
            self.counter_and.clone(),
        );

        let splitter_output_index = self.load_input_splitter.borrow_mut().get_index_for_output(
            0, 3,
        );
        let multiplexed_control_index = self.load_multiplexer.borrow_mut().get_index_from_tag("C_0");
        self.load_input_splitter.borrow_mut().connect_output_to_next_gate(
            splitter_output_index,
            multiplexed_control_index,
            self.load_multiplexer.clone(),
        );
    }

    fn connect_reset_controlled_buffer(&mut self, bus_size: usize) {
        for i in 0..bus_size {
            let input_tag = format!("i_{}", i);
            let input_index = self.reset_controlled_buffer.borrow_mut().get_index_from_tag(input_tag.as_str());
            self.reset_controlled_buffer.borrow_mut().update_input_signal(
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
        clock: &Rc<RefCell<Clock>>
    ) {
        let input_index = self.get_index_from_tag(VariableBitCPU::CLK_IN);
        let output_signals = self.complex_gate.input_gates[input_index].borrow_mut().fetch_output_signals().unwrap();
        let output_signal =
            match output_signals.first().unwrap() {
                GateOutputState::NotConnected(_) => panic!("SimpleInput for VariableBitCPU CLK_IN should always be connected."),
                GateOutputState::Connected(connected_output) => {
                    connected_output.throughput.signal.clone()
                }
            };
        clock.borrow_mut().set_clock_state(output_signal);
    }
}

impl LogicGate for VariableBitCPU {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.complex_gate.connect_output_to_next_gate(
            self.get_unique_id(),
            current_gate_output_key,
            next_gate_input_key,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
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

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        self.complex_gate.remove_connected_input(input_index, connected_id);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::globals::{CLOCK_TICK_NUMBER, END_OUTPUT_GATE_TAG, get_clock_tick_number};
    use crate::logic::foundations::{GateOutputState, LogicGate, Signal};
    use crate::logic::foundations::Signal::{HIGH, LOW_};
    use crate::logic::input_gates::{AutomaticInput, Clock};
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::logic::processor_components::RAMUnit;
    use crate::logic::variable_bit_cpu::VariableBitCPU;
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::extract_output_tags_sorted_by_index;

    fn generate_default_output(cpu: &Rc<RefCell<VariableBitCPU>>) -> Vec<Signal> {

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

        let mut generated_signals = vec![LOW_; cpu.borrow_mut().complex_gate.output_gates.len()];
        let clke_index = cpu.borrow_mut().complex_gate.gate_tags_to_index[VariableBitCPU::CLKE].index;
        generated_signals[clke_index] = HIGH;
        generated_signals
    }

    fn convert_binary_to_inputs_for_load(binary_strings: Vec<&str>, num_ram_cells: usize) -> Vec<Rc<RefCell<AutomaticInput>>> {
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

    fn collect_signals_from_cpu(cpu: &Rc<RefCell<VariableBitCPU>>) -> Vec<Signal> {
        let cpu_output = cpu.borrow_mut().fetch_output_signals().unwrap();
        let mut collected_signals = Vec::new();
        for out in cpu_output.into_iter() {
            match out {
                GateOutputState::NotConnected(signal) => {
                    collected_signals.push(signal);
                }
                GateOutputState::Connected(_) => panic!("Clock should not be connected to anything")
            }
        }
        collected_signals
    }

    //This should leave the cpu in the same state as it started in. The only difference is that
    // there will now be values loaded into RAM. It should be run without any inputs connected to
    // the cpu itself.
    fn load_values_into_ram(
        cpu: &Rc<RefCell<VariableBitCPU>>,
        binary_strings: Vec<&str>,
        num_ram_cells: usize,
    ) {
        let automatic_inputs = convert_binary_to_inputs_for_load(
            binary_strings.clone(),
            num_ram_cells,
        );

        //todo d
        // cpu.borrow_mut().four_cycle_clock_hookup.borrow_mut().toggle_output_printing(true);

        let num_cycles = num_ram_cells * 4 - 2;

        //The last two cycles are turn the LOAD and MARS to low in order to advance the clock to
        // the starting position.
        let mut output_values = vec![HIGH; num_cycles + 2];

        //This will allow the values for LOAD and MARS to actually be set low as the final clock
        // tick.
        output_values[num_cycles] = LOW_;
        output_values[num_cycles + 1] = LOW_;

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
            println!("CLOCK TICK {}", get_clock_tick_number());

            continue_load_operation = run_circuit(
                &input_gates,
                &Vec::new(),
                propagate_signal,
                &mut |_clock_tick_inputs, _output_gates| {},
                None,
            );

            propagate_signal = false;
        }

        let mut generated_output = generate_default_output(&cpu);

        for (i, binary_string) in binary_strings.iter().enumerate() {
            for (j, c) in binary_string.chars().rev().enumerate() {
                let output_tag = RAMUnit::get_ram_output_string(i, j);
                let output_index = cpu.borrow_mut().complex_gate.gate_tags_to_index[&output_tag.to_string()].index;

                let signal =
                    if c == '0' {
                        LOW_
                    } else {
                        HIGH
                    };

                generated_output[output_index] = signal.clone();
            }
        }

        let collected_signals = collect_signals_from_cpu(&cpu);

        let failed = compare_generate_and_collected_output(&cpu, generated_output, collected_signals);

        assert!(!failed);

        //Disconnect all inputs so that future connections can be made.
        for automatic_input_gate in automatic_input_gates.into_iter() {
            automatic_input_gate.borrow_mut().disconnect_gate(0);
        }

        clock.borrow_mut().disconnect_gate(0);
    }

    fn compare_generate_and_collected_output(
        cpu: &Rc<RefCell<VariableBitCPU>>,
        generated_output: Vec<Signal>,
        collected_signals: Vec<Signal>,
    ) -> bool {
        let tags_sorted_by_index = extract_output_tags_sorted_by_index(&cpu.borrow_mut().complex_gate);

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
                println!("Clock tick {}\nfailed (passed, collected): {:?}", i, failed_map);
            }
        }
        failed
    }

    fn reset_cpu_values(cpu: &Rc<RefCell<VariableBitCPU>>) {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let reset_input = AutomaticInput::new(
            vec![HIGH],
            1,
            "RESET",
        );

        let reset_cpu_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::RESET);
        reset_input.borrow_mut().connect_output_to_next_gate(
            0,
            reset_cpu_index,
            cpu.clone(),
        );

        input_gates.push(reset_input.clone());

        run_circuit(
            &input_gates,
            &Vec::new(),
            false,
            &mut |_clock_tick_inputs, _output_gates| {},
            None,
        );

        let collected_signals = collect_signals_from_cpu(&cpu);
        let generated_output = generate_default_output(&cpu);

        let failed = compare_generate_and_collected_output(&cpu, generated_output, collected_signals);

        assert!(!failed);

        reset_input.borrow_mut().disconnect_gate(0);
    }

    #[test]
    fn initialization() {
        let cpu = VariableBitCPU::new(8, 4);

        let collected_signals = collect_signals_from_cpu(&cpu);

        let generated_signals = generate_default_output(&cpu);

        assert_eq!(collected_signals, generated_signals);
    }

    #[test]
    fn load() {
        let number_bits = 8;
        let decoder_input_size = 1;
        let cpu = VariableBitCPU::new(number_bits, decoder_input_size);

        let binary_strings = vec![
            "11110000",
            "00110011",
            "01010110",
        ];

        let num_ram_cells = usize::pow(2, (decoder_input_size * 2) as u32);
        assert!(binary_strings.len() <= num_ram_cells);
        if !binary_strings.is_empty() {
            assert_eq!(binary_strings[0].len(), number_bits);
        }

        load_values_into_ram(
            &cpu,
            binary_strings,
            num_ram_cells,
        );
    }

    #[test]
    fn reset() {
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
            binary_strings,
            num_ram_cells,
        );

        reset_cpu_values(&cpu);
    }

    #[test]
    fn end() {
        let number_bits = 8;
        let decoder_input_size = 1;
        let cpu = VariableBitCPU::new(number_bits, decoder_input_size);

        let binary_strings = vec![
            VariableBitCPU::END_INSTRUCTION,
        ];

        //tODO: When it disconnects from the complex gate, I don't think it is changing the input of the gates themselves
        let num_ram_cells = usize::pow(2, (decoder_input_size * 2) as u32);
        assert!(binary_strings.len() <= num_ram_cells);
        if !binary_strings.is_empty() {
            assert_eq!(binary_strings[0].len(), number_bits);
        }
        //todo d
        cpu.borrow_mut().ram.borrow_mut().toggle_output_printing(true);

        load_values_into_ram(
            &cpu,
            binary_strings,
            num_ram_cells,
        );

        //TODO: make a `run_current_program` function
        //TODO: probably put a time limit on this test

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let clock = Clock::new(1, "PRIMARY_CLOCK");
        let clk_in_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::CLK_IN);
        //TODO: sync this
        // get_clock_synced_with_cpu
        clock.borrow_mut().connect_output_to_next_gate(
            0,
            clk_in_index,
            cpu.clone(),
        );

        input_gates.push(clock.clone());

        //TODO: need to delete this, it needs to run with just the clock.
        input_gates.push(AutomaticInput::new(vec![LOW_; 12], 1, "LOAD"));

        let current_clock_tick_number = get_clock_tick_number();

        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();

        //TODO: The gate is never set to HIGH, maybe print isn't input and see if its actually connected properly.

        let end_output_gate = SimpleOutput::new(END_OUTPUT_GATE_TAG);


        //todo d
        // end_output_gate.borrow_mut().toggle_output_printing(true);
        println!("end_output_gate id {}", end_output_gate.borrow_mut().get_unique_id().id());
        println!("cpu id {}", cpu.borrow_mut().get_unique_id().id());
        // cpu.borrow_mut().toggle_output_printing(true);


        let cpu_end_index = cpu.borrow_mut().get_index_from_tag(VariableBitCPU::END);
        println!("cpu_end_index {cpu_end_index}");
        cpu.borrow_mut().connect_output_to_next_gate(
            cpu_end_index,
            0,
            end_output_gate.clone(),
        );

        // cpu.borrow_mut().control_section.borrow_mut().toggle_output_printing(true);
        // cpu.borrow_mut().instruction_register.borrow_mut().toggle_output_printing(true);
        // cpu.borrow_mut().ram.borrow_mut().toggle_output_printing(true);
        // cpu.borrow_mut().bus.borrow_mut().toggle_output_printing(true);

        //TODO: There is a problem here actually right, the way the clock works now, it doesn't ever
        // reset the mem_address of the RAM to 0 even though the counter (I think) went back to 0.

        //TODO: something is wrong with the fetch cycle

        //TODO: This all stems from changing the clock, what happened?
        // The problem is that the ram unit is getting all HIGH input values from the BUS.
        // When the RAM_S goes HIGH this is set inside the RAM.
        // The BUS seems to be getting it from id 2160, or the RAM_UNIT.
        // On CLOCK TICK 16, the bus has all low values inside it.
        // So on CLOCK TICK 17, the RAM_UNIT gets all HIGH values from the bus when RAM_S is HIGH and mem_0 is saved

        //todo d
        // cpu.borrow_mut().complex_gate.output_gates[cpu_end_index].borrow_mut().toggle_output_printing(true);

        output_gates.push(end_output_gate.clone());

        let mut continue_load_operation = true;
        let mut propagate_signal = true;
        let mut num = 0;
        while continue_load_operation {
            unsafe {
                CLOCK_TICK_NUMBER += 1;
            }
            println!("CLOCK TICK {}", get_clock_tick_number());

            //todo d
            if get_clock_tick_number() > 1 {
                // let output = cpu.borrow_mut().fetch_output_signals().unwrap();

                if get_clock_tick_number() > 18 {
                    break;
                }

                // println!("SimpleOutput output: {:#?}", end_output_gate.borrow_mut().fetch_output_signals());
                // println!("cpu END output: {:?}", output[cpu_end_index]);
                // RUN_CIRCUIT_IS_HIGH_LEVEL.store(true, Ordering::Relaxed);
            }
            num += 1;

            continue_load_operation = run_circuit(
                &input_gates,
                &output_gates,
                propagate_signal,
                &mut |_clock_tick_inputs, _output_gates| {},
                None,
            );

            propagate_signal = false;
        }

        //TODO: this is probably not correct, the cycle should take four clock ticks or something
        assert_eq!(current_clock_tick_number, get_clock_tick_number());
    }


    //TODO: test each instruction (can look at the registers to make sure things are properly loaded).
}

use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::arithmetic_gates::ArithmeticLogicUnit;
use crate::logic::basic_gates::{And, Splitter};
use crate::logic::complex_logic::FourCycleClockHookup;
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
    clock: Rc<RefCell<Clock>>,
    four_cycle_clock_hookup: Rc<RefCell<FourCycleClockHookup>>,
    //TODO: The other pieces
}

#[allow(dead_code)]
impl VariableBitCPU {

    //Inputs
    pub const LOAD: &'static str = "LOAD";
    pub const RESET: &'static str = "RESET";
    pub const MARS: &'static str = "MARS";
    pub const END: &'static str = "END";

    //Outputs
    pub const R0: &'static str = "R0";
    pub const R1: &'static str = "R1";
    pub const R2: &'static str = "R2";
    pub const R3: &'static str = "R3";
    pub const IR: &'static str = "IR";
    pub const IAR: &'static str = "IAR";
    pub const ACC: &'static str = "ACC";
    pub const TMP: &'static str = "TMP";
    pub const FLAGS: &'static str = "FLAGS";

    pub fn new(number_bits: usize, ram_cells_decoder_input: usize) -> Rc<RefCell<Self>> {
        assert_ne!(number_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        //TODO: The reset pin on the ControlUnit relies on all registers being set to pull down, so if
        // they get NONE as the input, they need the bits to be set low.

        input_gates.push(SimpleInput::new(1, VariableBitCPU::LOAD));
        input_gates.push(SimpleInput::new(1, VariableBitCPU::RESET));
        input_gates.push(SimpleInput::new(1, VariableBitCPU::MARS));
        input_gates.push(SimpleInput::new(1, VariableBitCPU::END));

        let mut store_output = |gate: Rc<RefCell<SimpleOutput>>| {
            output_gates.push(gate.clone());
            output_gates_logic.push(gate.clone());
        };

        store_output(SimpleOutput::new(VariableBitCPU::R0));
        store_output(SimpleOutput::new(VariableBitCPU::R1));
        store_output(SimpleOutput::new(VariableBitCPU::R2));
        store_output(SimpleOutput::new(VariableBitCPU::R3));
        store_output(SimpleOutput::new(VariableBitCPU::IR));
        store_output(SimpleOutput::new(VariableBitCPU::IAR));
        store_output(SimpleOutput::new(VariableBitCPU::ACC));
        store_output(SimpleOutput::new(VariableBitCPU::TMP));
        store_output(SimpleOutput::new(VariableBitCPU::FLAGS));

        let mut bit_register = VariableBitCPU {
            complex_gate: ComplexGateMembers::new(
                0,
                0,
                GateType::VariableBitCPUType,
                input_gates,
                output_gates,
            ),
            control_section: ControlSection::new(number_bits),
            temp_s_splitter: Splitter::new(1, 2),
            bus: Splitter::new(number_bits, 10),
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
            c_tmp_and: And::new(2,1),
            acc: VariableBitRegister::new(number_bits),
            flags: VariableBitMemoryCell::new(4), //size 4 for the alu outputs
            flags_c_out_splitter: Splitter::new(1, 2),
            clock: Clock::new(1, "CLK"),
            four_cycle_clock_hookup: FourCycleClockHookup::new(),
        };

        bit_register.build_and_prime_circuit(number_bits, output_gates_logic);

        Rc::new(RefCell::new(bit_register))
    }

    pub fn get_clock(&self) -> Rc<RefCell<Clock>> {
        self.clock.clone()
    }

    fn build_and_prime_circuit(
        &mut self,
        bus_size: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {

        //TODO: do the clock and the four_cycle hookups
        self.connect_bus(bus_size);
        self.connect_register_0(bus_size);
        self.connect_register_1(bus_size);
        self.connect_register_2(bus_size);
        self.connect_register_3(bus_size);
        self.connect_instruction_address_register(bus_size);
        self.connect_instruction_register(bus_size);
        self.connect_ram(bus_size);
        self.connect_alu(bus_size);
        self.connect_bus_1(bus_size);
        self.connect_tmp(bus_size);
        self.connect_c_tmp();
        self.connect_c_tmp_and();
        self.connect_acc(bus_size);
        self.connect_flags();

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }

    fn connect_control_section(&mut self) {
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
    }

    fn connect_bus(&mut self, bus_size: usize) {
        let mut mut_bus = self.bus.borrow_mut();

        for i in 0..bus_size {
            let input_tag = format!("i_{}", i);

            //reg_0
            let output_index = mut_bus.get_index_for_output(0, i);
            let input_index = self.register_0.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_0.clone(),
            );

            //reg_1
            let output_index = mut_bus.get_index_for_output(1, i);
            let input_index = self.register_1.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_1.clone(),
            );

            //reg_2
            let output_index = mut_bus.get_index_for_output(2, i);
            let input_index = self.register_2.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_2.clone(),
            );

            //reg_3
            let output_index = mut_bus.get_index_for_output(3, i);
            let input_index = self.register_3.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.register_3.clone(),
            );

            //memory address register
            let address_input_tag = format!("addr_{}", i);
            let output_index = mut_bus.get_index_for_output(4, i);
            let input_index = self.ram.borrow_mut().get_index_from_tag(address_input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.ram.clone(),
            );

            //ram input
            let output_index = mut_bus.get_index_for_output(5, i);
            let input_index = self.ram.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.ram.clone(),
            );

            //ir
            let output_index = mut_bus.get_index_for_output(6, i);
            let input_index = self.instruction_register.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.instruction_register.clone(),
            );

            //iar
            let output_index = mut_bus.get_index_for_output(7, i);
            let input_index = self.instruction_address_register.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.instruction_address_register.clone(),
            );

            //tmp
            let output_index = mut_bus.get_index_for_output(8, i);
            let input_index = self.tmp.borrow_mut().get_index_from_tag(input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.tmp.clone(),
            );

            //alu a
            let a_input_tag = format!("a_{}", i);
            let output_index = mut_bus.get_index_for_output(9, i);
            let input_index = self.alu.borrow_mut().get_index_from_tag(a_input_tag.as_str());
            mut_bus.connect_output_to_next_gate(
                output_index,
                input_index,
                self.alu.clone(),
            );
        }
    }

    fn connect_input_to_output(
        bus_size: usize,
        input_gate: Rc<RefCell<dyn LogicGate>>,
        output_gate: Rc<RefCell<dyn LogicGate>>,
        input_val: &str,
    ) {
        for i in 0..bus_size {
            let input_tag = format!("{}_{}", input_val, i);
            let output_tag = format!("o_{}", i);
            let input_index = output_gate.borrow_mut().get_index_from_tag(input_tag.as_str());
            let output_index = input_gate.borrow_mut().get_index_from_tag(output_tag. as_str());
            input_gate.borrow_mut().connect_output_to_next_gate(
                output_index,
                input_index,
                output_gate.clone(),
            );
        }
    }

    fn connect_register_0(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_0.clone(),
            self.bus.clone(),
            "i",
        );
    }

    fn connect_register_1(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_1.clone(),
            self.bus.clone(),
            "i",
        );
    }

    fn connect_register_2(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_2.clone(),
            self.bus.clone(),
            "i",
        );
    }
    fn connect_register_3(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.register_3.clone(),
            self.bus.clone(),
            "i",
        );
    }

    fn connect_instruction_address_register(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.instruction_address_register.clone(),
            self.bus.clone(),
            "i",
        );
    }

    fn connect_instruction_register(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.instruction_register.clone(),
            self.control_section.clone(),
            "IR",
        );
    }

    fn connect_ram(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.ram.clone(),
            self.bus.clone(),
            "i",
        );
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

    fn connect_tmp(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.tmp.clone(),
            self.bus_1.clone(),
            "i",
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
            self.c_tmp_and.clone(),
        );
    }

    fn connect_acc(&mut self, bus_size: usize) {
        VariableBitCPU::connect_input_to_output(
            bus_size,
            self.acc.clone(),
            self.bus.clone(),
            "i",
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
}

#[cfg(test)]
mod tests {
    use crate::logic::foundations::LogicGate;
    use crate::logic::variable_bit_cpu::VariableBitCPU;

    //TODO: might want to make a public function of 'dump registers' or 'dump memory' or something?
    // It would help a lot with testing. After all, all the registers have the little hookup to show
    // what they currently store.
    // Right now the way that it gets the memory is actually part of the circuit, might want to change
    // that so it is programmatically done instead. There isn't actually any need to do it the way
    // I currently am.
    // In fact anything higher level than the CPU itself (so higher level than this) can be done
    // programmatically. For example looking at the RAM.
    #[test]
    fn variable_bit_cpu() {
        let cpu = VariableBitCPU::new(8, 4);
        cpu.borrow_mut().toggle_output_printing(true);
    }
}

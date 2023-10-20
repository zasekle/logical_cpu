use std::time::Instant;
use crate::CONTROL_SECTION_TIME;
use crate::logic::basic_gates::{And, Not, Or, Splitter};
use crate::logic::complex_logic::VariableOutputStepper;

#[allow(unused_imports)]
use crate::logic::foundations::{BasicGateMembers, ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::foundations::connect_gates;
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW_, HIGH};
use crate::logic::processor_components::VariableDecoder;
use crate::shared_mutex::{new_shared_mutex, SharedMutex};

pub struct ControlSection {
    complex_gate: ComplexGateMembers,
    clk_and: SharedMutex<And>,
    load_not: SharedMutex<Not>,
    reset_not: SharedMutex<Not>,
    stepper: SharedMutex<VariableOutputStepper>,
    stepper_splitters: Vec<SharedMutex<Splitter>>,
    stepper_1_and: SharedMutex<And>,
    bus_1_or: SharedMutex<Or>,
    ram_e_and: SharedMutex<And>,
    ram_e_or: SharedMutex<Or>,
    acc_e_and: SharedMutex<And>,
    acc_e_or: SharedMutex<Or>,
    iar_e_and: SharedMutex<And>,
    iar_e_or: SharedMutex<Or>,
    io_clk_e_and: SharedMutex<And>,
    io_clks_s_and: SharedMutex<And>,
    r0_e_or: SharedMutex<Or>,
    r0_e_reg_b_and: SharedMutex<And>,
    r0_e_reg_a_and: SharedMutex<And>,
    r1_e_or: SharedMutex<Or>,
    r1_e_reg_b_and: SharedMutex<And>,
    r1_e_reg_a_and: SharedMutex<And>,
    r2_e_or: SharedMutex<Or>,
    r2_e_reg_b_and: SharedMutex<And>,
    r2_e_reg_a_and: SharedMutex<And>,
    r3_e_or: SharedMutex<Or>,
    r3_e_reg_b_and: SharedMutex<And>,
    r3_e_reg_a_and: SharedMutex<And>,
    r_e_reg_b_decoder: SharedMutex<VariableDecoder>,
    r_e_reg_a_decoder: SharedMutex<VariableDecoder>,
    mar_s_or: SharedMutex<Or>,
    mar_s_and: SharedMutex<And>,
    mar_s_outer_or: SharedMutex<Or>,
    ram_s_or: SharedMutex<Or>,
    ram_s_load_and: SharedMutex<And>,
    ram_s_and: SharedMutex<And>,
    acc_s_or: SharedMutex<Or>,
    acc_s_and: SharedMutex<And>,
    acc_s_outer_or: SharedMutex<Or>,
    iar_s_or: SharedMutex<Or>,
    iar_s_and: SharedMutex<And>,
    iar_s_outer_or: SharedMutex<Or>,
    r0_s_or: SharedMutex<Or>,
    r0_s_and: SharedMutex<And>,
    r1_s_or: SharedMutex<Or>,
    r1_s_and: SharedMutex<And>,
    r2_s_or: SharedMutex<Or>,
    r2_s_and: SharedMutex<And>,
    r3_s_or: SharedMutex<Or>,
    r3_s_and: SharedMutex<And>,
    r_s_decoder: SharedMutex<VariableDecoder>,
    ir_s_or: SharedMutex<Or>,
    ir_s_and: SharedMutex<And>,
    tmp_s_or: SharedMutex<Or>,
    tmp_s_and: SharedMutex<And>,
    reg_b_e_or: SharedMutex<Or>,
    reg_a_or: SharedMutex<Or>,
    alu_0_and: SharedMutex<And>,
    alu_1_and: SharedMutex<And>,
    alu_2_and: SharedMutex<And>,
    flags_s_or: SharedMutex<Or>,
    flags_s_and: SharedMutex<And>,
    flags_s_outer_or: SharedMutex<Or>,
    reg_b_s_or: SharedMutex<Or>,
    load_store_instr_not: SharedMutex<Not>,
    load_store_instr_decoder: SharedMutex<VariableDecoder>,
    load_store_instr_0_top_and: SharedMutex<And>,
    load_store_instr_1_and: SharedMutex<And>,
    load_store_instr_2_and: SharedMutex<And>,
    load_store_instr_3_and: SharedMutex<And>,
    load_store_instr_4_and: SharedMutex<And>,
    load_store_instr_5_and: SharedMutex<And>,
    load_store_instr_6_and: SharedMutex<And>,
    load_store_instr_7_and: SharedMutex<And>,
    stepper_out_4_top_0_and: SharedMutex<And>,
    stepper_out_4_1_and: SharedMutex<And>,
    stepper_out_4_2_and: SharedMutex<And>,
    stepper_out_4_3_and: SharedMutex<And>,
    stepper_out_4_4_and: SharedMutex<And>,
    stepper_out_4_5_and: SharedMutex<And>,
    stepper_out_4_6_and: SharedMutex<And>,
    stepper_out_4_7_and: SharedMutex<And>,
    stepper_out_4_8_and: SharedMutex<And>,
    stepper_out_5_top_0_and: SharedMutex<And>,
    stepper_out_5_1_and: SharedMutex<And>,
    stepper_out_5_2_and: SharedMutex<And>,
    stepper_out_5_3_and: SharedMutex<And>,
    stepper_out_5_4_and: SharedMutex<And>,
    stepper_out_5_5_and: SharedMutex<And>,
    stepper_out_5_6_and: SharedMutex<And>,
    stepper_out_5_6_not: SharedMutex<Not>,
    stepper_out_6_top_0_and: SharedMutex<And>,
    stepper_out_6_1_and: SharedMutex<And>,
    stepper_out_6_2_and: SharedMutex<And>,
    eight_input_and: SharedMutex<And>,
    eight_input_and_not_loc_2: SharedMutex<Not>,
    eight_input_and_not_loc_3: SharedMutex<Not>,
    c_in_and: SharedMutex<And>,
    a_l_and: SharedMutex<And>,
    eq_and: SharedMutex<And>,
    z_and: SharedMutex<And>,
    alu_input_or: SharedMutex<Or>,
    add_and: SharedMutex<And>,
    add_not: SharedMutex<Not>,
}

#[allow(dead_code)]
impl ControlSection {
    //Inputs
    pub const CLOCK: &'static str = "CLK";
    pub const CLOCK_ENABLE: &'static str = "CLKE";
    pub const CLOCK_SET: &'static str = "CLKS";
    pub const C_IN: &'static str = "C_IN";
    pub const A_L: &'static str = "A_L";
    pub const EQ: &'static str = "EQ";
    pub const Z: &'static str = "Z";

    //High level inputs
    pub const HIGH_LVL_MARS: &'static str = "H_MARS";
    pub const HIGH_LVL_RESET: &'static str = "H_RESET";
    pub const HIGH_LVL_LOAD: &'static str = "H_LOAD";

    //Outputs
    pub const BUS_1: &'static str = "BUS_1";
    pub const RAM_E: &'static str = "RAM_E";
    pub const ACC_E: &'static str = "ACC_E";
    pub const IAR_E: &'static str = "IAR_E";
    pub const R0_E: &'static str = "R0_E";
    pub const R1_E: &'static str = "R1_E";
    pub const R2_E: &'static str = "R2_E";
    pub const R3_E: &'static str = "R3_E";
    pub const MAR_S: &'static str = "MAR_S";
    pub const RAM_S: &'static str = "RAM_S";
    pub const ACC_S: &'static str = "ACC_S";
    pub const IAR_S: &'static str = "IAR_S";
    pub const R0_S: &'static str = "R0_S";
    pub const R1_S: &'static str = "R1_S";
    pub const R2_S: &'static str = "R2_S";
    pub const R3_S: &'static str = "R3_S";
    pub const IR_S: &'static str = "IR_S";
    pub const TMP_S: &'static str = "TMP_S";
    pub const ALU_0: &'static str = "ALU_0";
    pub const ALU_1: &'static str = "ALU_1";
    pub const ALU_2: &'static str = "ALU_2";
    pub const FLAG_S: &'static str = "FLAG_S";
    pub const IO_CLK_S: &'static str = "IO_CLK_S";
    pub const IO_CLK_E: &'static str = "IO_CLK_E";
    pub const C_OUT: &'static str = "C_OUT";
    pub const END: &'static str = "END";
    pub const IO: &'static str = "IO";
    pub const DA: &'static str = "DA";

    pub fn new(bus_width: usize) -> SharedMutex<Self> {
        assert!(bus_width > 7);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        input_gates.push(SimpleInput::new(4, "IR_0"));
        input_gates.push(SimpleInput::new(4, "IR_1"));
        input_gates.push(SimpleInput::new(4, "IR_2"));
        input_gates.push(SimpleInput::new(6, "IR_3"));
        input_gates.push(SimpleInput::new(4, "IR_4"));
        input_gates.push(SimpleInput::new(4, "IR_5"));
        input_gates.push(SimpleInput::new(4, "IR_6"));
        input_gates.push(SimpleInput::new(8, "IR_7"));

        for i in 8..bus_width {
            let input_tag = format!("IR_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));
        }

        let mut stepper_splitters = Vec::new();

        stepper_splitters.push(Splitter::new(1, 1));
        stepper_splitters.push(Splitter::new(1, 2));
        stepper_splitters.push(Splitter::new(1, 2));
        stepper_splitters.push(Splitter::new(1, 9));
        stepper_splitters.push(Splitter::new(1, 10));
        stepper_splitters.push(Splitter::new(1, 3));

        input_gates.push(SimpleInput::new(13, ControlSection::CLOCK_SET));
        input_gates.push(SimpleInput::new(1, ControlSection::CLOCK));
        input_gates.push(SimpleInput::new(12, ControlSection::CLOCK_ENABLE));
        input_gates.push(SimpleInput::new(1, ControlSection::HIGH_LVL_MARS));
        input_gates.push(SimpleInput::new(11, ControlSection::HIGH_LVL_RESET));
        input_gates.push(SimpleInput::new(2, ControlSection::HIGH_LVL_LOAD));
        input_gates.push(SimpleInput::new(1, ControlSection::C_IN));
        input_gates.push(SimpleInput::new(1, ControlSection::A_L));
        input_gates.push(SimpleInput::new(1, ControlSection::EQ));
        input_gates.push(SimpleInput::new(1, ControlSection::Z));

        let mut store_output = |gate: SharedMutex<SimpleOutput>| {
            output_gates.push(gate.clone());
            output_gates_logic.push(gate.clone());
        };

        store_output(SimpleOutput::new(ControlSection::BUS_1));
        store_output(SimpleOutput::new(ControlSection::RAM_E));
        store_output(SimpleOutput::new(ControlSection::ACC_E));
        store_output(SimpleOutput::new(ControlSection::IAR_E));
        store_output(SimpleOutput::new(ControlSection::R0_E));
        store_output(SimpleOutput::new(ControlSection::R1_E));
        store_output(SimpleOutput::new(ControlSection::R2_E));
        store_output(SimpleOutput::new(ControlSection::R3_E));
        store_output(SimpleOutput::new(ControlSection::MAR_S));
        store_output(SimpleOutput::new(ControlSection::RAM_S));
        store_output(SimpleOutput::new(ControlSection::ACC_S));
        store_output(SimpleOutput::new(ControlSection::IAR_S));
        store_output(SimpleOutput::new(ControlSection::R0_S));
        store_output(SimpleOutput::new(ControlSection::R1_S));
        store_output(SimpleOutput::new(ControlSection::R2_S));
        store_output(SimpleOutput::new(ControlSection::R3_S));
        store_output(SimpleOutput::new(ControlSection::IR_S));
        store_output(SimpleOutput::new(ControlSection::TMP_S));
        store_output(SimpleOutput::new(ControlSection::ALU_0));
        store_output(SimpleOutput::new(ControlSection::ALU_1));
        store_output(SimpleOutput::new(ControlSection::ALU_2));
        store_output(SimpleOutput::new(ControlSection::FLAG_S));
        store_output(SimpleOutput::new(ControlSection::IO_CLK_E));
        store_output(SimpleOutput::new(ControlSection::IO_CLK_S));
        store_output(SimpleOutput::new(ControlSection::C_OUT));
        store_output(SimpleOutput::new(ControlSection::END));
        store_output(SimpleOutput::new(ControlSection::IO));
        store_output(SimpleOutput::new(ControlSection::DA));

        let mut control_section = ControlSection {
            complex_gate: ComplexGateMembers::new(
                bus_width + 10,
                28,
                GateType::ControlSectionType,
                input_gates,
                output_gates,
            ),
            clk_and: And::new(3, 1),
            load_not: Not::new(2),
            reset_not: Not::new(2),
            stepper: VariableOutputStepper::new(6),
            stepper_splitters,
            stepper_1_and: And::new(3, 4),
            bus_1_or: Or::new(4, 1),
            ram_e_and: And::new(2, 1),
            ram_e_or: Or::new(5, 1),
            acc_e_and: And::new(2, 1),
            acc_e_or: Or::new(4, 1),
            iar_e_and: And::new(2, 1),
            iar_e_or: Or::new(4, 1),
            io_clk_e_and: And::new(2, 1),
            io_clks_s_and: And::new(2, 1),
            r0_e_or: Or::new(2, 1),
            r0_e_reg_b_and: And::new(3, 1),
            r0_e_reg_a_and: And::new(3, 1),
            r1_e_or: Or::new(2, 1),
            r1_e_reg_b_and: And::new(3, 1),
            r1_e_reg_a_and: And::new(3, 1),
            r2_e_or: Or::new(2, 1),
            r2_e_reg_b_and: And::new(3, 1),
            r2_e_reg_a_and: And::new(3, 1),
            r3_e_or: Or::new(2, 1),
            r3_e_reg_b_and: And::new(3, 1),
            r3_e_reg_a_and: And::new(3, 1),
            r_e_reg_b_decoder: VariableDecoder::new(2),
            r_e_reg_a_decoder: VariableDecoder::new(2),
            mar_s_or: Or::new(3, 1),
            mar_s_and: And::new(2, 1),
            mar_s_outer_or: Or::new(6, 1),
            ram_s_or: Or::new(2, 1),
            ram_s_load_and: And::new(2, 1),
            ram_s_and: And::new(2, 1),
            acc_s_or: Or::new(2, 1),
            acc_s_and: And::new(2, 1),
            acc_s_outer_or: Or::new(4, 1),
            iar_s_or: Or::new(2, 1),
            iar_s_and: And::new(2, 1),
            iar_s_outer_or: Or::new(6, 1),
            r0_s_or: Or::new(2, 1),
            r0_s_and: And::new(3, 1),
            r1_s_or: Or::new(2, 1),
            r1_s_and: And::new(3, 1),
            r2_s_or: Or::new(2, 1),
            r2_s_and: And::new(3, 1),
            r3_s_or: Or::new(2, 1),
            r3_s_and: And::new(3, 1),
            r_s_decoder: VariableDecoder::new(2),
            ir_s_or: Or::new(2, 1),
            ir_s_and: And::new(2, 1),
            tmp_s_or: Or::new(2, 1),
            tmp_s_and: And::new(2, 1),
            reg_b_e_or: Or::new(4, 4),
            reg_a_or: Or::new(3, 4),
            alu_0_and: And::new(3, 1),
            alu_1_and: And::new(3, 1),
            alu_2_and: And::new(3, 1),
            flags_s_or: Or::new(2, 1),
            flags_s_and: And::new(2, 1),
            flags_s_outer_or: Or::new(2, 1),
            reg_b_s_or: Or::new(4, 4),
            load_store_instr_not: Not::new(8),
            load_store_instr_decoder: VariableDecoder::new(3),
            load_store_instr_0_top_and: And::new(2, 2),
            load_store_instr_1_and: And::new(2, 2),
            load_store_instr_2_and: And::new(2, 3),
            load_store_instr_3_and: And::new(2, 1),
            load_store_instr_4_and: And::new(2, 2),
            load_store_instr_5_and: And::new(2, 3),
            load_store_instr_6_and: And::new(2, 1),
            load_store_instr_7_and: And::new(2, 2),
            stepper_out_4_top_0_and: And::new(2, 2),
            stepper_out_4_1_and: And::new(2, 2),
            stepper_out_4_2_and: And::new(2, 2),
            stepper_out_4_3_and: And::new(2, 4),
            stepper_out_4_4_and: And::new(2, 2),
            stepper_out_4_5_and: And::new(2, 2),
            stepper_out_4_6_and: And::new(2, 4),
            stepper_out_4_7_and: And::new(2, 2),
            stepper_out_4_8_and: And::new(3, 2),
            stepper_out_5_top_0_and: And::new(2, 4),
            stepper_out_5_1_and: And::new(2, 2),
            stepper_out_5_2_and: And::new(2, 2),
            stepper_out_5_3_and: And::new(2, 2),
            stepper_out_5_4_and: And::new(2, 2),
            stepper_out_5_5_and: And::new(2, 2),
            stepper_out_5_6_and: And::new(3, 2),
            stepper_out_5_6_not: Not::new(1),
            stepper_out_6_top_0_and: And::new(3, 2),
            stepper_out_6_1_and: And::new(2, 2),
            stepper_out_6_2_and: And::new(3, 2),
            eight_input_and: And::new(8, 1),
            eight_input_and_not_loc_2: Not::new(1),
            eight_input_and_not_loc_3: Not::new(1),
            c_in_and: And::new(2, 1),
            a_l_and: And::new(2, 1),
            eq_and: And::new(2, 1),
            z_and: And::new(2, 1),
            alu_input_or: Or::new(4, 1),
            add_and: And::new(3, 1),
            add_not: Not::new(1),
        };

        control_section.clk_and.lock().unwrap().set_tag("clk_and");
        control_section.load_not.lock().unwrap().set_tag("load_not");
        control_section.reset_not.lock().unwrap().set_tag("reset_not");
        control_section.stepper.lock().unwrap().set_tag("stepper");
        control_section.stepper_splitters[0].lock().unwrap().set_tag("stepper_splitters[0]");
        control_section.stepper_splitters[1].lock().unwrap().set_tag("stepper_splitters[1]");
        control_section.stepper_splitters[2].lock().unwrap().set_tag("stepper_splitters[2]");
        control_section.stepper_splitters[3].lock().unwrap().set_tag("stepper_splitters[3]");
        control_section.stepper_splitters[4].lock().unwrap().set_tag("stepper_splitters[4]");
        control_section.stepper_splitters[5].lock().unwrap().set_tag("stepper_splitters[5]");
        control_section.stepper_1_and.lock().unwrap().set_tag("stepper_1_and");
        control_section.bus_1_or.lock().unwrap().set_tag("bus_1_or");
        control_section.ram_e_and.lock().unwrap().set_tag("ram_e_and");
        control_section.ram_e_or.lock().unwrap().set_tag("ram_e_or");
        control_section.acc_e_and.lock().unwrap().set_tag("acc_e_and");
        control_section.acc_e_or.lock().unwrap().set_tag("acc_e_or");
        control_section.iar_e_and.lock().unwrap().set_tag("iar_e_and");
        control_section.iar_e_or.lock().unwrap().set_tag("iar_e_or");
        control_section.io_clk_e_and.lock().unwrap().set_tag("io_clk_e_and");
        control_section.io_clks_s_and.lock().unwrap().set_tag("io_clks_s_and");
        control_section.r0_e_or.lock().unwrap().set_tag("r0_e_or");
        control_section.r0_e_reg_b_and.lock().unwrap().set_tag("r0_e_reg_b_and");
        control_section.r0_e_reg_a_and.lock().unwrap().set_tag("r0_e_reg_a_and");
        control_section.r1_e_or.lock().unwrap().set_tag("r1_e_or");
        control_section.r1_e_reg_b_and.lock().unwrap().set_tag("r1_e_reg_b_and");
        control_section.r1_e_reg_a_and.lock().unwrap().set_tag("r1_e_reg_a_and");
        control_section.r2_e_or.lock().unwrap().set_tag("r2_e_or");
        control_section.r2_e_reg_b_and.lock().unwrap().set_tag("r2_e_reg_b_and");
        control_section.r2_e_reg_a_and.lock().unwrap().set_tag("r2_e_reg_a_and");
        control_section.r3_e_or.lock().unwrap().set_tag("r3_e_or");
        control_section.r3_e_reg_b_and.lock().unwrap().set_tag("r3_e_reg_b_and");
        control_section.r3_e_reg_a_and.lock().unwrap().set_tag("r3_e_reg_a_and");
        control_section.r_e_reg_b_decoder.lock().unwrap().set_tag("r_e_reg_b_decoder");
        control_section.r_e_reg_a_decoder.lock().unwrap().set_tag("r_e_reg_a_decoder");
        control_section.mar_s_or.lock().unwrap().set_tag("mar_s_or");
        control_section.mar_s_and.lock().unwrap().set_tag("mar_s_and");
        control_section.mar_s_outer_or.lock().unwrap().set_tag("mar_s_outer_or");
        control_section.ram_s_or.lock().unwrap().set_tag("ram_s_or");
        control_section.ram_s_load_and.lock().unwrap().set_tag("ram_s_load_and");
        control_section.ram_s_and.lock().unwrap().set_tag("ram_s_and");
        control_section.acc_s_or.lock().unwrap().set_tag("acc_s_or");
        control_section.acc_s_and.lock().unwrap().set_tag("acc_s_and");
        control_section.acc_s_outer_or.lock().unwrap().set_tag("acc_s_outer_or");
        control_section.iar_s_or.lock().unwrap().set_tag("iar_s_or");
        control_section.iar_s_and.lock().unwrap().set_tag("iar_s_and");
        control_section.iar_s_outer_or.lock().unwrap().set_tag("iar_s_outer_or");
        control_section.r0_s_or.lock().unwrap().set_tag("r0_s_or");
        control_section.r0_s_and.lock().unwrap().set_tag("r0_s_and");
        control_section.r1_s_or.lock().unwrap().set_tag("r1_s_or");
        control_section.r1_s_and.lock().unwrap().set_tag("r1_s_and");
        control_section.r2_s_or.lock().unwrap().set_tag("r2_s_or");
        control_section.r2_s_and.lock().unwrap().set_tag("r2_s_and");
        control_section.r3_s_or.lock().unwrap().set_tag("r3_s_or");
        control_section.r3_s_and.lock().unwrap().set_tag("r3_s_and");
        control_section.r_s_decoder.lock().unwrap().set_tag("r_s_decoder");
        control_section.ir_s_or.lock().unwrap().set_tag("ir_s_or");
        control_section.ir_s_and.lock().unwrap().set_tag("ir_s_and");
        control_section.tmp_s_or.lock().unwrap().set_tag("tmp_s_or");
        control_section.tmp_s_and.lock().unwrap().set_tag("tmp_s_and");
        control_section.reg_b_e_or.lock().unwrap().set_tag("reg_b_e_or");
        control_section.reg_a_or.lock().unwrap().set_tag("reg_a_or");
        control_section.alu_0_and.lock().unwrap().set_tag("alu_0_and");
        control_section.alu_1_and.lock().unwrap().set_tag("alu_1_and");
        control_section.alu_2_and.lock().unwrap().set_tag("alu_2_and");
        control_section.flags_s_or.lock().unwrap().set_tag("flags_s_or");
        control_section.flags_s_and.lock().unwrap().set_tag("flags_s_and");
        control_section.flags_s_outer_or.lock().unwrap().set_tag("flags_s_outer_or");
        control_section.reg_b_s_or.lock().unwrap().set_tag("reg_b_s_or");
        control_section.load_store_instr_not.lock().unwrap().set_tag("load_store_instr_not");
        control_section.load_store_instr_decoder.lock().unwrap().set_tag("load_store_instr_decoder");
        control_section.load_store_instr_0_top_and.lock().unwrap().set_tag("load_store_instr_0_top_and");
        control_section.load_store_instr_1_and.lock().unwrap().set_tag("load_store_instr_1_and");
        control_section.load_store_instr_2_and.lock().unwrap().set_tag("load_store_instr_2_and");
        control_section.load_store_instr_3_and.lock().unwrap().set_tag("load_store_instr_3_and");
        control_section.load_store_instr_4_and.lock().unwrap().set_tag("load_store_instr_4_and");
        control_section.load_store_instr_5_and.lock().unwrap().set_tag("load_store_instr_5_and");
        control_section.load_store_instr_6_and.lock().unwrap().set_tag("load_store_instr_6_and");
        control_section.load_store_instr_7_and.lock().unwrap().set_tag("load_store_instr_7_and");
        control_section.stepper_out_4_top_0_and.lock().unwrap().set_tag("stepper_out_4_top_0_and");
        control_section.stepper_out_4_1_and.lock().unwrap().set_tag("stepper_out_4_1_and");
        control_section.stepper_out_4_2_and.lock().unwrap().set_tag("stepper_out_4_2_and");
        control_section.stepper_out_4_3_and.lock().unwrap().set_tag("stepper_out_4_3_and");
        control_section.stepper_out_4_4_and.lock().unwrap().set_tag("stepper_out_4_4_and");
        control_section.stepper_out_4_5_and.lock().unwrap().set_tag("stepper_out_4_5_and");
        control_section.stepper_out_4_6_and.lock().unwrap().set_tag("stepper_out_4_6_and");
        control_section.stepper_out_4_7_and.lock().unwrap().set_tag("stepper_out_4_7_and");
        control_section.stepper_out_4_8_and.lock().unwrap().set_tag("stepper_out_4_8_and");
        control_section.stepper_out_5_top_0_and.lock().unwrap().set_tag("stepper_out_5_top_0_and");
        control_section.stepper_out_5_1_and.lock().unwrap().set_tag("stepper_out_5_1_and");
        control_section.stepper_out_5_2_and.lock().unwrap().set_tag("stepper_out_5_2_and");
        control_section.stepper_out_5_3_and.lock().unwrap().set_tag("stepper_out_5_3_and");
        control_section.stepper_out_5_4_and.lock().unwrap().set_tag("stepper_out_5_4_and");
        control_section.stepper_out_5_5_and.lock().unwrap().set_tag("stepper_out_5_5_and");
        control_section.stepper_out_5_6_and.lock().unwrap().set_tag("stepper_out_5_6_and");
        control_section.stepper_out_5_6_not.lock().unwrap().set_tag("stepper_out_5_6_not");
        control_section.stepper_out_6_top_0_and.lock().unwrap().set_tag("stepper_out_6_top_0_and");
        control_section.stepper_out_6_1_and.lock().unwrap().set_tag("stepper_out_6_1_and");
        control_section.stepper_out_6_2_and.lock().unwrap().set_tag("stepper_out_6_2_and");
        control_section.eight_input_and.lock().unwrap().set_tag("eight_input_and");
        control_section.eight_input_and_not_loc_2.lock().unwrap().set_tag("eight_input_and_not_loc_2");
        control_section.eight_input_and_not_loc_3.lock().unwrap().set_tag("eight_input_and_not_loc_3");
        control_section.c_in_and.lock().unwrap().set_tag("c_in_and");
        control_section.a_l_and.lock().unwrap().set_tag("a_l_and");
        control_section.eq_and.lock().unwrap().set_tag("eq_and");
        control_section.z_and.lock().unwrap().set_tag("z_and");
        control_section.alu_input_or.lock().unwrap().set_tag("alu_input_or");
        control_section.add_and.lock().unwrap().set_tag("add_and");
        control_section.add_not.lock().unwrap().set_tag("add_not");

        control_section.stepper.lock().unwrap().toggle_print_each_input_output_gate(false);

        control_section.build_and_prime_circuit(output_gates_logic);

        new_shared_mutex(control_section)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {

        //Inputs
        self.connect_clk_input();
        self.connect_clke_input();
        self.connect_clks_input();
        self.connect_high_level_mars_input();
        self.connect_high_level_reset_input();
        self.connect_high_level_load_input();
        self.connect_c_in_input();
        self.connect_a_l_input();
        self.connect_eq_input();
        self.connect_z_input();

        // idx 0 is the least significant bit
        // idx 7 is the most significant bit
        self.connect_ir_0_input();
        self.connect_ir_1_input();
        self.connect_ir_2_input(&output_gates);
        self.connect_ir_3_input(&output_gates);
        self.connect_ir_4_input();
        self.connect_ir_5_input();
        self.connect_ir_6_input();
        self.connect_ir_7_input();

        //Gates
        self.connect_stepper_to_splitter();
        self.clk_and_connect();
        self.load_not_connect();
        self.reset_not_connect();
        self.stepper_splitters_1_connect();
        self.stepper_splitters_2_connect();
        self.stepper_splitters_3_connect();
        self.stepper_splitters_4_connect();
        self.stepper_splitters_5_connect();
        self.stepper_splitters_6_connect();
        self.stepper_1_and_connect();
        self.bus_1_or_connect(&output_gates);
        self.ram_e_and_connect(&output_gates);
        self.ram_e_or_connect();
        self.acc_e_and_connect(&output_gates);
        self.acc_e_or_connect();
        self.iar_e_and_connect(&output_gates);
        self.iar_e_or_connect();
        self.io_clk_e_and_connect(&output_gates);
        self.io_clks_s_and_connect(&output_gates);
        self.r0_e_or_connect(&output_gates);
        self.r0_e_reg_b_and_connect();
        self.r0_e_reg_a_and_connect();
        self.r1_e_or_connect(&output_gates);
        self.r1_e_reg_b_and_connect();
        self.r1_e_reg_a_and_connect();
        self.r2_e_or_connect(&output_gates);
        self.r2_e_reg_b_and_connect();
        self.r2_e_reg_a_and_connect();
        self.r3_e_or_connect(&output_gates);
        self.r3_e_reg_b_and_connect();
        self.r3_e_reg_a_and_connect();
        self.r_e_reg_b_decoder_connect();
        self.r_e_reg_a_decoder_connect();
        self.mar_s_or_connect(&output_gates);
        self.mar_s_and_connect();
        self.mar_s_outer_or_connect();
        self.ram_s_or_connect(&output_gates);
        self.ram_s_load_and_connect();
        self.ram_s_and_connect();
        self.acc_s_or_connect(&output_gates);
        self.acc_s_and_connect();
        self.acc_s_outer_or_connect();
        self.iar_s_or_connect(&output_gates);
        self.iar_s_and_connect();
        self.iar_s_outer_or_connect();
        self.r0_s_or_connect(&output_gates);
        self.r0_s_and_connect();
        self.r1_s_or_connect(&output_gates);
        self.r1_s_and_connect();
        self.r2_s_or_connect(&output_gates);
        self.r2_s_and_connect();
        self.r3_s_or_connect(&output_gates);
        self.r3_s_and_connect();
        self.r_s_decoder_connect();
        self.ir_s_or_connect(&output_gates);
        self.ir_s_and_connect();
        self.tmp_s_or_connect(&output_gates);
        self.tmp_s_and_connect();
        self.reg_b_e_or_connect();
        self.reg_a_or_connect();
        self.alu_0_and_connect(&output_gates);
        self.alu_1_and_connect(&output_gates);
        self.alu_2_and_connect(&output_gates);
        self.flags_s_or_connect(&output_gates);
        self.flags_s_and_connect();
        self.flags_s_outer_or_connect();
        self.reg_b_s_or_connect();
        self.load_store_instr_not_connect();
        self.load_store_instr_decoder_connect();
        self.load_store_instr_0_top_and_connect();
        self.load_store_instr_1_and_connect();
        self.load_store_instr_2_and_connect();
        self.load_store_instr_3_and_connect();
        self.load_store_instr_4_and_connect();
        self.load_store_instr_5_and_connect();
        self.load_store_instr_6_and_connect();
        self.load_store_instr_7_and_connect();
        self.stepper_out_4_top_0_and_connect();
        self.stepper_out_4_1_and_connect();
        self.stepper_out_4_2_and_connect();
        self.stepper_out_4_3_and_connect();
        self.stepper_out_4_4_and_connect();
        self.stepper_out_4_5_and_connect();
        self.stepper_out_4_6_and_connect();
        self.stepper_out_4_7_and_connect();
        self.stepper_out_4_8_and_connect();
        self.stepper_out_5_top_0_and_connect(&output_gates);
        self.stepper_out_5_1_and_connect();
        self.stepper_out_5_2_and_connect();
        self.stepper_out_5_3_and_connect();
        self.stepper_out_5_4_and_connect();
        self.stepper_out_5_5_and_connect();
        self.stepper_out_5_6_and_connect();
        self.stepper_out_5_6_not_connect();
        self.stepper_out_6_top_0_and_connect();
        self.stepper_out_6_1_and_connect();
        self.stepper_out_6_2_and_connect();
        self.eight_input_and_connect(&output_gates);
        self.eight_input_and_not_loc_2_connect();
        self.eight_input_and_not_loc_3_connect();
        self.c_in_and_connect();
        self.a_l_and_connect();
        self.eq_and_connect();
        self.z_and_connect();
        self.alu_input_or_connect();
        self.add_and_connect();
        self.add_not_connect();

        #[cfg(feature = "high_restriction")]
        self.check_output();

        //Prime gates
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

        check_output(&self.clk_and.lock().unwrap().members);
        check_output(&self.load_not.lock().unwrap().members);
        check_output(&self.stepper.lock().unwrap().complex_gate.simple_gate);
        check_output(&self.stepper_splitters[0].lock().unwrap().members);
        check_output(&self.stepper_splitters[1].lock().unwrap().members);
        check_output(&self.stepper_splitters[2].lock().unwrap().members);
        check_output(&self.stepper_splitters[3].lock().unwrap().members);
        check_output(&self.stepper_splitters[4].lock().unwrap().members);
        check_output(&self.stepper_splitters[5].lock().unwrap().members);
        check_output(&self.stepper_1_and.lock().unwrap().members);
        check_output(&self.bus_1_or.lock().unwrap().members);
        check_output(&self.ram_e_and.lock().unwrap().members);
        check_output(&self.ram_e_or.lock().unwrap().members);
        check_output(&self.acc_e_and.lock().unwrap().members);
        check_output(&self.acc_e_or.lock().unwrap().members);
        check_output(&self.iar_e_and.lock().unwrap().members);
        check_output(&self.iar_e_or.lock().unwrap().members);
        check_output(&self.io_clk_e_and.lock().unwrap().members);
        check_output(&self.io_clks_s_and.lock().unwrap().members);
        check_output(&self.r0_e_or.lock().unwrap().members);
        check_output(&self.r0_e_reg_b_and.lock().unwrap().members);
        check_output(&self.r0_e_reg_a_and.lock().unwrap().members);
        check_output(&self.r1_e_or.lock().unwrap().members);
        check_output(&self.r1_e_reg_b_and.lock().unwrap().members);
        check_output(&self.r1_e_reg_a_and.lock().unwrap().members);
        check_output(&self.r2_e_or.lock().unwrap().members);
        check_output(&self.r2_e_reg_b_and.lock().unwrap().members);
        check_output(&self.r2_e_reg_a_and.lock().unwrap().members);
        check_output(&self.r3_e_or.lock().unwrap().members);
        check_output(&self.r3_e_reg_b_and.lock().unwrap().members);
        check_output(&self.r3_e_reg_a_and.lock().unwrap().members);
        check_output(&self.r_e_reg_b_decoder.lock().unwrap().complex_gate.simple_gate);
        check_output(&self.r_e_reg_a_decoder.lock().unwrap().complex_gate.simple_gate);
        check_output(&self.mar_s_or.lock().unwrap().members);
        check_output(&self.mar_s_and.lock().unwrap().members);
        check_output(&self.mar_s_outer_or.lock().unwrap().members);
        check_output(&self.ram_s_or.lock().unwrap().members);
        check_output(&self.ram_s_load_and.lock().unwrap().members);
        check_output(&self.ram_s_and.lock().unwrap().members);
        check_output(&self.acc_s_or.lock().unwrap().members);
        check_output(&self.acc_s_and.lock().unwrap().members);
        check_output(&self.acc_s_outer_or.lock().unwrap().members);
        check_output(&self.iar_s_or.lock().unwrap().members);
        check_output(&self.iar_s_and.lock().unwrap().members);
        check_output(&self.iar_s_outer_or.lock().unwrap().members);
        check_output(&self.r0_s_or.lock().unwrap().members);
        check_output(&self.r0_s_and.lock().unwrap().members);
        check_output(&self.r1_s_or.lock().unwrap().members);
        check_output(&self.r1_s_and.lock().unwrap().members);
        check_output(&self.r2_s_or.lock().unwrap().members);
        check_output(&self.r2_s_and.lock().unwrap().members);
        check_output(&self.r3_s_or.lock().unwrap().members);
        check_output(&self.r3_s_and.lock().unwrap().members);
        check_output(&self.r_s_decoder.lock().unwrap().complex_gate.simple_gate);
        check_output(&self.ir_s_or.lock().unwrap().members);
        check_output(&self.ir_s_and.lock().unwrap().members);
        check_output(&self.tmp_s_or.lock().unwrap().members);
        check_output(&self.tmp_s_and.lock().unwrap().members);
        check_output(&self.reg_b_e_or.lock().unwrap().members);
        check_output(&self.reg_a_or.lock().unwrap().members);
        check_output(&self.alu_0_and.lock().unwrap().members);
        check_output(&self.alu_1_and.lock().unwrap().members);
        check_output(&self.alu_2_and.lock().unwrap().members);
        check_output(&self.flags_s_or.lock().unwrap().members);
        check_output(&self.flags_s_and.lock().unwrap().members);
        check_output(&self.flags_s_outer_or.lock().unwrap().members);
        check_output(&self.reg_b_s_or.lock().unwrap().members);
        check_output(&self.load_store_instr_not.lock().unwrap().members);
        check_output(&self.load_store_instr_decoder.lock().unwrap().complex_gate.simple_gate);
        check_output(&self.load_store_instr_0_top_and.lock().unwrap().members);
        check_output(&self.load_store_instr_1_and.lock().unwrap().members);
        check_output(&self.load_store_instr_2_and.lock().unwrap().members);
        check_output(&self.load_store_instr_3_and.lock().unwrap().members);
        check_output(&self.load_store_instr_4_and.lock().unwrap().members);
        check_output(&self.load_store_instr_5_and.lock().unwrap().members);
        check_output(&self.load_store_instr_6_and.lock().unwrap().members);
        check_output(&self.load_store_instr_7_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_top_0_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_1_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_2_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_3_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_4_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_5_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_6_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_7_and.lock().unwrap().members);
        check_output(&self.stepper_out_4_8_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_top_0_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_1_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_2_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_3_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_4_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_5_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_6_and.lock().unwrap().members);
        check_output(&self.stepper_out_5_6_not.lock().unwrap().members);
        check_output(&self.stepper_out_6_top_0_and.lock().unwrap().members);
        check_output(&self.stepper_out_6_1_and.lock().unwrap().members);
        check_output(&self.stepper_out_6_2_and.lock().unwrap().members);
        check_output(&self.eight_input_and.lock().unwrap().members);
        check_output(&self.eight_input_and_not_loc_2.lock().unwrap().members);
        check_output(&self.eight_input_and_not_loc_3.lock().unwrap().members);
        check_output(&self.c_in_and.lock().unwrap().members);
        check_output(&self.a_l_and.lock().unwrap().members);
        check_output(&self.eq_and.lock().unwrap().members);
        check_output(&self.z_and.lock().unwrap().members);
        check_output(&self.alu_input_or.lock().unwrap().members);
        check_output(&self.add_and.lock().unwrap().members);
        check_output(&self.add_not.lock().unwrap().members);
    }

    fn connect_stepper_to_splitter(&mut self) {
        for i in 0..self.stepper_splitters.len() {
            connect_gates(
                self.stepper.clone(),
                i,
                self.stepper_splitters[i].clone(),
                0,
            );
        }
    }

    fn connect_clk_input(&mut self) {
        let clk_input = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::CLOCK)
            ].clone();

        connect_gates(
            clk_input.clone(),
            0,
            self.clk_and.clone(),
            0,
        );
    }

    fn connect_clke_input(&mut self) {
        let clke_input = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::CLOCK_ENABLE)
            ].clone();

        connect_gates(
            clke_input.clone(),
            0,
            self.iar_e_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            1,
            self.ram_e_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            2,
            self.acc_e_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            3,
            self.io_clk_e_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            4,
            self.r0_e_reg_b_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            5,
            self.r1_e_reg_b_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            6,
            self.r2_e_reg_b_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            7,
            self.r3_e_reg_b_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            8,
            self.r0_e_reg_a_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            9,
            self.r1_e_reg_a_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            10,
            self.r2_e_reg_a_and.clone(),
            0,
        );

        connect_gates(
            clke_input.clone(),
            11,
            self.r3_e_reg_a_and.clone(),
            0,
        );
    }

    fn connect_clks_input(&mut self) {
        let clks_input = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::CLOCK_SET)
            ].clone();

        connect_gates(
            clks_input.clone(),
            0,
            self.ir_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            1,
            self.mar_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            2,
            self.iar_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            3,
            self.acc_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            4,
            self.ram_s_load_and.clone(),
            1,
        );

        connect_gates(
            clks_input.clone(),
            5,
            self.ram_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            6,
            self.tmp_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            7,
            self.flags_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            8,
            self.io_clks_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            9,
            self.r0_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            10,
            self.r1_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            11,
            self.r2_s_and.clone(),
            0,
        );

        connect_gates(
            clks_input.clone(),
            12,
            self.r3_s_and.clone(),
            0,
        );
    }

    fn connect_high_level_mars_input(&mut self) {
        let high_level_mars = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::HIGH_LVL_MARS)
            ].clone();

        connect_gates(
            high_level_mars.clone(),
            0,
            self.mar_s_or.clone(),
            0,
        );
    }

    fn connect_high_level_reset_input(&mut self) {
        let high_level_reset = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::HIGH_LVL_RESET)
            ].clone();

        connect_gates(
            high_level_reset.clone(),
            0,
            self.ir_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            1,
            self.mar_s_or.clone(),
            1,
        );

        connect_gates(
            high_level_reset.clone(),
            2,
            self.iar_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            3,
            self.acc_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            4,
            self.tmp_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            5,
            self.flags_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            6,
            self.r0_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            7,
            self.r1_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            8,
            self.r2_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            9,
            self.r3_s_or.clone(),
            0,
        );

        connect_gates(
            high_level_reset.clone(),
            10,
            self.reset_not.clone(),
            0,
        );
    }

    fn connect_high_level_load_input(&mut self) {
        let high_level_load = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::HIGH_LVL_LOAD)
            ].clone();

        connect_gates(
            high_level_load.clone(),
            0,
            self.load_not.clone(),
            0,
        );

        connect_gates(
            high_level_load.clone(),
            1,
            self.ram_s_load_and.clone(),
            0,
        );
    }

    fn connect_c_in_input(&mut self) {
        let c_in_input = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::C_IN)
            ].clone();

        connect_gates(
            c_in_input.clone(),
            0,
            self.c_in_and.clone(),
            0,
        );
    }

    fn connect_a_l_input(&mut self) {
        let a_l_input = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::A_L)
            ].clone();

        connect_gates(
            a_l_input.clone(),
            0,
            self.a_l_and.clone(),
            0,
        );
    }

    fn connect_eq_input(&mut self) {
        let eq_input = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::EQ)
            ].clone();

        connect_gates(
            eq_input.clone(),
            0,
            self.eq_and.clone(),
            0,
        );
    }

    fn connect_z_input(&mut self) {
        let z_input = self.complex_gate.input_gates[
            self.get_index_from_tag(ControlSection::Z)
            ].clone();

        connect_gates(
            z_input.clone(),
            0,
            self.z_and.clone(),
            0,
        );
    }

    fn connect_ir_0_input(&mut self) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_0")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.z_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            1,
            self.eight_input_and.clone(),
            7,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.r_e_reg_b_decoder.clone(),
            0,
        );

        connect_gates(
            input_gate.clone(),
            3,
            self.r_s_decoder.clone(),
            0,
        );
    }

    fn connect_ir_1_input(&mut self) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_1")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.eq_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            1,
            self.eight_input_and.clone(),
            6,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.r_e_reg_b_decoder.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            3,
            self.r_s_decoder.clone(),
            1,
        );
    }

    fn connect_ir_2_input(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_2")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.a_l_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            1,
            self.eight_input_and.clone(),
            5,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.r_e_reg_a_decoder.clone(),
            0,
        );

        let io_index = self.get_index_from_tag(ControlSection::IO);
        connect_gates(
            input_gate.clone(),
            3,
            output_gates[io_index].clone(),
            0,
        );
    }

    fn connect_ir_3_input(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_3")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.c_in_and.clone(),
            1,
        );

        let da_index = self.get_index_from_tag(ControlSection::DA);
        connect_gates(
            input_gate.clone(),
            1,
            output_gates[da_index].clone(),
            0,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.eight_input_and.clone(),
            4,
        );

        connect_gates(
            input_gate.clone(),
            3,
            self.r_e_reg_a_decoder.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            4,
            self.stepper_out_4_8_and.clone(),
            2,
        );

        connect_gates(
            input_gate.clone(),
            5,
            self.stepper_out_5_6_not.clone(),
            0,
        );
    }

    fn connect_ir_4_input(&mut self) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_4")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.alu_0_and.clone(),
            2,
        );

        connect_gates(
            input_gate.clone(),
            1,
            self.eight_input_and_not_loc_3.clone(),
            0,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.load_store_instr_decoder.clone(),
            0,
        );

        connect_gates(
            input_gate.clone(),
            3,
            self.add_and.clone(),
            0,
        );
    }

    fn connect_ir_5_input(&mut self) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_5")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.alu_1_and.clone(),
            2,
        );

        connect_gates(
            input_gate.clone(),
            1,
            self.eight_input_and_not_loc_2.clone(),
            0,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.load_store_instr_decoder.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            3,
            self.add_and.clone(),
            1,
        );
    }

    fn connect_ir_6_input(&mut self) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_6")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.alu_2_and.clone(),
            2,
        );

        connect_gates(
            input_gate.clone(),
            1,
            self.eight_input_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.load_store_instr_decoder.clone(),
            2,
        );

        connect_gates(
            input_gate.clone(),
            3,
            self.add_and.clone(),
            2,
        );
    }

    fn connect_ir_7_input(&mut self) {
        let input_gate = self.complex_gate.input_gates[self.get_index_from_tag("IR_7")].clone();

        connect_gates(
            input_gate.clone(),
            0,
            self.alu_0_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            1,
            self.alu_1_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            2,
            self.alu_2_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            3,
            self.eight_input_and.clone(),
            0,
        );

        connect_gates(
            input_gate.clone(),
            4,
            self.load_store_instr_not.clone(),
            0,
        );

        connect_gates(
            input_gate.clone(),
            5,
            self.stepper_out_4_top_0_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            6,
            self.stepper_out_5_top_0_and.clone(),
            1,
        );

        connect_gates(
            input_gate.clone(),
            7,
            self.stepper_out_6_top_0_and.clone(),
            1,
        );
    }

    fn clk_and_connect(&mut self) {
        let clk_index = self.stepper.lock().unwrap().get_index_from_tag("CLK");
        connect_gates(
            self.clk_and.clone(),
            0,
            self.stepper.clone(),
            clk_index,
        );
    }

    fn load_not_connect(&mut self) {
        connect_gates(
            self.load_not.clone(),
            0,
            self.clk_and.clone(),
            1,
        );

        connect_gates(
            self.load_not.clone(),
            1,
            self.stepper_1_and.clone(),
            0,
        );
    }

    fn reset_not_connect(&mut self) {
        connect_gates(
            self.reset_not.clone(),
            0,
            self.clk_and.clone(),
            2,
        );

        connect_gates(
            self.reset_not.clone(),
            1,
            self.stepper_1_and.clone(),
            2,
        );
    }

    fn stepper_splitters_1_connect(&mut self) {
        connect_gates(
            self.stepper_splitters[0].clone(),
            0,
            self.stepper_1_and.clone(),
            1,
        );
    }

    fn stepper_splitters_2_connect(&mut self) {
        connect_gates(
            self.stepper_splitters[1].clone(),
            0,
            self.ir_s_and.clone(),
            1,
        );

        connect_gates(
            self.stepper_splitters[1].clone(),
            1,
            self.ram_e_or.clone(),
            0,
        );
    }

    fn stepper_splitters_3_connect(&mut self) {
        connect_gates(
            self.stepper_splitters[2].clone(),
            0,
            self.iar_s_outer_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[2].clone(),
            1,
            self.acc_e_or.clone(),
            0,
        );
    }

    fn stepper_splitters_4_connect(&mut self) {
        connect_gates(
            self.stepper_splitters[3].clone(),
            0,
            self.stepper_out_4_top_0_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            1,
            self.stepper_out_4_1_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            2,
            self.stepper_out_4_2_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            3,
            self.stepper_out_4_3_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            4,
            self.stepper_out_4_4_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            5,
            self.stepper_out_4_5_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            6,
            self.stepper_out_4_6_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            7,
            self.stepper_out_4_7_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[3].clone(),
            8,
            self.stepper_out_4_8_and.clone(),
            0,
        );
    }

    fn stepper_splitters_5_connect(&mut self) {
        connect_gates(
            self.stepper_splitters[4].clone(),
            0,
            self.stepper_out_5_top_0_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            1,
            self.stepper_out_5_1_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            2,
            self.stepper_out_5_2_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            3,
            self.stepper_out_5_3_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            4,
            self.stepper_out_5_4_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            5,
            self.stepper_out_5_5_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            6,
            self.stepper_out_5_6_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            7,
            self.alu_0_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            8,
            self.alu_1_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[4].clone(),
            9,
            self.alu_2_and.clone(),
            0,
        );
    }

    fn stepper_splitters_6_connect(&mut self) {
        connect_gates(
            self.stepper_splitters[5].clone(),
            0,
            self.stepper_out_6_top_0_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[5].clone(),
            1,
            self.stepper_out_6_1_and.clone(),
            0,
        );

        connect_gates(
            self.stepper_splitters[5].clone(),
            2,
            self.stepper_out_6_2_and.clone(),
            0,
        );
    }

    fn stepper_1_and_connect(&mut self) {
        connect_gates(
            self.stepper_1_and.clone(),
            0,
            self.bus_1_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_1_and.clone(),
            1,
            self.mar_s_outer_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_1_and.clone(),
            2,
            self.iar_e_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_1_and.clone(),
            3,
            self.acc_s_outer_or.clone(),
            0,
        );
    }

    fn bus_1_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let bus_1_index = self.get_index_from_tag(ControlSection::BUS_1);
        connect_gates(
            self.bus_1_or.clone(),
            0,
            output_gates[bus_1_index].clone(),
            0,
        );
    }

    fn ram_e_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let ram_e_index = self.get_index_from_tag(ControlSection::RAM_E);
        connect_gates(
            self.ram_e_and.clone(),
            0,
            output_gates[ram_e_index].clone(),
            0,
        );
    }

    fn ram_e_or_connect(&mut self) {
        connect_gates(
            self.ram_e_or.clone(),
            0,
            self.ram_e_and.clone(),
            1,
        );
    }

    fn acc_e_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let acc_e_index = self.get_index_from_tag(ControlSection::ACC_E);
        connect_gates(
            self.acc_e_and.clone(),
            0,
            output_gates[acc_e_index].clone(),
            0,
        );
    }

    fn acc_e_or_connect(&mut self) {
        connect_gates(
            self.acc_e_or.clone(),
            0,
            self.acc_e_and.clone(),
            1,
        );
    }

    fn iar_e_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let iar_e_index = self.get_index_from_tag(ControlSection::IAR_E);
        connect_gates(
            self.iar_e_and.clone(),
            0,
            output_gates[iar_e_index].clone(),
            0,
        );
    }

    fn iar_e_or_connect(&mut self) {
        connect_gates(
            self.iar_e_or.clone(),
            0,
            self.iar_e_and.clone(),
            1,
        );
    }

    fn io_clk_e_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let io_clk_e_index = self.get_index_from_tag(ControlSection::IO_CLK_E);
        connect_gates(
            self.io_clk_e_and.clone(),
            0,
            output_gates[io_clk_e_index].clone(),
            0,
        );
    }

    fn io_clks_s_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let io_clk_s_index = self.get_index_from_tag(ControlSection::IO_CLK_S);
        connect_gates(
            self.io_clks_s_and.clone(),
            0,
            output_gates[io_clk_s_index].clone(),
            0,
        );
    }

    fn r0_e_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r0_e_index = self.get_index_from_tag(ControlSection::R0_E);
        connect_gates(
            self.r0_e_or.clone(),
            0,
            output_gates[r0_e_index].clone(),
            0,
        );
    }

    fn r0_e_reg_b_and_connect(&mut self) {
        connect_gates(
            self.r0_e_reg_b_and.clone(),
            0,
            self.r0_e_or.clone(),
            0,
        );
    }

    fn r0_e_reg_a_and_connect(&mut self) {
        connect_gates(
            self.r0_e_reg_a_and.clone(),
            0,
            self.r0_e_or.clone(),
            1,
        );
    }

    fn r1_e_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r1_e_index = self.get_index_from_tag(ControlSection::R1_E);
        connect_gates(
            self.r1_e_or.clone(),
            0,
            output_gates[r1_e_index].clone(),
            0,
        );
    }

    fn r1_e_reg_b_and_connect(&mut self) {
        connect_gates(
            self.r1_e_reg_b_and.clone(),
            0,
            self.r1_e_or.clone(),
            0,
        );
    }

    fn r1_e_reg_a_and_connect(&mut self) {
        connect_gates(
            self.r1_e_reg_a_and.clone(),
            0,
            self.r1_e_or.clone(),
            1,
        );
    }

    fn r2_e_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r2_e_index = self.get_index_from_tag(ControlSection::R2_E);
        connect_gates(
            self.r2_e_or.clone(),
            0,
            output_gates[r2_e_index].clone(),
            0,
        );
    }

    fn r2_e_reg_b_and_connect(&mut self) {
        connect_gates(
            self.r2_e_reg_b_and.clone(),
            0,
            self.r2_e_or.clone(),
            0,
        );
    }

    fn r2_e_reg_a_and_connect(&mut self) {
        connect_gates(
            self.r2_e_reg_a_and.clone(),
            0,
            self.r2_e_or.clone(),
            1,
        );
    }

    fn r3_e_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r3_e_index = self.get_index_from_tag(ControlSection::R3_E);
        connect_gates(
            self.r3_e_or.clone(),
            0,
            output_gates[r3_e_index].clone(),
            0,
        );
    }

    fn r3_e_reg_b_and_connect(&mut self) {
        connect_gates(
            self.r3_e_reg_b_and.clone(),
            0,
            self.r3_e_or.clone(),
            0,
        );
    }

    fn r3_e_reg_a_and_connect(&mut self) {
        connect_gates(
            self.r3_e_reg_a_and.clone(),
            0,
            self.r3_e_or.clone(),
            1,
        );
    }

    fn r_e_reg_b_decoder_connect(&mut self) {
        connect_gates(
            self.r_e_reg_b_decoder.clone(),
            0,
            self.r0_e_reg_b_and.clone(),
            2,
        );

        connect_gates(
            self.r_e_reg_b_decoder.clone(),
            1,
            self.r1_e_reg_b_and.clone(),
            2,
        );

        connect_gates(
            self.r_e_reg_b_decoder.clone(),
            2,
            self.r2_e_reg_b_and.clone(),
            2,
        );

        connect_gates(
            self.r_e_reg_b_decoder.clone(),
            3,
            self.r3_e_reg_b_and.clone(),
            2,
        );
    }

    fn r_e_reg_a_decoder_connect(&mut self) {
        connect_gates(
            self.r_e_reg_a_decoder.clone(),
            0,
            self.r0_e_reg_a_and.clone(),
            2,
        );

        connect_gates(
            self.r_e_reg_a_decoder.clone(),
            1,
            self.r1_e_reg_a_and.clone(),
            2,
        );

        connect_gates(
            self.r_e_reg_a_decoder.clone(),
            2,
            self.r2_e_reg_a_and.clone(),
            2,
        );

        connect_gates(
            self.r_e_reg_a_decoder.clone(),
            3,
            self.r3_e_reg_a_and.clone(),
            2,
        );
    }

    fn mar_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let mar_s_index = self.get_index_from_tag(ControlSection::MAR_S);
        connect_gates(
            self.mar_s_or.clone(),
            0,
            output_gates[mar_s_index].clone(),
            0,
        );
    }

    fn mar_s_and_connect(&mut self) {
        connect_gates(
            self.mar_s_and.clone(),
            0,
            self.mar_s_or.clone(),
            2,
        );
    }

    fn mar_s_outer_or_connect(&mut self) {
        connect_gates(
            self.mar_s_outer_or.clone(),
            0,
            self.mar_s_and.clone(),
            1,
        );
    }

    fn ram_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let ram_s_index = self.get_index_from_tag(ControlSection::RAM_S);
        connect_gates(
            self.ram_s_or.clone(),
            0,
            output_gates[ram_s_index].clone(),
            0,
        );
    }

    fn ram_s_load_and_connect(&mut self) {
        connect_gates(
            self.ram_s_load_and.clone(),
            0,
            self.ram_s_or.clone(),
            0,
        );
    }

    fn ram_s_and_connect(&mut self) {
        connect_gates(
            self.ram_s_and.clone(),
            0,
            self.ram_s_or.clone(),
            1,
        );
    }

    fn acc_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let acc_s_index = self.get_index_from_tag(ControlSection::ACC_S);
        connect_gates(
            self.acc_s_or.clone(),
            0,
            output_gates[acc_s_index].clone(),
            0,
        );
    }

    fn acc_s_and_connect(&mut self) {
        connect_gates(
            self.acc_s_and.clone(),
            0,
            self.acc_s_or.clone(),
            1,
        );
    }

    fn acc_s_outer_or_connect(&mut self) {
        connect_gates(
            self.acc_s_outer_or.clone(),
            0,
            self.acc_s_and.clone(),
            1,
        );
    }

    fn iar_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let ias_s_index = self.get_index_from_tag(ControlSection::IAR_S);
        connect_gates(
            self.iar_s_or.clone(),
            0,
            output_gates[ias_s_index].clone(),
            0,
        );
    }

    fn iar_s_and_connect(&mut self) {
        connect_gates(
            self.iar_s_and.clone(),
            0,
            self.iar_s_or.clone(),
            1,
        );
    }

    fn iar_s_outer_or_connect(&mut self) {
        connect_gates(
            self.iar_s_outer_or.clone(),
            0,
            self.iar_s_and.clone(),
            1,
        );
    }

    fn r0_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r0_s_index = self.get_index_from_tag(ControlSection::R0_S);
        connect_gates(
            self.r0_s_or.clone(),
            0,
            output_gates[r0_s_index].clone(),
            0,
        );
    }

    fn r0_s_and_connect(&mut self) {
        connect_gates(
            self.r0_s_and.clone(),
            0,
            self.r0_s_or.clone(),
            1,
        );
    }

    fn r1_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r1_s_index = self.get_index_from_tag(ControlSection::R1_S);
        connect_gates(
            self.r1_s_or.clone(),
            0,
            output_gates[r1_s_index].clone(),
            0,
        );
    }

    fn r1_s_and_connect(&mut self) {
        connect_gates(
            self.r1_s_and.clone(),
            0,
            self.r1_s_or.clone(),
            1,
        );
    }

    fn r2_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r2_s_index = self.get_index_from_tag(ControlSection::R2_S);
        connect_gates(
            self.r2_s_or.clone(),
            0,
            output_gates[r2_s_index].clone(),
            0,
        );
    }

    fn r2_s_and_connect(&mut self) {
        connect_gates(
            self.r2_s_and.clone(),
            0,
            self.r2_s_or.clone(),
            1,
        );
    }

    fn r3_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let r2_s_index = self.get_index_from_tag(ControlSection::R3_S);
        connect_gates(
            self.r3_s_or.clone(),
            0,
            output_gates[r2_s_index].clone(),
            0,
        );
    }

    fn r3_s_and_connect(&mut self) {
        connect_gates(
            self.r3_s_and.clone(),
            0,
            self.r3_s_or.clone(),
            1,
        );
    }

    fn r_s_decoder_connect(&mut self) {
        connect_gates(
            self.r_s_decoder.clone(),
            0,
            self.r0_s_and.clone(),
            2,
        );

        connect_gates(
            self.r_s_decoder.clone(),
            1,
            self.r1_s_and.clone(),
            2,
        );

        connect_gates(
            self.r_s_decoder.clone(),
            2,
            self.r2_s_and.clone(),
            2,
        );

        connect_gates(
            self.r_s_decoder.clone(),
            3,
            self.r3_s_and.clone(),
            2,
        );
    }

    fn ir_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let ir_s_index = self.get_index_from_tag(ControlSection::IR_S);
        connect_gates(
            self.ir_s_or.clone(),
            0,
            output_gates[ir_s_index].clone(),
            0,
        );
    }

    fn ir_s_and_connect(&mut self) {
        connect_gates(
            self.ir_s_and.clone(),
            0,
            self.ir_s_or.clone(),
            1,
        );
    }

    fn tmp_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let tmp_s_index = self.get_index_from_tag(ControlSection::TMP_S);
        connect_gates(
            self.tmp_s_or.clone(),
            0,
            output_gates[tmp_s_index].clone(),
            0,
        );
    }

    fn tmp_s_and_connect(&mut self) {
        connect_gates(
            self.tmp_s_and.clone(),
            0,
            self.tmp_s_or.clone(),
            1,
        );
    }

    fn reg_b_e_or_connect(&mut self) {
        connect_gates(
            self.reg_b_e_or.clone(),
            0,
            self.r0_e_reg_b_and.clone(),
            1,
        );

        connect_gates(
            self.reg_b_e_or.clone(),
            1,
            self.r1_e_reg_b_and.clone(),
            1,
        );

        connect_gates(
            self.reg_b_e_or.clone(),
            2,
            self.r2_e_reg_b_and.clone(),
            1,
        );

        connect_gates(
            self.reg_b_e_or.clone(),
            3,
            self.r3_e_reg_b_and.clone(),
            1,
        );
    }

    fn reg_a_or_connect(&mut self) {
        connect_gates(
            self.reg_a_or.clone(),
            0,
            self.r0_e_reg_a_and.clone(),
            1,
        );

        connect_gates(
            self.reg_a_or.clone(),
            1,
            self.r1_e_reg_a_and.clone(),
            1,
        );

        connect_gates(
            self.reg_a_or.clone(),
            2,
            self.r2_e_reg_a_and.clone(),
            1,
        );

        connect_gates(
            self.reg_a_or.clone(),
            3,
            self.r3_e_reg_a_and.clone(),
            1,
        );
    }

    fn alu_0_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let alu_0_index = self.get_index_from_tag(ControlSection::ALU_0);
        connect_gates(
            self.alu_0_and.clone(),
            0,
            output_gates[alu_0_index].clone(),
            0,
        );
    }

    fn alu_1_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let alu_1_index = self.get_index_from_tag(ControlSection::ALU_1);
        connect_gates(
            self.alu_1_and.clone(),
            0,
            output_gates[alu_1_index].clone(),
            0,
        );
    }

    fn alu_2_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let alu_2_index = self.get_index_from_tag(ControlSection::ALU_2);
        connect_gates(
            self.alu_2_and.clone(),
            0,
            output_gates[alu_2_index].clone(),
            0,
        );
    }

    fn flags_s_or_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let flag_s_index = self.get_index_from_tag(ControlSection::FLAG_S);
        connect_gates(
            self.flags_s_or.clone(),
            0,
            output_gates[flag_s_index].clone(),
            0,
        );
    }

    fn flags_s_and_connect(&mut self) {
        connect_gates(
            self.flags_s_and.clone(),
            0,
            self.flags_s_or.clone(),
            1,
        );
    }

    fn flags_s_outer_or_connect(&mut self) {
        connect_gates(
            self.flags_s_outer_or.clone(),
            0,
            self.flags_s_and.clone(),
            1,
        );
    }

    fn reg_b_s_or_connect(&mut self) {
        connect_gates(
            self.reg_b_s_or.clone(),
            0,
            self.r0_s_and.clone(),
            1,
        );

        connect_gates(
            self.reg_b_s_or.clone(),
            1,
            self.r1_s_and.clone(),
            1,
        );

        connect_gates(
            self.reg_b_s_or.clone(),
            2,
            self.r2_s_and.clone(),
            1,
        );

        connect_gates(
            self.reg_b_s_or.clone(),
            3,
            self.r3_s_and.clone(),
            1,
        );
    }

    fn load_store_instr_not_connect(&mut self) {
        connect_gates(
            self.load_store_instr_not.clone(),
            0,
            self.load_store_instr_0_top_and.clone(),
            0,
        );

        connect_gates(
            self.load_store_instr_not.clone(),
            1,
            self.load_store_instr_1_and.clone(),
            0,
        );

        connect_gates(
            self.load_store_instr_not.clone(),
            2,
            self.load_store_instr_2_and.clone(),
            0,
        );

        connect_gates(
            self.load_store_instr_not.clone(),
            3,
            self.load_store_instr_3_and.clone(),
            0,
        );

        connect_gates(
            self.load_store_instr_not.clone(),
            4,
            self.load_store_instr_4_and.clone(),
            0,
        );

        connect_gates(
            self.load_store_instr_not.clone(),
            5,
            self.load_store_instr_5_and.clone(),
            0,
        );

        connect_gates(
            self.load_store_instr_not.clone(),
            6,
            self.load_store_instr_6_and.clone(),
            0,
        );

        connect_gates(
            self.load_store_instr_not.clone(),
            7,
            self.load_store_instr_7_and.clone(),
            0,
        );
    }

    fn load_store_instr_decoder_connect(&mut self) {
        connect_gates(
            self.load_store_instr_decoder.clone(),
            0,
            self.load_store_instr_0_top_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_decoder.clone(),
            1,
            self.load_store_instr_1_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_decoder.clone(),
            2,
            self.load_store_instr_2_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_decoder.clone(),
            3,
            self.load_store_instr_3_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_decoder.clone(),
            4,
            self.load_store_instr_4_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_decoder.clone(),
            5,
            self.load_store_instr_5_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_decoder.clone(),
            6,
            self.load_store_instr_6_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_decoder.clone(),
            7,
            self.load_store_instr_7_and.clone(),
            1,
        );
    }

    fn load_store_instr_0_top_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_0_top_and.clone(),
            0,
            self.stepper_out_4_1_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_0_top_and.clone(),
            1,
            self.stepper_out_5_1_and.clone(),
            1,
        );
    }

    fn load_store_instr_1_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_1_and.clone(),
            0,
            self.stepper_out_4_2_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_1_and.clone(),
            1,
            self.stepper_out_5_2_and.clone(),
            1,
        );
    }

    fn load_store_instr_2_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_2_and.clone(),
            0,
            self.stepper_out_4_3_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_2_and.clone(),
            1,
            self.stepper_out_5_3_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_2_and.clone(),
            2,
            self.stepper_out_6_1_and.clone(),
            1,
        );
    }

    fn load_store_instr_3_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_3_and.clone(),
            0,
            self.stepper_out_4_4_and.clone(),
            1,
        );
    }

    fn load_store_instr_4_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_4_and.clone(),
            0,
            self.stepper_out_4_5_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_4_and.clone(),
            1,
            self.stepper_out_5_4_and.clone(),
            1,
        );
    }

    fn load_store_instr_5_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_5_and.clone(),
            0,
            self.stepper_out_4_6_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_5_and.clone(),
            1,
            self.stepper_out_5_5_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_5_and.clone(),
            2,
            self.stepper_out_6_2_and.clone(),
            1,
        );
    }

    fn load_store_instr_6_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_6_and.clone(),
            0,
            self.stepper_out_4_7_and.clone(),
            1,
        );
    }

    fn load_store_instr_7_and_connect(&mut self) {
        connect_gates(
            self.load_store_instr_7_and.clone(),
            0,
            self.stepper_out_4_8_and.clone(),
            1,
        );

        connect_gates(
            self.load_store_instr_7_and.clone(),
            1,
            self.stepper_out_5_6_and.clone(),
            1,
        );
    }

    fn stepper_out_4_top_0_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_top_0_and.clone(),
            0,
            self.reg_b_e_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_out_4_top_0_and.clone(),
            1,
            self.tmp_s_and.clone(),
            1,
        );
    }

    fn stepper_out_4_1_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_1_and.clone(),
            0,
            self.reg_a_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_4_1_and.clone(),
            1,
            self.mar_s_outer_or.clone(),
            3,
        );
    }

    fn stepper_out_4_2_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_2_and.clone(),
            0,
            self.reg_a_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_out_4_2_and.clone(),
            1,
            self.mar_s_outer_or.clone(),
            4,
        );
    }

    fn stepper_out_4_3_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_3_and.clone(),
            0,
            self.mar_s_outer_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_4_3_and.clone(),
            1,
            self.bus_1_or.clone(),
            3,
        );

        connect_gates(
            self.stepper_out_4_3_and.clone(),
            2,
            self.acc_s_outer_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_4_3_and.clone(),
            3,
            self.iar_e_or.clone(),
            1,
        );
    }

    fn stepper_out_4_4_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_4_and.clone(),
            0,
            self.reg_b_e_or.clone(),
            2,
        );

        connect_gates(
            self.stepper_out_4_4_and.clone(),
            1,
            self.iar_s_outer_or.clone(),
            1,
        );
    }

    fn stepper_out_4_5_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_5_and.clone(),
            0,
            self.iar_e_or.clone(),
            2,
        );

        connect_gates(
            self.stepper_out_4_5_and.clone(),
            1,
            self.mar_s_outer_or.clone(),
            5,
        );
    }

    fn stepper_out_4_6_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_6_and.clone(),
            0,
            self.acc_s_outer_or.clone(),
            2,
        );

        connect_gates(
            self.stepper_out_4_6_and.clone(),
            1,
            self.iar_e_or.clone(),
            3,
        );

        connect_gates(
            self.stepper_out_4_6_and.clone(),
            2,
            self.mar_s_outer_or.clone(),
            2,
        );

        connect_gates(
            self.stepper_out_4_6_and.clone(),
            3,
            self.bus_1_or.clone(),
            2,
        );
    }

    fn stepper_out_4_7_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_7_and.clone(),
            0,
            self.flags_s_outer_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_out_4_7_and.clone(),
            1,
            self.bus_1_or.clone(),
            1,
        );
    }

    fn stepper_out_4_8_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_4_8_and.clone(),
            0,
            self.reg_b_e_or.clone(),
            3,
        );

        connect_gates(
            self.stepper_out_4_8_and.clone(),
            1,
            self.io_clks_s_and.clone(),
            1,
        );
    }

    fn stepper_out_5_top_0_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        connect_gates(
            self.stepper_out_5_top_0_and.clone(),
            0,
            self.reg_a_or.clone(),
            2,
        );

        connect_gates(
            self.stepper_out_5_top_0_and.clone(),
            1,
            self.flags_s_outer_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_5_top_0_and.clone(),
            2,
            self.acc_s_outer_or.clone(),
            3,
        );

        let c_out_index = self.get_index_from_tag(ControlSection::C_OUT);
        connect_gates(
            self.stepper_out_5_top_0_and.clone(),
            3,
            output_gates[c_out_index].clone(),
            0,
        );
    }

    fn stepper_out_5_1_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_5_1_and.clone(),
            0,
            self.reg_b_s_or.clone(),
            0,
        );

        connect_gates(
            self.stepper_out_5_1_and.clone(),
            1,
            self.ram_e_or.clone(),
            4,
        );
    }

    fn stepper_out_5_2_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_5_2_and.clone(),
            0,
            self.reg_b_e_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_5_2_and.clone(),
            1,
            self.ram_s_and.clone(),
            1,
        );
    }

    fn stepper_out_5_3_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_5_3_and.clone(),
            0,
            self.reg_b_s_or.clone(),
            2,
        );

        connect_gates(
            self.stepper_out_5_3_and.clone(),
            1,
            self.ram_e_or.clone(),
            3,
        );
    }

    fn stepper_out_5_4_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_5_4_and.clone(),
            0,
            self.iar_s_outer_or.clone(),
            3,
        );

        connect_gates(
            self.stepper_out_5_4_and.clone(),
            1,
            self.ram_e_or.clone(),
            2,
        );
    }

    fn stepper_out_5_5_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_5_5_and.clone(),
            0,
            self.acc_e_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_5_5_and.clone(),
            1,
            self.iar_s_outer_or.clone(),
            2,
        );
    }

    fn stepper_out_5_6_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_5_6_and.clone(),
            0,
            self.reg_b_s_or.clone(),
            3,
        );

        connect_gates(
            self.stepper_out_5_6_and.clone(),
            1,
            self.io_clk_e_and.clone(),
            1,
        );
    }

    fn stepper_out_5_6_not_connect(&mut self) {
        connect_gates(
            self.stepper_out_5_6_not.clone(),
            0,
            self.stepper_out_5_6_and.clone(),
            2,
        );
    }

    fn stepper_out_6_top_0_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_6_top_0_and.clone(),
            0,
            self.reg_b_s_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_6_top_0_and.clone(),
            1,
            self.acc_e_or.clone(),
            3,
        );
    }

    fn stepper_out_6_1_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_6_1_and.clone(),
            0,
            self.acc_e_or.clone(),
            2,
        );

        connect_gates(
            self.stepper_out_6_1_and.clone(),
            1,
            self.iar_s_outer_or.clone(),
            4,
        );
    }

    fn stepper_out_6_2_and_connect(&mut self) {
        connect_gates(
            self.stepper_out_6_2_and.clone(),
            0,
            self.ram_e_or.clone(),
            1,
        );

        connect_gates(
            self.stepper_out_6_2_and.clone(),
            1,
            self.iar_s_outer_or.clone(),
            5,
        );
    }

    fn eight_input_and_connect(
        &mut self,
        output_gates: &Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let end_index = self.get_index_from_tag(ControlSection::END);
        connect_gates(
            self.eight_input_and.clone(),
            0,
            output_gates[end_index].clone(),
            0,
        );
    }

    fn eight_input_and_not_loc_2_connect(&mut self) {
        connect_gates(
            self.eight_input_and_not_loc_2.clone(),
            0,
            self.eight_input_and.clone(),
            2,
        );
    }

    fn eight_input_and_not_loc_3_connect(&mut self) {
        connect_gates(
            self.eight_input_and_not_loc_3.clone(),
            0,
            self.eight_input_and.clone(),
            3,
        );
    }

    fn c_in_and_connect(&mut self) {
        connect_gates(
            self.c_in_and.clone(),
            0,
            self.alu_input_or.clone(),
            0,
        );
    }

    fn a_l_and_connect(&mut self) {
        connect_gates(
            self.a_l_and.clone(),
            0,
            self.alu_input_or.clone(),
            1,
        );
    }

    fn eq_and_connect(&mut self) {
        connect_gates(
            self.eq_and.clone(),
            0,
            self.alu_input_or.clone(),
            2,
        );
    }

    fn z_and_connect(&mut self) {
        connect_gates(
            self.z_and.clone(),
            0,
            self.alu_input_or.clone(),
            3,
        );
    }

    fn alu_input_or_connect(&mut self) {
        connect_gates(
            self.alu_input_or.clone(),
            0,
            self.stepper_out_6_2_and.clone(),
            2,
        );
    }

    fn add_and_connect(&mut self) {
        connect_gates(
            self.add_and.clone(),
            0,
            self.add_not.clone(),
            0,
        );
    }

    fn add_not_connect(&mut self) {
        connect_gates(
            self.add_not.clone(),
            0,
            self.stepper_out_6_top_0_and.clone(),
            2,
        );
    }
}

impl LogicGate for ControlSection {
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
        let start = Instant::now();

        let result = self.complex_gate.fetch_output_signals(
            &self.get_tag(),
        );

        unsafe {
            CONTROL_SECTION_TIME += start.elapsed();
        }

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
    use crate::logic::input_gates::AutomaticInput;
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::{extract_output_tags_sorted_by_index, run_multi_input_output_logic_gate_return};
    use super::*;

    //This uses a 4 cycle clock, so the number of clock cycles it will advance will be
    // 4 * advance_to_splitter_num.
    fn test_control_section(
        output_signals_map: HashMap<&str, Vec<Signal>>,
        input_signals_map: HashMap<&str, Vec<Vec<Signal>>>,
        start_at_splitter_num: usize,
    ) {
        assert_ne!(output_signals_map.len(), 0);
        //Splitters only go [1-6].
        assert!(start_at_splitter_num > 0);
        assert!(start_at_splitter_num < 7);

        //Minus 1 to get a proper number to fit multiplication (to get to splitter 1 it is 0 * 4).
        // The outer minus 1 is to get to the spot right before the splitter is initialized.
        let clock_ticks_to_advance: i32 = ((start_at_splitter_num as i32 - 1) * 4) - 1;

        let length = output_signals_map.iter().next().unwrap().1.len();
        for (_tag, v) in output_signals_map.iter() {
            assert_eq!(v.len(), length);
        }

        let control_section = ControlSection::new(8);
        let mut output_signals = Vec::new();

        let mut current_idx = 0;
        while current_idx < length {
            let mut output = vec![LOW_; control_section.lock().unwrap().complex_gate.output_gates.len()];
            for (tag, v) in output_signals_map.iter() {
                let idx = control_section.lock().unwrap().get_index_from_tag(tag);
                output[idx] = v[current_idx].clone();
            }
            current_idx += 1;
            output_signals.push(output);
        }

        if clock_ticks_to_advance > 1 {
            //The splitter changes on the negative edge, so transitioning from HIGH->LOW.
            let mut clock_input = Vec::new();
            for _ in 0..start_at_splitter_num {
                clock_input.push(HIGH);
                clock_input.push(LOW_);
            }

            let clock_input = AutomaticInput::new(clock_input, 1, "Clock");

            let clk_index = control_section.lock().unwrap().get_index_from_tag(ControlSection::CLOCK);
            connect_gates(
                clock_input.clone(),
                0,
                control_section.clone(),
                clk_index,
            );

            let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
            input_gates.push(clock_input);
            let output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

            let mut cycle_nums = 0;
            let mut continue_cycles = true;
            while continue_cycles {
                cycle_nums += 1;
                continue_cycles = run_circuit(
                    &input_gates,
                    &output_gates,
                    false,
                    &mut |_clock_tick_inputs, _output_gates| {},
                );
            }

            println!("cycle_nums {cycle_nums}");

            control_section.lock().unwrap().update_input_signal(
                GateInput::new(
                    clk_index,
                    Signal::NONE,
                    input_gates[0].lock().unwrap().get_unique_id(),
                )
            );
        }

        println!("Running inputs.");
        let collected_output = run_multi_input_output_logic_gate_return(
            vec![],
            &output_signals,
            input_signals_map,
            control_section.clone(),
        );

        assert_eq!(output_signals.len(), collected_output.len());

        println!("Advanced for {} clock ticks", clock_ticks_to_advance);
        println!("Ran for {} clock ticks", output_signals.len());
        let tags_sorted_by_index = extract_output_tags_sorted_by_index(&control_section.lock().unwrap().complex_gate);

        let mut failed = false;
        for i in 0..output_signals.len() {
            let mut failed_map = HashMap::new();

            for j in 0..output_signals[i].len() {
                if (tags_sorted_by_index[j].clone(), output_signals[i][j].clone()) != (tags_sorted_by_index[j].clone(), collected_output[i][j].clone()) {
                    failed_map.insert(tags_sorted_by_index[j].clone(), (output_signals[i][j].clone(), collected_output[i][j].clone()));
                    failed = true;
                };
            }

            if !failed_map.is_empty() {
                println!("Clock tick {}\nfailed (passed, collected): {:?}", i, failed_map);
            }
        }

        assert!(!failed);
    }

    struct ClockTickRounds {
        clock: (&'static str, Vec<Vec<Signal>>),
        clock_enable: (&'static str, Vec<Vec<Signal>>),
        clock_set: (&'static str, Vec<Vec<Signal>>),
    }

    fn get_clock_cycles(num_cycles: usize) -> ClockTickRounds {
        let mut clock_vec = Vec::new();
        let mut clock_enable_vec = Vec::new();
        let mut clock_set_vec = Vec::new();

        for _ in 0..num_cycles {
            clock_vec.push(vec![LOW_]);
            clock_vec.push(vec![HIGH]);
            clock_vec.push(vec![HIGH]);
            clock_vec.push(vec![LOW_]);

            clock_enable_vec.push(vec![HIGH]);
            clock_enable_vec.push(vec![HIGH]);
            clock_enable_vec.push(vec![HIGH]);
            clock_enable_vec.push(vec![LOW_]);

            clock_set_vec.push(vec![LOW_]);
            clock_set_vec.push(vec![HIGH]);
            clock_set_vec.push(vec![LOW_]);
            clock_set_vec.push(vec![LOW_]);
        }

        ClockTickRounds {
            clock: (ControlSection::CLOCK, clock_vec),
            clock_enable: (ControlSection::CLOCK_ENABLE, clock_enable_vec),
            clock_set: (ControlSection::CLOCK_SET, clock_set_vec),
        }
    }

    #[test]
    fn control_section_initialization() {
        let control_section = ControlSection::new(8);

        let tags_sorted_by_index = extract_output_tags_sorted_by_index(&control_section.lock().unwrap().complex_gate);

        let collected_output = control_section.lock().unwrap().fetch_output_signals().unwrap();

        let mut generated_output = vec![LOW_; tags_sorted_by_index.len()];
        generated_output[0] = HIGH; //Set BUS_1 high

        let collected_output: Vec<Signal> = collected_output.into_iter().map(
            |out| {
                match out {
                    GateOutputState::NotConnected(signal) => signal,
                    GateOutputState::Connected(_) => panic!("No gate output should be connected.")
                }
            }
        ).collect();

        assert_eq!(generated_output, collected_output);
    }

    fn copy_input_n_times(v: Vec<Signal>, num_copies: usize) -> Vec<Vec<Signal>> {
        let mut return_vec = Vec::new();
        for _ in 0..num_copies {
            return_vec.push(v.clone());
        }
        return_vec
    }

    #[test]
    fn control_section_fetch_instructions() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::IAR_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::MAR_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::RAM_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::IR_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::IAR_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_set,
                    clock_tick_rounds.clock_enable
                ]
            ),
            1,
        );
    }

    #[test]
    fn control_section_add() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R2_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::IO, vec![HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH]),
                    (ControlSection::R1_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R2_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![LOW_, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, HIGH], // R1+R2=R2
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_shift_right() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R0_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R3_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                    (ControlSection::ALU_0, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, HIGH, LOW_, LOW_, HIGH], // R3
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_shift_left() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R0_E, vec![HIGH, HIGH, HIGH, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R0_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                    (ControlSection::ALU_1, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, HIGH], // R3
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_not() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R1_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R0_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R1_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                    (ControlSection::ALU_0, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ALU_1, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, LOW_, LOW_, LOW_, HIGH, HIGH, LOW_, HIGH], // R1
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_and() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R0_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R3_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                    (ControlSection::ALU_2, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH], // R3 & R0 = R3
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_or() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R0_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R3_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                    (ControlSection::ALU_0, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ALU_2, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, HIGH, LOW_, HIGH, HIGH], // R3 | R0 = R3
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_xor() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R0_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R3_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                    (ControlSection::ALU_1, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ALU_2, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH], // R3 ^ R0 = R3
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_cmp() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::TMP_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R0_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::C_OUT, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ALU_0, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ALU_1, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ALU_2, vec![LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, HIGH, HIGH, HIGH, HIGH], // R3 > R0 = R3
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_store() {
        let clock_tick_rounds = get_clock_cycles(2);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::R0_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::MAR_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::RAM_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_], // Store R3 in RAM address inside R0.
                        8,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_load() {
        let clock_tick_rounds = get_clock_cycles(2);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::R0_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::MAR_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::RAM_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::R3_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_], // Load R3 from RAM address inside R0.
                        8,
                    ))
                ]
            ),
            4,
        );
    }


    #[test]
    fn control_section_data() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::IAR_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::MAR_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::RAM_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::IAR_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_], // Store data from next instruction address in R3.
                        12,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_jump() {
        let clock_tick_rounds = get_clock_cycles(1);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::R3_E, vec![HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::IAR_S, vec![LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, HIGH, LOW_, LOW_, HIGH, HIGH, LOW_, LOW_], // Jump to address in R3.
                        4,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_jump_addr() {
        let clock_tick_rounds = get_clock_cycles(2);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::IAR_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::MAR_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::RAM_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::IAR_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_], // Jumps to the address stored in the next byte (inside IAR) in RAM.
                        8,
                    ))
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_jump_if_true() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::IAR_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::MAR_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::RAM_E, vec![LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::IAR_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![HIGH, LOW_, LOW_, LOW_, HIGH, LOW_, HIGH, LOW_], // Jumps to the address stored in next RAM location if flags set (flag set).
                        12,
                    )),
                    ("Z", vec![vec![LOW_], vec![LOW_], vec![LOW_], vec![LOW_], vec![LOW_], vec![LOW_], vec![LOW_], vec![LOW_], vec![HIGH], vec![HIGH], vec![HIGH], vec![LOW_]])
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_jump_if_false() {
        let clock_tick_rounds = get_clock_cycles(3);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, HIGH]),
                    (ControlSection::IAR_E, vec![HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::MAR_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_S, vec![LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::ACC_E, vec![LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_]),
                    (ControlSection::IAR_S, vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, LOW_, LOW_, LOW_, LOW_, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![LOW_, HIGH, LOW_, LOW_, HIGH, LOW_, HIGH, LOW_], // Jumps to the address stored in next RAM location if flags set (flag not set).
                        12,
                    )),
                ]
            ),
            4,
        );
    }

    #[test]
    fn control_section_clear_flags() {
        let clock_tick_rounds = get_clock_cycles(1);
        test_control_section(
            HashMap::from(
                [
                    (ControlSection::BUS_1,  vec![HIGH, HIGH, HIGH, LOW_]),
                    (ControlSection::FLAG_S, vec![LOW_, HIGH, LOW_, LOW_]),
                ]
            ),
            HashMap::from(
                [
                    clock_tick_rounds.clock,
                    clock_tick_rounds.clock_enable,
                    clock_tick_rounds.clock_set,
                    ("IR", copy_input_n_times(
                        vec![LOW_, LOW_, LOW_, LOW_, LOW_, HIGH, HIGH, LOW_], // Jumps to the address stored in next RAM location if flags set (flag not set).
                        4,
                    )),
                ]
            ),
            4,
        );
    }
}

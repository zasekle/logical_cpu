use std::cell::RefCell;
use std::rc::Rc;
use crate::logic::basic_gates::{And, Not, Or, Splitter};
use crate::logic::complex_logic::VariableOutputStepper;
use crate::logic::foundations::{build_simple_inputs_and_outputs, build_simple_inputs_and_outputs_with_and, calculate_input_signals_from_all_inputs, ComplexGateMembers, GateInput, GateLogicError, GateOutputState, GateType, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::foundations::GateType::OneBitMemoryCellType;
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};

#[allow(unused_imports)]
use crate::logic::foundations::Signal::{LOW, HIGH};
use crate::logic::memory_gates::OneBitMemoryCell;

pub struct ControlSection {
    complex_gate: ComplexGateMembers,
    stepper: Rc<RefCell<VariableOutputStepper>>,
    stepper_splitters: Vec<Rc<RefCell<Splitter>>>,
    ram_e_and: Rc<RefCell<And>>,
    acc_e_and: Rc<RefCell<And>>,
    iar_e_and: Rc<RefCell<And>>,
    r0_e_and: Rc<RefCell<And>>,
    r1_e_and: Rc<RefCell<And>>,
    r2_e_and: Rc<RefCell<And>>,
    r3_e_and: Rc<RefCell<And>>,
    mar_s_and: Rc<RefCell<And>>,
    ram_s_and: Rc<RefCell<And>>,
    acc_s_and: Rc<RefCell<And>>,
    iar_s_and: Rc<RefCell<And>>,
    r0_s_and: Rc<RefCell<And>>,
    r1_s_and: Rc<RefCell<And>>,
    r2_s_and: Rc<RefCell<And>>,
    r3_s_and: Rc<RefCell<And>>,
    ir_s_and: Rc<RefCell<And>>,
    tmp_s_and: Rc<RefCell<And>>,
}

#[allow(dead_code)]
impl ControlSection {
    //Inputs
    const CLOCK: &'static str = "CLK";
    const CLOCK_ENABLE: &'static str = "CLKE";
    const CLOCK_SET: &'static str = "CLKS";

    //Outputs
    const BUS_1: &'static str = "BUS_1";
    const RAM_E: &'static str = "RAM_E";
    const ACC_E: &'static str = "ACC_E";
    const IAR_E: &'static str = "IAR_E";
    const R0_E: &'static str = "R0_E";
    const R1_E: &'static str = "R1_E";
    const R2_E: &'static str = "R2_E";
    const R3_E: &'static str = "R3_E";
    const MAR_S: &'static str = "MAR_S";
    const RAM_S: &'static str = "RAM_S";
    const ACC_S: &'static str = "ACC_S";
    const IAR_S: &'static str = "IAR_S";
    const R0_S: &'static str = "R0_S";
    const R1_S: &'static str = "R1_S";
    const R2_S: &'static str = "R2_S";
    const R3_S: &'static str = "R3_S";
    const IR_S: &'static str = "IR_S";
    const TMP_S: &'static str = "TMP_S";

    pub fn new(bus_width: usize) -> Rc<RefCell<Self>> {
        assert_ne!(bus_width, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        //TODO: how many bits? it deals with instructions mostly, does it need to be big enough to
        // get it in from the bus?

        for i in 0..bus_width {
            let input_tag = format!("IR_{}",i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));
        }

        let mut stepper_splitters = Vec::new();

        stepper_splitters.push(Splitter::new(1, 4));
        stepper_splitters.push(Splitter::new(1, 2));
        stepper_splitters.push(Splitter::new(1, 2));
        //Placeholders
        stepper_splitters.push(Splitter::new(1, 1));
        stepper_splitters.push(Splitter::new(1, 1));
        stepper_splitters.push(Splitter::new(1, 1));

        input_gates.push(SimpleInput::new(10, ControlSection::CLOCK_SET));
        input_gates.push(SimpleInput::new(1, ControlSection::CLOCK));
        input_gates.push(SimpleInput::new(7, ControlSection::CLOCK_ENABLE));

        let bus_1_output_gate = SimpleOutput::new(ControlSection::BUS_1);
        let ram_e_output_gate = SimpleOutput::new(ControlSection::RAM_E);
        let acc_e_output_gate = SimpleOutput::new(ControlSection::ACC_E);
        let iar_e_output_gate = SimpleOutput::new(ControlSection::IAR_E);
        let r0_e_output_gate = SimpleOutput::new(ControlSection::R0_E);
        let r1_e_output_gate = SimpleOutput::new(ControlSection::R1_E);
        let r2_e_output_gate = SimpleOutput::new(ControlSection::R2_E);
        let r3_e_output_gate = SimpleOutput::new(ControlSection::R3_E);
        let mar_s_output_gate = SimpleOutput::new(ControlSection::MAR_S);
        let ram_s_output_gate = SimpleOutput::new(ControlSection::RAM_S);
        let acc_s_output_gate = SimpleOutput::new(ControlSection::ACC_S);
        let iar_s_output_gate = SimpleOutput::new(ControlSection::IAR_S);
        let r0_s_output_gate = SimpleOutput::new(ControlSection::R0_S);
        let r1_s_output_gate = SimpleOutput::new(ControlSection::R1_S);
        let r2_s_output_gate = SimpleOutput::new(ControlSection::R2_S);
        let r3_s_output_gate = SimpleOutput::new(ControlSection::R3_S);
        let ir_s_output_gate = SimpleOutput::new(ControlSection::IR_S);
        let tmp_s_output_gate = SimpleOutput::new(ControlSection::TMP_S);

        output_gates.push(bus_1_output_gate.clone());
        output_gates.push(ram_e_output_gate.clone());
        output_gates.push(acc_e_output_gate.clone());
        output_gates.push(iar_e_output_gate.clone());
        output_gates.push(r0_e_output_gate.clone());
        output_gates.push(r1_e_output_gate.clone());
        output_gates.push(r2_e_output_gate.clone());
        output_gates.push(r3_e_output_gate.clone());
        output_gates.push(mar_s_output_gate.clone());
        output_gates.push(ram_s_output_gate.clone());
        output_gates.push(acc_s_output_gate.clone());
        output_gates.push(iar_s_output_gate.clone());
        output_gates.push(r0_s_output_gate.clone());
        output_gates.push(r1_s_output_gate.clone());
        output_gates.push(r2_s_output_gate.clone());
        output_gates.push(r3_s_output_gate.clone());
        output_gates.push(ir_s_output_gate.clone());
        output_gates.push(tmp_s_output_gate.clone());
        output_gates_logic.push(bus_1_output_gate);
        output_gates_logic.push(ram_e_output_gate);
        output_gates_logic.push(acc_e_output_gate);
        output_gates_logic.push(iar_e_output_gate);
        output_gates_logic.push(r0_e_output_gate);
        output_gates_logic.push(r1_e_output_gate);
        output_gates_logic.push(r2_e_output_gate);
        output_gates_logic.push(r3_e_output_gate);
        output_gates_logic.push(mar_s_output_gate);
        output_gates_logic.push(ram_s_output_gate);
        output_gates_logic.push(acc_s_output_gate);
        output_gates_logic.push(iar_s_output_gate);
        output_gates_logic.push(r0_s_output_gate);
        output_gates_logic.push(r1_s_output_gate);
        output_gates_logic.push(r2_s_output_gate);
        output_gates_logic.push(r3_s_output_gate);
        output_gates_logic.push(ir_s_output_gate);
        output_gates_logic.push(tmp_s_output_gate);

        let mut one_bit_memory_cell = ControlSection {
            complex_gate: ComplexGateMembers::new(
                bus_width + 3,
                18,
                GateType::ControlSectionType,
                input_gates,
                output_gates,
            ),
            stepper: VariableOutputStepper::new(6),
            stepper_splitters,
            ram_e_and: And::new(2, 1),
            acc_e_and: And::new(2, 1),
            iar_e_and: And::new(2, 1),
            r0_e_and: And::new(2, 1),
            r1_e_and: And::new(2, 1),
            r2_e_and: And::new(2, 1),
            r3_e_and: And::new(2, 1),
            mar_s_and: And::new(2, 1),
            ram_s_and: And::new(2, 1),
            acc_s_and: And::new(2, 1),
            iar_s_and: And::new(2, 1),
            r0_s_and: And::new(2, 1),
            r1_s_and: And::new(2, 1),
            r2_s_and: And::new(2, 1),
            r3_s_and: And::new(2, 1),
            ir_s_and: And::new(2, 1),
            tmp_s_and: And::new(2, 1),
        };

        one_bit_memory_cell.build_and_prime_circuit(
            bus_width,
            output_gates_logic
        );

        Rc::new(RefCell::new(one_bit_memory_cell))
    }

    fn build_and_prime_circuit(
        &mut self,
        bus_width: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {

        self.connect_enable_and_gates(
            &output_gates,
        );

        self.connect_set_and_gates(
            &output_gates
        );

        let clk_input_gate = self.complex_gate.input_gates[self.get_index_from_tag(ControlSection::CLOCK)].clone();
        let stepper_clk_index = self.stepper.borrow_mut().get_index_from_tag("CLK");
        clk_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            stepper_clk_index,
            self.stepper.clone(),
        );

        self.connect_clks_to_set_gates();

        self.connect_clke_to_enable_gates();

        self.connect_fetch_cycle();

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }

    fn connect_enable_and_gates(
        &mut self,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let ram_e_index = self.get_index_from_tag(ControlSection::RAM_E);
        self.ram_e_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[ram_e_index].clone(),
        );

        let acc_e_index = self.get_index_from_tag(ControlSection::ACC_E);
        self.acc_e_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[acc_e_index].clone(),
        );

        let iar_e_index = self.get_index_from_tag(ControlSection::IAR_E);
        self.iar_e_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[iar_e_index].clone(),
        );

        let r0_e_index = self.get_index_from_tag(ControlSection::R0_E);
        self.r0_e_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r0_e_index].clone(),
        );

        let r1_e_index = self.get_index_from_tag(ControlSection::R1_E);
        self.r1_e_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r1_e_index].clone(),
        );

        let r2_e_index = self.get_index_from_tag(ControlSection::R2_E);
        self.r2_e_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r2_e_index].clone(),
        );

        let r3_e_index = self.get_index_from_tag(ControlSection::R3_E);
        self.r3_e_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r3_e_index].clone(),
        );
    }

    fn connect_set_and_gates(
        &mut self,
        output_gates: &Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let mar_s_index = self.get_index_from_tag(ControlSection::MAR_S);
        self.mar_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[mar_s_index].clone(),
        );

        let ram_s_index = self.get_index_from_tag(ControlSection::RAM_S);
        self.ram_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[ram_s_index].clone(),
        );

        let acc_s_index = self.get_index_from_tag(ControlSection::ACC_S);
        self.acc_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[acc_s_index].clone(),
        );

        let iar_s_index = self.get_index_from_tag(ControlSection::IAR_S);
        self.iar_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[iar_s_index].clone(),
        );

        let r0_s_index = self.get_index_from_tag(ControlSection::R0_S);
        self.r0_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r0_s_index].clone(),
        );

        let r1_s_index = self.get_index_from_tag(ControlSection::R1_S);
        self.r1_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r1_s_index].clone(),
        );

        let r2_s_index = self.get_index_from_tag(ControlSection::R2_S);
        self.r2_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r2_s_index].clone(),
        );

        let r3_s_index = self.get_index_from_tag(ControlSection::R3_S);
        self.r3_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[r3_s_index].clone(),
        );

        let ir_s_index = self.get_index_from_tag(ControlSection::IR_S);
        self.ir_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[ir_s_index].clone(),
        );

        let tmp_s_index = self.get_index_from_tag(ControlSection::TMP_S);
        self.tmp_s_and.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[tmp_s_index].clone(),
        );
    }

    fn connect_clks_to_set_gates(&mut self) {
        let clks_input_gate = self.complex_gate.input_gates[self.get_index_from_tag(ControlSection::CLOCK_SET)].clone();

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.mar_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.ram_s_and.clone(),
        );
        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.acc_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.iar_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r0_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r1_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r2_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r3_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.ir_s_and.clone(),
        );

        clks_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.tmp_s_and.clone(),
        );
    }

    fn connect_clke_to_enable_gates(&mut self) {
        let clke_input_gate = self.complex_gate.input_gates[self.get_index_from_tag(ControlSection::CLOCK_ENABLE)].clone();

        clke_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.ram_e_and.clone(),
        );

        clke_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.acc_e_and.clone(),
        );

        clke_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.iar_e_and.clone(),
        );

        clke_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r0_e_and.clone(),
        );

        clke_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r1_e_and.clone(),
        );

        clke_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r2_e_and.clone(),
        );

        clke_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.r3_e_and.clone(),
        );
    }

    fn connect_fetch_cycle(&mut self) {


    }
}

impl LogicGate for ControlSection {
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

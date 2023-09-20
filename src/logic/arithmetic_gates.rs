use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use crate::logic::basic_gates::{And, Not, Or, XOr};

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, InputSignalReturn, Signal, ComplexGateMembers, build_simple_inputs_and_outputs};
use crate::logic::foundations::Signal::{HIGH, LOW};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
use crate::logic::processor_components::VariableBitRegister;

pub struct HalfAdder {
    complex_gate: ComplexGateMembers,
    sum_xor_gate: Rc<RefCell<XOr>>,
    carry_and_gate: Rc<RefCell<And>>,
}

#[allow(dead_code)]
impl HalfAdder {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        input_gates.push(SimpleInput::new(2, "A"));
        input_gates.push(SimpleInput::new(2, "B"));

        let sum_output_gate = SimpleOutput::new("S");
        let carry_output_gate = SimpleOutput::new("C");
        output_gates.push(sum_output_gate.clone());
        output_gates.push(carry_output_gate.clone());
        output_gates_logic.push(sum_output_gate);
        output_gates_logic.push(carry_output_gate);

        let mut half_adder = HalfAdder {
            complex_gate: ComplexGateMembers::new(
                2,
                2,
                GateType::HalfAdderType,
                input_gates,
                output_gates,
            ),
            sum_xor_gate: XOr::new(2, 1),
            carry_and_gate: And::new(2, 1),
        };

        half_adder.build_and_prime_circuit(output_gates_logic);

        Rc::new(RefCell::new(half_adder))
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let a_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("A")].clone();
        let b_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("B")].clone();

        a_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.sum_xor_gate.clone(),
        );

        a_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            1,
            self.carry_and_gate.clone(),
        );

        b_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.sum_xor_gate.clone(),
        );

        b_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            0,
            self.carry_and_gate.clone(),
        );

        let sum_output_gate_index = self.get_index_from_tag("S");
        self.sum_xor_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[sum_output_gate_index].clone(),
        );

        let carry_output_gate_index = self.get_index_from_tag("C");
        self.carry_and_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[carry_output_gate_index].clone(),
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for HalfAdder {
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

pub struct FullAdder {
    complex_gate: ComplexGateMembers,
    input_half_added: Rc<RefCell<HalfAdder>>,
    carry_half_adder: Rc<RefCell<HalfAdder>>,
    or_gate: Rc<RefCell<Or>>,
}

#[allow(dead_code)]
impl FullAdder {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        input_gates.push(SimpleInput::new(1, "A"));
        input_gates.push(SimpleInput::new(1, "B"));
        input_gates.push(SimpleInput::new(1, "C_IN"));

        let sum_output_gate = SimpleOutput::new("S");
        let carry_output_gate = SimpleOutput::new("C_OUT");
        output_gates.push(sum_output_gate.clone());
        output_gates.push(carry_output_gate.clone());
        output_gates_logic.push(sum_output_gate);
        output_gates_logic.push(carry_output_gate);

        let mut full_adder = FullAdder {
            complex_gate: ComplexGateMembers::new(
                3,
                2,
                GateType::FullAdderType,
                input_gates,
                output_gates,
            ),
            input_half_added: HalfAdder::new(),
            carry_half_adder: HalfAdder::new(),
            or_gate: Or::new(2, 1),
        };

        full_adder.build_and_prime_circuit(output_gates_logic);

        Rc::new(RefCell::new(full_adder))
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let a_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("A")].clone();
        let b_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("B")].clone();
        let c_in_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("C_IN")].clone();

        let a_input_index = self.input_half_added.borrow_mut().get_index_from_tag("A");
        a_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            a_input_index,
            self.input_half_added.clone(),
        );

        let b_input_index = self.input_half_added.borrow_mut().get_index_from_tag("B");
        b_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            b_input_index,
            self.input_half_added.clone(),
        );

        let a_input_index = self.carry_half_adder.borrow_mut().get_index_from_tag("A");
        let s_output_index = self.input_half_added.borrow_mut().get_index_from_tag("S");
        self.input_half_added.borrow_mut().connect_output_to_next_gate(
            s_output_index,
            a_input_index,
            self.carry_half_adder.clone(),
        );

        let b_input_index = self.carry_half_adder.borrow_mut().get_index_from_tag("B");
        c_in_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            b_input_index,
            self.carry_half_adder.clone(),
        );

        let s_out_output_index = self.carry_half_adder.borrow_mut().get_index_from_tag("S");
        let s_out_index = self.get_index_from_tag("S");
        self.carry_half_adder.borrow_mut().connect_output_to_next_gate(
            s_out_output_index,
            0,
            output_gates[s_out_index].clone(),
        );

        let c_out_output_index = self.carry_half_adder.borrow_mut().get_index_from_tag("C");
        self.carry_half_adder.borrow_mut().connect_output_to_next_gate(
            c_out_output_index,
            0,
            self.or_gate.clone(),
        );

        let c_out_output_index = self.input_half_added.borrow_mut().get_index_from_tag("C");
        self.input_half_added.borrow_mut().connect_output_to_next_gate(
            c_out_output_index,
            1,
            self.or_gate.clone(),
        );

        let c_out_index = self.get_index_from_tag("C_OUT");
        self.or_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gates[c_out_index].clone(),
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for FullAdder {
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

pub struct VariableBitAdder {
    complex_gate: ComplexGateMembers,
    full_adders: Vec<Rc<RefCell<FullAdder>>>,
}

#[allow(dead_code)]
impl VariableBitAdder {
    pub fn new(num_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let mut full_adders = Vec::new();

        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            input_gates.push(SimpleInput::new(1, a_input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            full_adders.push(
                FullAdder::new()
            );
        }

        //All a gates should go in before all b gates for consistency.
        for i in 0..num_bits {
            let b_input_tag = format!("b_{}", i);
            input_gates.push(SimpleInput::new(1, b_input_tag.as_str()));
        }

        input_gates.push(SimpleInput::new(1, "C_IN"));

        let carry_output_gate = SimpleOutput::new("C_OUT");
        output_gates.push(carry_output_gate.clone());
        output_gates_logic.push(carry_output_gate);

        let mut variable_bit_adder = VariableBitAdder {
            complex_gate: ComplexGateMembers::new(
                num_bits * 2 + 1,
                num_bits + 1,
                GateType::VariableBitAdderType,
                input_gates,
                output_gates,
            ),
            full_adders,
        };

        variable_bit_adder.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_bit_adder))
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);
            let output_tag = format!("o_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());
            let output_index = self.get_index_from_tag(output_tag.as_str());

            let mut mut_full_adder = self.full_adders[i].borrow_mut();

            let a_adder_index = mut_full_adder.get_index_from_tag("A");
            let b_adder_index = mut_full_adder.get_index_from_tag("B");
            let s_adder_index = mut_full_adder.get_index_from_tag("S");
            let c_in_adder_index = mut_full_adder.get_index_from_tag("C_IN");

            self.complex_gate.input_gates[a_input_index].borrow_mut().connect_output_to_next_gate(
                0,
                a_adder_index,
                self.full_adders[i].clone(),
            );

            self.complex_gate.input_gates[b_input_index].borrow_mut().connect_output_to_next_gate(
                0,
                b_adder_index,
                self.full_adders[i].clone(),
            );

            mut_full_adder.connect_output_to_next_gate(
                s_adder_index,
                0,
                output_gates[output_index].clone(),
            );

            if i == 0 { //First adder in the line.
                let c_out_adder_index = mut_full_adder.get_index_from_tag("C_OUT");
                let c_out_output_index = self.get_index_from_tag("C_OUT");
                mut_full_adder.connect_output_to_next_gate(
                    c_out_adder_index,
                    0,
                    output_gates[c_out_output_index].clone(),
                );
            } else {
                let c_out_adder_index = mut_full_adder.get_index_from_tag("C_OUT");
                mut_full_adder.connect_output_to_next_gate(
                    c_out_adder_index,
                    c_in_adder_index,
                    self.full_adders[i-1].clone(),
                );
            }
        }

        let c_in_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("C_IN")].clone();
        let c_in_adder_index = self.full_adders[num_bits-1].borrow_mut().get_index_from_tag("C_IN");

        c_in_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            c_in_adder_index,
            self.full_adders[num_bits-1].clone(),
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for VariableBitAdder {
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

pub struct VariableBitShiftLeft<const LEFT_SHIFT: bool> {
    complex_gate: ComplexGateMembers,
    first_register: Rc<RefCell<VariableBitRegister>>,
    second_register: Rc<RefCell<VariableBitRegister>>,
}

#[allow(dead_code)]
impl<const LEFT_SHIFT: bool> VariableBitShiftLeft<LEFT_SHIFT> {
    pub fn new(num_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        build_simple_inputs_and_outputs(
            num_bits,
            &mut input_gates,
            &mut output_gates,
            &mut output_gates_logic,
        );

        input_gates.push(SimpleInput::new(1, "S_IN"));

        let shift_output_gate = SimpleOutput::new("S_OUT");
        output_gates.push(shift_output_gate.clone());
        output_gates_logic.push(shift_output_gate);

        let mut variable_bit_shift_left = VariableBitShiftLeft {
            complex_gate: ComplexGateMembers::new(
                num_bits + 1,
                num_bits + 1,
                GateType::VariableBitShiftLeftType,
                input_gates,
                output_gates,
            ),
            first_register: VariableBitRegister::new(num_bits),
            second_register: VariableBitRegister::new(num_bits),
        };

        variable_bit_shift_left.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_bit_shift_left))
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        fn tie_register_bits_high(
            mut register: RefMut<VariableBitRegister>
        ) {
            let set_index = register.get_index_from_tag("S");
            let enable_index = register.get_index_from_tag("E");

            register.update_input_signal(
                GateInput::new(
                    set_index,
                    HIGH,
                    UniqueID::zero_id(),
                )
            );

            register.update_input_signal(
                GateInput::new(
                    enable_index,
                    HIGH,
                    UniqueID::zero_id(),
                )
            );
        }

        tie_register_bits_high(self.first_register.borrow_mut());
        tie_register_bits_high(self.second_register.borrow_mut());

        for i in 0..num_bits {
            self.complex_gate.input_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                i,
                self.first_register.clone(),
            );

            if (LEFT_SHIFT && i == (num_bits - 1))
                || (!LEFT_SHIFT && i == 0) {
                let shift_out_index = self.get_index_from_tag("S_OUT");
                self.first_register.borrow_mut().connect_output_to_next_gate(
                    i,
                    0,
                    output_gates[shift_out_index].clone(),
                );
            } else {
                let second_index =
                    if LEFT_SHIFT {
                        i + 1
                    } else {
                        i - 1
                    };

                self.first_register.borrow_mut().connect_output_to_next_gate(
                    i,
                    second_index,
                    self.second_register.clone(),
                );
            }

            self.second_register.borrow_mut().connect_output_to_next_gate(
                i,
                0,
                output_gates[i].clone(),
            );
        }

        let shift_in_index = self.get_index_from_tag("S_IN");
        let second_index =
            if LEFT_SHIFT {
                0
            } else {
                num_bits - 1
            };

        self.complex_gate.input_gates[shift_in_index].borrow_mut().connect_output_to_next_gate(
            0,
            second_index,
            self.second_register.clone(),
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl<const LEFT_SHIFT: bool> LogicGate for VariableBitShiftLeft<LEFT_SHIFT> {
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

pub struct VariableBitNot {
    complex_gate: ComplexGateMembers,
    not_gates: Vec<Rc<RefCell<Not>>>,
}

#[allow(dead_code)]
impl VariableBitNot {
    pub fn new(num_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let mut not_gates = Vec::new();

        for i in 0..num_bits {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            not_gates.push(
                Not::new(1)
            );
        }

        let mut variable_bit_not = VariableBitNot {
            complex_gate: ComplexGateMembers::new(
                num_bits,
                num_bits,
                GateType::VariableBitNotType,
                input_gates,
                output_gates,
            ),
            not_gates,
        };

        variable_bit_not.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_bit_not))
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..num_bits {
            self.complex_gate.input_gates[i].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                self.not_gates[i].clone(),
            );

            self.not_gates[i].borrow_mut().connect_output_to_next_gate(
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

impl LogicGate for VariableBitNot {
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

pub struct VariableBitAnd {
    complex_gate: ComplexGateMembers,
    and_gates: Vec<Rc<RefCell<And>>>,
}

#[allow(dead_code)]
impl VariableBitAnd {
    pub fn new(num_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let mut and_gates = Vec::new();

        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            input_gates.push(SimpleInput::new(1, a_input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            and_gates.push(
                And::new(2, 1)
            );
        }

        for i in 0..num_bits {
            let b_input_tag = format!("b_{}", i);
            input_gates.push(SimpleInput::new(1, b_input_tag.as_str()));
        }

        let mut variable_bit_and = VariableBitAnd {
            complex_gate: ComplexGateMembers::new(
                2 * num_bits,
                num_bits,
                GateType::VariableBitAndType,
                input_gates,
                output_gates,
            ),
            and_gates,
        };

        variable_bit_and.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_bit_and))
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());

            self.complex_gate.input_gates[a_input_index].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                self.and_gates[i].clone(),
            );

            self.complex_gate.input_gates[b_input_index].borrow_mut().connect_output_to_next_gate(
                0,
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

impl LogicGate for VariableBitAnd {
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

pub struct VariableBitOr {
    complex_gate: ComplexGateMembers,
    or_gates: Vec<Rc<RefCell<Or>>>,
}

#[allow(dead_code)]
impl VariableBitOr {
    pub fn new(num_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let mut or_gates = Vec::new();

        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            input_gates.push(SimpleInput::new(1, a_input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            or_gates.push(
                Or::new(2, 1)
            );
        }

        for i in 0..num_bits {
            let b_input_tag = format!("b_{}", i);
            input_gates.push(SimpleInput::new(1, b_input_tag.as_str()));
        }

        let mut variable_bit_and = VariableBitOr {
            complex_gate: ComplexGateMembers::new(
                2 * num_bits,
                num_bits,
                GateType::VariableBitOrType,
                input_gates,
                output_gates,
            ),
            or_gates,
        };

        variable_bit_and.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_bit_and))
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());

            self.complex_gate.input_gates[a_input_index].borrow_mut().connect_output_to_next_gate(
                0,
                0,
                self.or_gates[i].clone(),
            );

            self.complex_gate.input_gates[b_input_index].borrow_mut().connect_output_to_next_gate(
                0,
                1,
                self.or_gates[i].clone(),
            );

            self.or_gates[i].borrow_mut().connect_output_to_next_gate(
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

impl LogicGate for VariableBitOr {
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

pub struct XOrLE {
    complex_gate: ComplexGateMembers,
    xor_gate: Rc<RefCell<XOr>>,
    not_gate: Rc<RefCell<Not>>,
    equal_and_gate: Rc<RefCell<And>>,
    middle_and_gate: Rc<RefCell<And>>,
    or_gate: Rc<RefCell<Or>>,
}

#[allow(dead_code)]
impl XOrLE {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        input_gates.push(SimpleInput::new(2, "A"));
        input_gates.push(SimpleInput::new(1, "B"));
        input_gates.push(SimpleInput::new(2, "ABOVE_E"));
        input_gates.push(SimpleInput::new(1, "ABOVE_L"));

        let c_output_gate = SimpleOutput::new("C");
        let equal_output_gate = SimpleOutput::new("E");
        let a_larger_output_gate = SimpleOutput::new("A_L");

        output_gates.push(c_output_gate.clone());
        output_gates.push(equal_output_gate.clone());
        output_gates.push(a_larger_output_gate.clone());
        output_gates_logic.push(c_output_gate);
        output_gates_logic.push(equal_output_gate);
        output_gates_logic.push(a_larger_output_gate);

        let mut variable_bit_and = XOrLE {
            complex_gate: ComplexGateMembers::new(
                4,
                3,
                GateType::XOrLEType,
                input_gates,
                output_gates,
            ),
            xor_gate: XOr::new(2, 3),
            not_gate: Not::new(1),
            equal_and_gate: And::new(2, 1),
            middle_and_gate: And::new(3, 1),
            or_gate: Or::new(2, 1),
        };

        variable_bit_and.build_and_prime_circuit(
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_bit_and))
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        let a_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("A")].clone();
        let b_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("B")].clone();
        let equal_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("ABOVE_E")].clone();
        let larger_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("ABOVE_L")].clone();

        let c_output_gate = output_gates[self.get_index_from_tag("C")].clone();
        let equal_output_gate = output_gates[self.get_index_from_tag("E")].clone();
        let a_larger_output_gate = output_gates[self.get_index_from_tag("A_L")].clone();

        a_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.xor_gate.clone(),
        );

        a_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            1,
            self.middle_and_gate.clone(),
        );

        b_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.xor_gate.clone(),
        );

        self.xor_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.not_gate.clone(),
        );

        self.xor_gate.borrow_mut().connect_output_to_next_gate(
            1,
            2,
            self.middle_and_gate.clone(),
        );

        self.xor_gate.borrow_mut().connect_output_to_next_gate(
            2,
            2,
            c_output_gate.clone(),
        );

        self.not_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.equal_and_gate.clone(),
        );

        self.equal_and_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            equal_output_gate.clone(),
        );

        equal_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.middle_and_gate.clone(),
        );

        equal_input_gate.borrow_mut().connect_output_to_next_gate(
            1,
            1,
            self.equal_and_gate.clone(),
        );

        self.middle_and_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            self.or_gate.clone(),
        );

        larger_input_gate.borrow_mut().connect_output_to_next_gate(
            0,
            1,
            self.or_gate.clone(),
        );

        self.or_gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            a_larger_output_gate.clone(),
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for XOrLE {
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

pub struct VariableBitXOrLE {
    complex_gate: ComplexGateMembers,
    xor_le_gates: Vec<Rc<RefCell<XOrLE>>>,
}

#[allow(dead_code)]
impl VariableBitXOrLE {
    pub fn new(num_bits: usize) -> Rc<RefCell<Self>> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();
        let mut output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>> = Vec::new();
        let mut output_gates_logic: Vec<Rc<RefCell<dyn LogicGate>>> = Vec::new();

        let mut xor_le_gates = Vec::new();

        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            input_gates.push(SimpleInput::new(1, a_input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);

            xor_le_gates.push(
                XOrLE::new()
            );
        }

        for i in 0..num_bits {
            let b_input_tag = format!("b_{}", i);
            input_gates.push(SimpleInput::new(1, b_input_tag.as_str()));
        }

        let larger_output_gate = SimpleOutput::new("L");
        let equal_output_gate = SimpleOutput::new("E");
        output_gates.push(larger_output_gate.clone());
        output_gates.push(equal_output_gate.clone());
        output_gates_logic.push(larger_output_gate);
        output_gates_logic.push(equal_output_gate);

        let mut variable_bit_and = VariableBitXOrLE {
            complex_gate: ComplexGateMembers::new(
                2 * num_bits,
                num_bits + 2,
                GateType::VariableBitXOrLEType,
                input_gates,
                output_gates,
            ),
            xor_le_gates,
        };

        variable_bit_and.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        Rc::new(RefCell::new(variable_bit_and))
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    ) {
        for i in 0..num_bits {
            let xor_a_input_index = self.xor_le_gates[i].borrow_mut().get_index_from_tag("A");
            let xor_b_input_index = self.xor_le_gates[i].borrow_mut().get_index_from_tag("B");
            let xor_equal_input_index = self.xor_le_gates[i].borrow_mut().get_index_from_tag("ABOVE_E");
            let xor_larger_input_index = self.xor_le_gates[i].borrow_mut().get_index_from_tag("ABOVE_L");

            let c_output_index = self.xor_le_gates[i].borrow_mut().get_index_from_tag("C");

            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());

            self.complex_gate.input_gates[a_input_index].borrow_mut().connect_output_to_next_gate(
                0,
                xor_a_input_index,
                self.xor_le_gates[i].clone(),
            );

            self.complex_gate.input_gates[b_input_index].borrow_mut().connect_output_to_next_gate(
                0,
                xor_b_input_index,
                self.xor_le_gates[i].clone(),
            );

            self.xor_le_gates[i].borrow_mut().connect_output_to_next_gate(
                c_output_index,
                0,
                output_gates[i].clone(),
            );

            if i == 0 {
                self.xor_le_gates[i].borrow_mut().update_input_signal(
                    GateInput::new(
                        xor_equal_input_index,
                        HIGH,
                        UniqueID::zero_id(),
                    )
                );

                self.xor_le_gates[i].borrow_mut().update_input_signal(
                    GateInput::new(
                        xor_larger_input_index,
                        LOW,
                        UniqueID::zero_id(),
                    )
                );
            } else {
                let equal_output_index = self.xor_le_gates[i - 1].borrow_mut().get_index_from_tag("E");
                let a_larger_output_index = self.xor_le_gates[i - 1].borrow_mut().get_index_from_tag("A_L");

                self.xor_le_gates[i - 1].borrow_mut().connect_output_to_next_gate(
                    equal_output_index,
                    xor_equal_input_index,
                    self.xor_le_gates[i].clone(),
                );

                self.xor_le_gates[i - 1].borrow_mut().connect_output_to_next_gate(
                    a_larger_output_index,
                    xor_larger_input_index,
                    self.xor_le_gates[i].clone(),
                );
            }
        }

        let xor_equal_output_index = self.xor_le_gates[num_bits - 1].borrow_mut().get_index_from_tag("E");
        let xor_a_larger_output_index = self.xor_le_gates[num_bits - 1].borrow_mut().get_index_from_tag("A_L");
        let equal_output_index = self.get_index_from_tag("E");
        let larger_output_index = self.get_index_from_tag("L");

        self.xor_le_gates[num_bits - 1].borrow_mut().connect_output_to_next_gate(
            xor_equal_output_index,
            0,
            output_gates[equal_output_index].clone(),
        );

        self.xor_le_gates[num_bits - 1].borrow_mut().connect_output_to_next_gate(
            xor_a_larger_output_index,
            0,
            output_gates[larger_output_index].clone(),
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs(
            true,
            None,
        );
    }
}

impl LogicGate for VariableBitXOrLE {
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
    use std::collections::HashMap;
    use rand::Rng;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::test_stuff::run_multi_input_output_logic_gate;
    use super::*;

    fn test_half_adder(
        a: Signal,
        b: Signal,
        sum: Signal,
        carry: Signal,
    ) {
        let half_adder = HalfAdder::new();

        run_multi_input_output_logic_gate(
            vec![],
            vec![
                vec![sum, carry], //SUM, CARRY
            ],
            HashMap::from(
                [
                    ("A", vec![vec![a]]),
                    ("B", vec![vec![b]]),
                ]
            ),
            half_adder,
        );
    }

    fn test_full_adder(
        a: Signal,
        b: Signal,
        c_in: Signal,
        sum: Signal,
        carry: Signal,
    ) {
        let half_adder = FullAdder::new();

        run_multi_input_output_logic_gate(
            vec![],
            vec![
                vec![sum, carry], //SUM, CARRY
            ],
            HashMap::from(
                [
                    ("A", vec![vec![a]]),
                    ("B", vec![vec![b]]),
                    ("C_IN", vec![vec![c_in]]),
                ]
            ),
            half_adder,
        );
    }

    fn convert_char_to_signal_and_num(c: char) -> (Signal, usize) {
        if c == '0' {
            (LOW, 0)
        } else {
            (HIGH, 1)
        }
    }

    fn convert_num_to_signal(j: usize) -> Signal {
        if j == 0 {
            LOW
        } else {
            HIGH
        }
    }

    fn convert_bool_to_signal(b: bool) -> Signal {
        if !b {
            LOW
        } else {
            HIGH
        }
    }

    #[test]
    fn half_adder_low_low() {
        test_half_adder(
            LOW,
            LOW,
            LOW,
            LOW,
        );
    }

    #[test]
    fn half_adder_low_high() {
        test_half_adder(
            LOW,
            HIGH,
            HIGH,
            LOW,
        );
    }

    #[test]
    fn half_adder_high_low() {
        test_half_adder(
            HIGH,
            LOW,
            HIGH,
            LOW,
        );
    }

    #[test]
    fn half_adder_high_high() {
        test_half_adder(
            HIGH,
            HIGH,
            LOW,
            HIGH,
        );
    }

    #[test]
    fn full_adder_low_low_low() {
        test_full_adder(
            LOW,
            LOW,
            LOW,
            LOW,
            LOW,
        );
    }

    #[test]
    fn full_adder_low_low_high() {
        test_full_adder(
            LOW,
            LOW,
            HIGH,
            HIGH,
            LOW,
        );
    }

    #[test]
    fn full_adder_low_high_low() {
        test_full_adder(
            LOW,
            HIGH,
            LOW,
            HIGH,
            LOW,
        );
    }

    #[test]
    fn full_adder_low_high_high() {
        test_full_adder(
            LOW,
            HIGH,
            HIGH,
            LOW,
            HIGH,
        );
    }

    #[test]
    fn full_adder_high_low_low() {
        test_full_adder(
            HIGH,
            LOW,
            LOW,
            HIGH,
            LOW,
        );
    }

    #[test]
    fn full_adder_high_low_high() {
        test_full_adder(
            HIGH,
            LOW,
            HIGH,
            LOW,
            HIGH,
        );
    }

    #[test]
    fn full_adder_high_high_low() {
        test_full_adder(
            HIGH,
            HIGH,
            LOW,
            LOW,
            HIGH,
        );
    }

    #[test]
    fn full_adder_high_high_high() {
        test_full_adder(
            HIGH,
            HIGH,
            HIGH,
            HIGH,
            HIGH,
        );
    }

    fn convert_binary_to_vec(
        binary: &String,
    ) -> Vec<Signal> {
        let mut vec = Vec::new();
        for c in binary.chars() {
            if c == '0' {
                vec.push(LOW);
            } else {
                vec.push(HIGH);
            }
        }
        vec
    }

    #[test]
    fn variable_bit_adder_tests() {
        for _ in 0..10 {
            let num_bits = rand::thread_rng().gen_range(2..17);

            let high_number_range = usize::pow(2, num_bits as u32);
            let first_num = rand::thread_rng().gen_range(0..high_number_range);
            let second_num = rand::thread_rng().gen_range(0..high_number_range);
            let sum = first_num + second_num;

            let variable_bit_adder = VariableBitAdder::new(num_bits);

            let first_binary = format!("{:0width$b}", first_num, width = num_bits);
            let second_binary = format!("{:0width$b}", second_num, width = num_bits);
            let sum_binary = format!("{:0width$b}", sum, width = num_bits + 1);

            //Leave these here in case it fails the number will be reproducible.
            println!("num_bits: {}", num_bits);
            println!("{}", first_binary);
            println!("{}", second_binary);
            println!("{}", sum_binary);

            let a_input = convert_binary_to_vec(&first_binary);
            let b_input = convert_binary_to_vec(&second_binary);
            let mut output = convert_binary_to_vec(&sum_binary);

            let carry_signal = output.remove(0);
            // output.push(carry_signal); TODO: uncomment

            run_multi_input_output_logic_gate(
                vec![],
                vec![
                    output
                ],
                HashMap::from(
                    [
                        ("a", vec![a_input]),
                        ("b", vec![b_input]),
                        ("C_IN", vec![vec![LOW]]),
                    ]
                ),
                variable_bit_adder,
            );
        }
    }

    #[test]
    fn variable_bit_shift_tests() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(1..16);

            let high_number_range = usize::pow(2, num_bits as u32);
            let first_num = rand::thread_rng().gen_range(0..high_number_range);
            let left_shift = rand::thread_rng().gen_bool(0.5);
            let result =
                if left_shift {
                    first_num << 1
                } else {
                    first_num >> 1
                };

            let variable_bit_left_shift: Rc<RefCell<dyn LogicGate>> =
                if left_shift {
                    VariableBitShiftLeft::<true>::new(num_bits)
                } else {
                    VariableBitShiftLeft::<false>::new(num_bits)
                };

            let first_binary = format!("{:0width$b}", first_num, width = num_bits);
            let result_binary = format!("{:0width$b}", result, width = num_bits);

            //Leave these here in case it fails the number will be reproducible.
            println!("num_bits: {}", num_bits);
            println!("left_shift: {}", left_shift);
            println!("{}", first_binary);
            println!("{}", result_binary);

            let a_input = convert_binary_to_vec(&first_binary);
            let result_output = convert_binary_to_vec(&result_binary);

            run_multi_input_output_logic_gate(
                vec![
                    a_input
                ],
                vec![
                    result_output
                ],
                HashMap::from(
                    []
                ),
                variable_bit_left_shift,
            );
        }
    }

    #[test]
    fn variable_bit_not_tests() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(1..16);

            let high_number_range = usize::pow(2, num_bits as u32);
            let num: u32 = rand::thread_rng().gen_range(0..high_number_range) as u32;
            let result: u32 = !num;

            let variable_bit_not = VariableBitNot::new(num_bits);

            let first_binary = format!("{:0width$b}", num, width = num_bits);
            let result_binary = format!("{:0width$b}", result, width = num_bits);
            let result_binary = &result_binary[32 - num_bits..].to_string();

            //Leave these here in case it fails the number will be reproducible.
            println!("num_bits: {}", num_bits);
            println!("{}", first_binary);
            println!("{}", result_binary);

            let input = convert_binary_to_vec(&first_binary);
            let output = convert_binary_to_vec(result_binary);

            run_multi_input_output_logic_gate(
                vec![
                    input
                ],
                vec![
                    output
                ],
                HashMap::from(
                    []
                ),
                variable_bit_not,
            );
        }
    }


    #[test]
    fn variable_bit_and_or_tests() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let high_number_range = usize::pow(2, num_bits as u32);
            let a_input = rand::thread_rng().gen_range(0..high_number_range);
            let b_input = rand::thread_rng().gen_range(0..high_number_range);

            let and_gate = rand::thread_rng().gen_bool(0.5);
            let result =
                if and_gate {
                    a_input & b_input
                } else {
                    a_input | b_input
                };

            let variable_bit_not: Rc<RefCell<dyn LogicGate>> =
                if and_gate {
                    VariableBitAnd::new(num_bits)
                } else {
                    VariableBitOr::new(num_bits)
                };

            let a_binary = format!("{:0width$b}", a_input, width = num_bits);
            let b_binary = format!("{:0width$b}", b_input, width = num_bits);
            let result_binary = format!("{:0width$b}", result, width = num_bits);

            //Leave these here in case it fails the number will be reproducible.
            println!("num_bits: {}", num_bits);
            println!("{}", a_input);
            println!("{}", b_input);
            println!("{}", result_binary);

            let a_input = convert_binary_to_vec(&a_binary);
            let b_input = convert_binary_to_vec(&b_binary);
            let output = convert_binary_to_vec(&result_binary);

            run_multi_input_output_logic_gate(
                vec![],
                vec![
                    output
                ],
                HashMap::from(
                    [
                        ("a", vec![a_input]),
                        ("b", vec![b_input]),
                    ]
                ),
                variable_bit_not,
            );
        }
    }

    #[test]
    fn xor_le_test() {
        for i in 0..16 {
            let permutation: Vec<char> = format!("{:0width$b}", i, width = 4).chars().collect();

            //Leave in case the gate breaks later.
            println!("{:?}", permutation);

            let xor_le = XOrLE::new();

            let (a_input_signal, a_input_num) = convert_char_to_signal_and_num(permutation[0]);
            let (b_input_signal, b_input_num) = convert_char_to_signal_and_num(permutation[1]);
            let (above_equal_input_signal, _) = convert_char_to_signal_and_num(permutation[2]);
            let (above_larger_input_signal, _) = convert_char_to_signal_and_num(permutation[3]);

            let c_output = convert_num_to_signal(a_input_num ^ b_input_num);
            let equal = convert_bool_to_signal(above_equal_input_signal == HIGH && a_input_num == b_input_num);
            let larger = convert_bool_to_signal(above_larger_input_signal == HIGH || (above_equal_input_signal == HIGH && a_input_num > b_input_num));

            run_multi_input_output_logic_gate(
                vec![],
                vec![
                    vec![c_output, equal, larger]
                ],
                HashMap::from(
                    [
                        ("A", vec![vec![a_input_signal]]),
                        ("B", vec![vec![b_input_signal]]),
                        ("ABOVE_E", vec![vec![above_equal_input_signal]]),
                        ("ABOVE_L", vec![vec![above_larger_input_signal]]),
                    ]
                ),
                xor_le,
            );
        }
    }

    #[test]
    fn variable_bit_xor_le() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let high_number_range = usize::pow(2, num_bits as u32);
            let a_input = rand::thread_rng().gen_range(0..high_number_range);
            let b_input = rand::thread_rng().gen_range(0..high_number_range);
            let result = a_input ^ b_input;

            let variable_bit_xor_le = VariableBitXOrLE::new(num_bits);

            let a_binary = format!("{:0width$b}", a_input, width = num_bits);
            let b_binary = format!("{:0width$b}", b_input, width = num_bits);
            let result_binary = format!("{:0width$b}", result, width = num_bits);

            //Leave these here in case it fails the number will be reproducible.
            println!("num_bits: {}", num_bits);
            println!("a: {} 0b: {}", a_input, a_binary);
            println!("b: {} 0b: {}", b_input, b_binary);
            println!("fin: {} 0b: {}", result, result_binary);

            let a_input_signals = convert_binary_to_vec(&a_binary);
            let b_input_signals = convert_binary_to_vec(&b_binary);
            let mut output = convert_binary_to_vec(&result_binary);

            output.push(
                convert_bool_to_signal(a_input > b_input),
            );

            output.push(
                convert_bool_to_signal(a_input == b_input),
            );

            run_multi_input_output_logic_gate(
                vec![],
                vec![
                    output
                ],
                HashMap::from(
                    [
                        ("a", vec![a_input_signals]),
                        ("b", vec![b_input_signals]),
                    ]
                ),
                variable_bit_xor_le,
            );
        }
    }
}

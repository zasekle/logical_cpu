use std::sync::{MutexGuard};
use std::time::Instant;
use crate::ALU_TIME;
use crate::logic::basic_gates::{And, ControlledBuffer, Not, Or, Splitter, XOr};
use crate::logic::complex_logic::SignalGatekeeper;

use crate::logic::foundations::{GateInput, GateOutputState, LogicGate, UniqueID, GateLogicError, GateType, InputSignalReturn, Signal, ComplexGateMembers, build_simple_inputs_and_outputs, connect_gates};
use crate::logic::foundations::Signal::{HIGH, LOW_};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
use crate::logic::processor_components::{VariableBitRegister, VariableDecoder};
use crate::shared_mutex::{new_shared_mutex, SharedMutex};

pub struct HalfAdder {
    complex_gate: ComplexGateMembers,
    sum_xor_gate: SharedMutex<XOr>,
    carry_and_gate: SharedMutex<And>,
}

#[allow(dead_code)]
impl HalfAdder {
    pub fn new() -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(half_adder)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let a_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("A")].clone();
        let b_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("B")].clone();

        connect_gates(
            a_input_gate.clone(),
            0,
            self.sum_xor_gate.clone(),
            0,
        );

        connect_gates(
            a_input_gate.clone(),
            1,
            self.carry_and_gate.clone(),
            1,
        );

        connect_gates(
            b_input_gate.clone(),
            0,
            self.sum_xor_gate.clone(),
            1,
        );

        connect_gates(
            b_input_gate.clone(),
            1,
            self.carry_and_gate.clone(),
            0,
        );

        let sum_output_gate_index = self.get_index_from_tag("S");
        connect_gates(
            self.sum_xor_gate.clone(),
            0,
            output_gates[sum_output_gate_index].clone(),
            0,
        );

        let carry_output_gate_index = self.get_index_from_tag("C");
        connect_gates(
            self.carry_and_gate.clone(),
            0,
            output_gates[carry_output_gate_index].clone(),
            0,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for HalfAdder {
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

pub struct FullAdder {
    complex_gate: ComplexGateMembers,
    input_half_added: SharedMutex<HalfAdder>,
    carry_half_adder: SharedMutex<HalfAdder>,
    or_gate: SharedMutex<Or>,
}

#[allow(dead_code)]
impl FullAdder {
    pub fn new() -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(full_adder)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let a_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("A")].clone();
        let b_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("B")].clone();
        let c_in_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("C_IN")].clone();

        let a_input_index = self.input_half_added.lock().unwrap().get_index_from_tag("A");
        connect_gates(
            a_input_gate.clone(),
            0,
            self.input_half_added.clone(),
            a_input_index,
        );

        let b_input_index = self.input_half_added.lock().unwrap().get_index_from_tag("B");
        connect_gates(
            b_input_gate.clone(),
            0,
            self.input_half_added.clone(),
            b_input_index,
        );

        let a_input_index = self.carry_half_adder.lock().unwrap().get_index_from_tag("A");
        let s_output_index = self.input_half_added.lock().unwrap().get_index_from_tag("S");
        connect_gates(
            self.input_half_added.clone(),
            s_output_index,
            self.carry_half_adder.clone(),
            a_input_index,
        );

        let b_input_index = self.carry_half_adder.lock().unwrap().get_index_from_tag("B");
        connect_gates(
            c_in_input_gate.clone(),
            0,
            self.carry_half_adder.clone(),
            b_input_index,
        );

        let s_out_output_index = self.carry_half_adder.lock().unwrap().get_index_from_tag("S");
        let s_out_index = self.get_index_from_tag("S");
        connect_gates(
            self.carry_half_adder.clone(),
            s_out_output_index,
            output_gates[s_out_index].clone(),
            0,
        );

        let c_out_output_index = self.carry_half_adder.lock().unwrap().get_index_from_tag("C");
        connect_gates(
            self.carry_half_adder.clone(),
            c_out_output_index,
            self.or_gate.clone(),
            0,
        );

        let c_out_output_index = self.input_half_added.lock().unwrap().get_index_from_tag("C");
        connect_gates(
            self.input_half_added.clone(),
            c_out_output_index,
            self.or_gate.clone(),
            1,
        );

        let c_out_index = self.get_index_from_tag("C_OUT");
        connect_gates(
            self.or_gate.clone(),
            0,
            output_gates[c_out_index].clone(),
            0,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for FullAdder {
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

pub struct VariableBitAdder {
    complex_gate: ComplexGateMembers,
    full_adders: Vec<SharedMutex<FullAdder>>,
}

#[allow(dead_code)]
impl VariableBitAdder {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);


        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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


        new_shared_mutex(variable_bit_adder)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {

        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);
            let output_tag = format!("o_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());
            let output_index = self.get_index_from_tag(output_tag.as_str());

            let mut_full_adder = self.full_adders[i].lock().unwrap();

            let a_adder_index = mut_full_adder.get_index_from_tag("A");
            let b_adder_index = mut_full_adder.get_index_from_tag("B");
            let s_adder_index = mut_full_adder.get_index_from_tag("S");
            let c_in_adder_index = mut_full_adder.get_index_from_tag("C_IN");

            //Must be dropped or deadlock will occur when the gate is passed to connect_gates below.
            drop(mut_full_adder);

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                0,
                self.full_adders[i].clone(),
                a_adder_index,
            );

            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                0,
                self.full_adders[i].clone(),
                b_adder_index,
            );

            connect_gates(
                self.full_adders[i].clone(),
                s_adder_index,
                output_gates[output_index].clone(),
                0,
            );

            if i == 0 { //First adder in the line.
                let c_in_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("C_IN")].clone();
                let c_in_adder_index = self.full_adders[i].lock().unwrap().get_index_from_tag("C_IN");

                connect_gates(
                    c_in_input_gate,
                    0,
                    self.full_adders[i].clone(),
                    c_in_adder_index,
                );
            } else {
                let c_out_adder_index = self.full_adders[i - 1].lock().unwrap().get_index_from_tag("C_OUT");
                connect_gates(
                    self.full_adders[i - 1].clone(),
                    c_out_adder_index,
                    self.full_adders[i].clone(),
                    c_in_adder_index,
                );
            }
        }


        let c_out_adder_index = self.full_adders[num_bits - 1].lock().unwrap().get_index_from_tag("C_OUT");
        let c_out_output_index = self.get_index_from_tag("C_OUT");
        connect_gates(
            self.full_adders[num_bits - 1].clone(),
            c_out_adder_index,
            output_gates[c_out_output_index].clone(),
            0,
        );


        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for VariableBitAdder {
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

pub struct VariableBitShiftLeft<const LEFT_SHIFT: bool> {
    complex_gate: ComplexGateMembers,
    first_register: SharedMutex<VariableBitRegister>,
    second_register: SharedMutex<VariableBitRegister>,
}

#[allow(dead_code)]
impl<const LEFT_SHIFT: bool> VariableBitShiftLeft<LEFT_SHIFT> {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(variable_bit_shift_left)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        fn tie_register_bits_high(
            mut register: MutexGuard<VariableBitRegister>
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

        tie_register_bits_high(self.first_register.lock().unwrap());
        tie_register_bits_high(self.second_register.lock().unwrap());

        for i in 0..num_bits {
            connect_gates(
                self.complex_gate.input_gates[i].clone(),
                0,
                self.first_register.clone(),
                i,
            );

            if (LEFT_SHIFT && i == (num_bits - 1))
                || (!LEFT_SHIFT && i == 0) {
                let shift_out_index = self.get_index_from_tag("S_OUT");
                connect_gates(
                    self.first_register.clone(),
                    i,
                    output_gates[shift_out_index].clone(),
                    0,
                );
            } else {
                let second_index =
                    if LEFT_SHIFT {
                        i + 1
                    } else {
                        i - 1
                    };

                connect_gates(
                    self.first_register.clone(),
                    i,
                    self.second_register.clone(),
                    second_index,
                );
            }

            connect_gates(
                self.second_register.clone(),
                i,
                output_gates[i].clone(),
                0,
            );
        }

        let shift_in_index = self.get_index_from_tag("S_IN");
        let second_index =
            if LEFT_SHIFT {
                0
            } else {
                num_bits - 1
            };

        connect_gates(
            self.complex_gate.input_gates[shift_in_index].clone(),
            0,
            self.second_register.clone(),
            second_index,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl<const LEFT_SHIFT: bool> LogicGate for VariableBitShiftLeft<LEFT_SHIFT> {
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

pub struct VariableBitNot {
    complex_gate: ComplexGateMembers,
    not_gates: Vec<SharedMutex<Not>>,
}

#[allow(dead_code)]
impl VariableBitNot {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(variable_bit_not)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        for i in 0..num_bits {
            connect_gates(
                self.complex_gate.input_gates[i].clone(),
                0,
                self.not_gates[i].clone(),
                0,
            );

            connect_gates(
                self.not_gates[i].clone(),
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

impl LogicGate for VariableBitNot {
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

pub struct VariableBitAnd {
    complex_gate: ComplexGateMembers,
    and_gates: Vec<SharedMutex<And>>,
}

#[allow(dead_code)]
impl VariableBitAnd {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(variable_bit_and)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                0,
                self.and_gates[i].clone(),
                0,
            );

            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                0,
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

impl LogicGate for VariableBitAnd {
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

pub struct VariableBitOr {
    complex_gate: ComplexGateMembers,
    or_gates: Vec<SharedMutex<Or>>,
}

#[allow(dead_code)]
impl VariableBitOr {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(variable_bit_and)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                0,
                self.or_gates[i].clone(),
                0,
            );

            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                0,
                self.or_gates[i].clone(),
                1,
            );

            connect_gates(
                self.or_gates[i].clone(),
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

impl LogicGate for VariableBitOr {
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

pub struct XOrLE {
    complex_gate: ComplexGateMembers,
    xor_gate: SharedMutex<XOr>,
    not_gate: SharedMutex<Not>,
    equal_and_gate: SharedMutex<And>,
    middle_and_gate: SharedMutex<And>,
    or_gate: SharedMutex<Or>,
}

#[allow(dead_code)]
impl XOrLE {
    pub fn new() -> SharedMutex<Self> {
        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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

        new_shared_mutex(variable_bit_and)
    }

    fn build_and_prime_circuit(
        &mut self,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let a_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("A")].clone();
        let b_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("B")].clone();
        let equal_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("ABOVE_E")].clone();
        let larger_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("ABOVE_L")].clone();

        let c_output_gate = output_gates[self.get_index_from_tag("C")].clone();
        let equal_output_gate = output_gates[self.get_index_from_tag("E")].clone();
        let a_larger_output_gate = output_gates[self.get_index_from_tag("A_L")].clone();

        connect_gates(
            a_input_gate.clone(),
            0,
            self.xor_gate.clone(),
            0,
        );

        connect_gates(
            a_input_gate.clone(),
            1,
            self.middle_and_gate.clone(),
            1,
        );

        connect_gates(
            b_input_gate.clone(),
            0,
            self.xor_gate.clone(),
            1,
        );

        connect_gates(
            self.xor_gate.clone(),
            0,
            self.not_gate.clone(),
            0,
        );

        connect_gates(
            self.xor_gate.clone(),
            1,
            self.middle_and_gate.clone(),
            2,
        );

        connect_gates(
            self.xor_gate.clone(),
            2,
            c_output_gate.clone(),
            2,
        );

        connect_gates(
            self.not_gate.clone(),
            0,
            self.equal_and_gate.clone(),
            0,
        );

        connect_gates(
            self.equal_and_gate.clone(),
            0,
            equal_output_gate.clone(),
            0,
        );

        connect_gates(
            equal_input_gate.clone(),
            0,
            self.middle_and_gate.clone(),
            0,
        );

        connect_gates(
            equal_input_gate.clone(),
            1,
            self.equal_and_gate.clone(),
            1,
        );

        connect_gates(
            self.middle_and_gate.clone(),
            0,
            self.or_gate.clone(),
            0,
        );

        connect_gates(
            larger_input_gate.clone(),
            0,
            self.or_gate.clone(),
            1,
        );

        connect_gates(
            self.or_gate.clone(),
            0,
            a_larger_output_gate.clone(),
            0,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for XOrLE {
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

pub struct VariableBitXOrLE {
    complex_gate: ComplexGateMembers,
    xor_le_gates: Vec<SharedMutex<XOrLE>>,
}

#[allow(dead_code)]
impl VariableBitXOrLE {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

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
        // larger_output_gate.lock().unwrap().toggle_output_printing(true);
        // equal_output_gate.lock().unwrap().toggle_output_printing(true);
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

        new_shared_mutex(variable_bit_and)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        for i in 0..num_bits {
            let xor_a_input_index = self.xor_le_gates[i].lock().unwrap().get_index_from_tag("A");
            let xor_b_input_index = self.xor_le_gates[i].lock().unwrap().get_index_from_tag("B");

            let c_output_index = self.xor_le_gates[i].lock().unwrap().get_index_from_tag("C");

            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                0,
                self.xor_le_gates[i].clone(),
                xor_a_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                0,
                self.xor_le_gates[i].clone(),
                xor_b_input_index,
            );

            connect_gates(
                self.xor_le_gates[i].clone(),
                c_output_index,
                output_gates[i].clone(),
                0,
            );

            if i == num_bits - 1 {
                let xor_equal_input_index = self.xor_le_gates[i].lock().unwrap().get_index_from_tag("ABOVE_E");
                let xor_larger_input_index = self.xor_le_gates[i].lock().unwrap().get_index_from_tag("ABOVE_L");

                self.xor_le_gates[i].lock().unwrap().update_input_signal(
                    GateInput::new(
                        xor_equal_input_index,
                        HIGH,
                        UniqueID::zero_id(),
                    )
                );

                self.xor_le_gates[i].lock().unwrap().update_input_signal(
                    GateInput::new(
                        xor_larger_input_index,
                        LOW_,
                        UniqueID::zero_id(),
                    )
                );
            } else {
                let equal_output_index = self.xor_le_gates[i + 1].lock().unwrap().get_index_from_tag("E");
                let a_larger_output_index = self.xor_le_gates[i + 1].lock().unwrap().get_index_from_tag("A_L");
                let xor_equal_input_index = self.xor_le_gates[i].lock().unwrap().get_index_from_tag("ABOVE_E");
                let xor_larger_input_index = self.xor_le_gates[i].lock().unwrap().get_index_from_tag("ABOVE_L");

                connect_gates(
                    self.xor_le_gates[i + 1].clone(),
                    equal_output_index,
                    self.xor_le_gates[i].clone(),
                    xor_equal_input_index,
                );

                connect_gates(
                    self.xor_le_gates[i + 1].clone(),
                    a_larger_output_index,
                    self.xor_le_gates[i].clone(),
                    xor_larger_input_index,
                );
            }
        }

        let xor_equal_output_index = self.xor_le_gates[0].lock().unwrap().get_index_from_tag("E");
        let xor_a_larger_output_index = self.xor_le_gates[0].lock().unwrap().get_index_from_tag("A_L");
        let equal_output_index = self.get_index_from_tag("E");
        let larger_output_index = self.get_index_from_tag("L");

        connect_gates(
            self.xor_le_gates[0].clone(),
            xor_equal_output_index,
            output_gates[equal_output_index].clone(),
            0,
        );

        connect_gates(
            self.xor_le_gates[0].clone(),
            xor_a_larger_output_index,
            output_gates[larger_output_index].clone(),
            0,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for VariableBitXOrLE {
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

pub struct VariableBitZ {
    complex_gate: ComplexGateMembers,
    or_gate: SharedMutex<Or>,
    not_gate: SharedMutex<Not>,
}

#[allow(dead_code)]
impl VariableBitZ {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        for i in 0..num_bits {
            let input_tag = format!("i_{}", i);
            input_gates.push(SimpleInput::new(1, input_tag.as_str()));
        }

        let output_gate = SimpleOutput::new("O");
        output_gates.push(output_gate.clone());
        output_gates_logic.push(output_gate);

        let mut variable_bit_z = VariableBitZ {
            complex_gate: ComplexGateMembers::new(
                num_bits,
                1,
                GateType::VariableBitZType,
                input_gates,
                output_gates,
            ),
            or_gate: Or::new(num_bits, 1),
            not_gate: Not::new(1),
        };

        variable_bit_z.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        new_shared_mutex(variable_bit_z)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        for i in 0..num_bits {
            connect_gates(
                self.complex_gate.input_gates[i].clone(),
                0,
                self.or_gate.clone(),
                i,
            );
        }

        connect_gates(
            self.or_gate.clone(),
            0,
            self.not_gate.clone(),
            0,
        );

        connect_gates(
            self.not_gate.clone(),
            0,
            output_gates[0].clone(),
            0,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for VariableBitZ {
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

pub struct VariableBitEnable {
    complex_gate: ComplexGateMembers,
    control_buffer: SharedMutex<ControlledBuffer>,
}

#[allow(dead_code)]
impl VariableBitEnable {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        build_simple_inputs_and_outputs(
            num_bits,
            &mut input_gates,
            &mut output_gates,
            &mut output_gates_logic,
        );

        input_gates.push(SimpleInput::new(1, "E"));

        let mut variable_bit_enable = VariableBitEnable {
            complex_gate: ComplexGateMembers::new(
                num_bits + 1,
                num_bits,
                GateType::VariableBitEnableType,
                input_gates,
                output_gates,
            ),
            control_buffer: ControlledBuffer::new(num_bits),
        };

        variable_bit_enable.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        new_shared_mutex(variable_bit_enable)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let e_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("E")].clone();
        let controlled_buffer_enable_index = self.control_buffer.lock().unwrap().get_index_from_tag("E");

        connect_gates(
            e_input_gate.clone(),
            0,
            self.control_buffer.clone(),
            controlled_buffer_enable_index,
        );

        for i in 0..num_bits {
            connect_gates(
                self.complex_gate.input_gates[i].clone(),
                0,
                self.control_buffer.clone(),
                i,
            );

            connect_gates(
                self.control_buffer.clone(),
                i,
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

impl LogicGate for VariableBitEnable {
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

#[allow(dead_code)]
enum AluOperations {
    None,
    XOrLe,
    Or,
    And,
    Not,
    Shl,
    Shr,
    Adder,
}

#[allow(dead_code)]
struct AluReturns {
    a: Vec<Signal>,
    b: Vec<Signal>,
    c: Vec<Signal>,
}

#[allow(dead_code)]
impl AluReturns {
    fn new(a: Signal, b: Signal, c: Signal) -> Self {
        AluReturns { a: vec![a], b: vec![b], c: vec![c] }
    }
}

//Rules for the ALU input/output.
// idx 0 is the least significant bit
// idx 7 is the most significant bit
#[allow(dead_code)]
impl AluOperations {
    fn get_vectors(alu_operation: AluOperations) -> AluReturns {
        match alu_operation {
            AluOperations::None => AluReturns::new(HIGH, HIGH, HIGH),
            AluOperations::XOrLe => AluReturns::new(HIGH, HIGH, LOW_),
            AluOperations::Or => AluReturns::new(HIGH, LOW_, HIGH),
            AluOperations::And => AluReturns::new(HIGH, LOW_, LOW_),
            AluOperations::Not => AluReturns::new(LOW_, HIGH, HIGH),
            AluOperations::Shl => AluReturns::new(LOW_, HIGH, LOW_),
            AluOperations::Shr => AluReturns::new(LOW_, LOW_, HIGH),
            AluOperations::Adder => AluReturns::new(LOW_, LOW_, LOW_),
        }
    }
}

pub struct ArithmeticLogicUnit {
    complex_gate: ComplexGateMembers,
    xor_le: SharedMutex<VariableBitXOrLE>,
    or: SharedMutex<VariableBitOr>,
    and: SharedMutex<VariableBitAnd>,
    not: SharedMutex<VariableBitNot>,
    shift_left: SharedMutex<VariableBitShiftLeft<true>>,
    shift_right: SharedMutex<VariableBitShiftLeft<false>>,
    adder: SharedMutex<VariableBitAdder>,
    decoder: SharedMutex<VariableDecoder>,
    decoder_splitters: Vec<SharedMutex<Splitter>>,
    enable_gates: Vec<SharedMutex<VariableBitEnable>>,
    enable_splitters: Vec<SharedMutex<Splitter>>,
    shl_controlled_buffer: SharedMutex<ControlledBuffer>,
    shr_controlled_buffer: SharedMutex<ControlledBuffer>,
    adder_controlled_buffer: SharedMutex<ControlledBuffer>,
    z: SharedMutex<VariableBitZ>,
    input_signal_gatekeepers: Vec<SharedMutex<SignalGatekeeper>>,
    carry_in_signal_gatekeepers: Vec<SharedMutex<SignalGatekeeper>>,
}

#[allow(dead_code)]
impl ArithmeticLogicUnit {
    pub fn new(num_bits: usize) -> SharedMutex<Self> {
        assert_ne!(num_bits, 0);


        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        let mut output_gates_logic: Vec<SharedMutex<dyn LogicGate>> = Vec::new();

        for i in 0..num_bits {
            let a_input_tag = format!("a_{}", i);
            input_gates.push(SimpleInput::new(7, a_input_tag.as_str()));

            let output_tag = format!("o_{}", i);
            let output_gate = SimpleOutput::new(output_tag.as_str());
            output_gates.push(output_gate.clone());
            output_gates_logic.push(output_gate);
        }

        for i in 0..num_bits {
            let b_input_tag = format!("b_{}", i);
            input_gates.push(SimpleInput::new(4, b_input_tag.as_str()));
        }

        input_gates.push(SimpleInput::new(1, "A"));
        input_gates.push(SimpleInput::new(1, "B"));
        input_gates.push(SimpleInput::new(1, "C"));
        input_gates.push(SimpleInput::new(3, "C_IN"));

        let a_larger_output_gate = SimpleOutput::new("A_L");
        let equal_output_gate = SimpleOutput::new("EQ");
        let zero_output_gate = SimpleOutput::new("Z");
        let carry_out_output_gate = SimpleOutput::new("C_OUT");
        output_gates.push(a_larger_output_gate.clone());
        output_gates.push(equal_output_gate.clone());
        output_gates.push(zero_output_gate.clone());
        output_gates.push(carry_out_output_gate.clone());
        output_gates_logic.push(a_larger_output_gate);
        output_gates_logic.push(equal_output_gate);
        output_gates_logic.push(zero_output_gate);
        output_gates_logic.push(carry_out_output_gate);

        let mut enable_gates = Vec::new();
        let mut enable_splitters = Vec::new();
        for _ in 0..7 {
            enable_gates.push(VariableBitEnable::new(num_bits));
            enable_splitters.push(Splitter::new(num_bits, 2));
        }

        let mut input_signal_gatekeepers = Vec::new();

        for _ in 0..9 {
            input_signal_gatekeepers.push(SignalGatekeeper::new(num_bits));
        }

        let mut carry_in_signal_gatekeepers = Vec::new();

        for _ in 0..3 {
            carry_in_signal_gatekeepers.push(SignalGatekeeper::new(1));
        }

        let mut decoder_splitters = Vec::new();
        decoder_splitters.push(Splitter::new(1, 3)); // 0; Or
        decoder_splitters.push(Splitter::new(1, 3)); // 1; And
        decoder_splitters.push(Splitter::new(1, 2)); // 2; Not
        decoder_splitters.push(Splitter::new(1, 4)); // 3; SHL
        decoder_splitters.push(Splitter::new(1, 4)); // 4; SHR
        decoder_splitters.push(Splitter::new(1, 5)); // 5: Adder


        let xor_le= VariableBitXOrLE::new(num_bits);

        let or= VariableBitOr::new(num_bits);

        let and= VariableBitAnd::new(num_bits);

        let not= VariableBitNot::new(num_bits);

        let shift_left= VariableBitShiftLeft::<true>::new(num_bits);

        let shift_right= VariableBitShiftLeft::<false>::new(num_bits);

        let adder= VariableBitAdder::new(num_bits);

        let decoder= VariableDecoder::new(3);


        let mut arithmetic_logic_unit = ArithmeticLogicUnit {
            complex_gate: ComplexGateMembers::new(
                2 * num_bits + 4,
                num_bits + 4,
                GateType::ArithmeticLogicUnitType,
                input_gates,
                output_gates,
            ),
            xor_le,
            or,
            and,
            not,
            shift_left,
            shift_right,
            adder,
            decoder,
            decoder_splitters,
            enable_gates,
            enable_splitters,
            shl_controlled_buffer: ControlledBuffer::new(num_bits),
            shr_controlled_buffer: ControlledBuffer::new(num_bits),
            adder_controlled_buffer: ControlledBuffer::new(num_bits),
            z: VariableBitZ::new(num_bits),
            input_signal_gatekeepers,
            carry_in_signal_gatekeepers,
        };


        arithmetic_logic_unit.build_and_prime_circuit(
            num_bits,
            output_gates_logic,
        );

        new_shared_mutex(arithmetic_logic_unit)
    }

    fn build_and_prime_circuit(
        &mut self,
        num_bits: usize,
        output_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let a_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("A")].clone();
        let b_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("B")].clone();
        let c_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("C")].clone();
        let c_in_input_gate = self.complex_gate.input_gates[self.get_index_from_tag("C_IN")].clone();


        for i in 0..num_bits {
            let input_tag = format!("i_{}", i);
            let a_input_tag = format!("a_{}", i);
            let b_input_tag = format!("b_{}", i);

            let a_input_index = self.get_index_from_tag(a_input_tag.as_str());
            let b_input_index = self.get_index_from_tag(b_input_tag.as_str());

            //A Input -> Signal Gatekeepers & Signal Gatekeepers -> Arithmetic gates

            //Xor_le doesn't get a signal gatekeeper so that the larger and equal outputs will always
            // be correct.
            let xor_a_input_index = self.xor_le.lock().unwrap().get_index_from_tag(a_input_tag.as_str());
            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                0,
                self.xor_le.clone(),
                xor_a_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                1,
                self.input_signal_gatekeepers[0].clone(),
                i,
            );

            let or_a_input_index = self.or.lock().unwrap().get_index_from_tag(a_input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[0].clone(),
                i,
                self.or.clone(),
                or_a_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                2,
                self.input_signal_gatekeepers[1].clone(),
                i,
            );

            let and_a_input_index = self.and.lock().unwrap().get_index_from_tag(a_input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[1].clone(),
                i,
                self.and.clone(),
                and_a_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                3,
                self.input_signal_gatekeepers[2].clone(),
                i,
            );

            let not_a_input_index = self.not.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[2].clone(),
                i,
                self.not.clone(),
                not_a_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                4,
                self.input_signal_gatekeepers[3].clone(),
                i,
            );

            let shl_a_input_index = self.shift_left.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[3].clone(),
                i,
                self.shift_left.clone(),
                shl_a_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                5,
                self.input_signal_gatekeepers[4].clone(),
                i,
            );

            let shr_a_input_index = self.shift_right.lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[4].clone(),
                i,
                self.shift_right.clone(),
                shr_a_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[a_input_index].clone(),
                6,
                self.input_signal_gatekeepers[5].clone(),
                i,
            );

            let adder_a_input_index = self.adder.lock().unwrap().get_index_from_tag(a_input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[5].clone(),
                i,
                self.adder.clone(),
                adder_a_input_index,
            );

            //B Input -> Signal Gatekeepers & Signal Gatekeepers -> Arithmetic gates

            // Xor does not get a signal gatekeeper so that the larger and equal bits will be true.
            let xor_b_input_index = self.xor_le.lock().unwrap().get_index_from_tag(b_input_tag.as_str());
            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                0,
                self.xor_le.clone(),
                xor_b_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                1,
                self.input_signal_gatekeepers[6].clone(),
                i,
            );

            let or_b_input_index = self.or.lock().unwrap().get_index_from_tag(b_input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[6].clone(),
                i,
                self.or.clone(),
                or_b_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                2,
                self.input_signal_gatekeepers[7].clone(),
                i,
            );

            let and_b_input_index = self.and.lock().unwrap().get_index_from_tag(b_input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[7].clone(),
                i,
                self.and.clone(),
                and_b_input_index,
            );

            connect_gates(
                self.complex_gate.input_gates[b_input_index].clone(),
                3,
                self.input_signal_gatekeepers[8].clone(),
                i,
            );

            let adder_b_input_index = self.adder.lock().unwrap().get_index_from_tag(b_input_tag.as_str());
            connect_gates(
                self.input_signal_gatekeepers[8].clone(),
                i,
                self.adder.clone(),
                adder_b_input_index,
            );

            let output_tag = format!("o_{}", i);

            //Arithmetic gates -> Enable gates
            let xor_output_index = self.xor_le.lock().unwrap().get_index_from_tag(output_tag.as_str());
            let enable_input_index = self.enable_gates[0].lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.xor_le.clone(),
                xor_output_index,
                self.enable_gates[0].clone(),
                enable_input_index,
            );

            let or_output_index = self.or.lock().unwrap().get_index_from_tag(output_tag.as_str());
            let enable_input_index = self.enable_gates[1].lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.or.clone(),
                or_output_index,
                self.enable_gates[1].clone(),
                enable_input_index,
            );

            let and_output_index = self.and.lock().unwrap().get_index_from_tag(output_tag.as_str());
            let enable_input_index = self.enable_gates[2].lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.and.clone(),
                and_output_index,
                self.enable_gates[2].clone(),
                enable_input_index,
            );

            let not_output_index = self.not.lock().unwrap().get_index_from_tag(output_tag.as_str());
            let enable_input_index = self.enable_gates[3].lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.not.clone(),
                not_output_index,
                self.enable_gates[3].clone(),
                enable_input_index,
            );

            let shl_output_index = self.shift_left.lock().unwrap().get_index_from_tag(output_tag.as_str());
            let enable_input_index = self.enable_gates[4].lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.shift_left.clone(),
                shl_output_index,
                self.enable_gates[4].clone(),
                enable_input_index,
            );

            let shr_output_index = self.shift_right.lock().unwrap().get_index_from_tag(output_tag.as_str());
            let enable_input_index = self.enable_gates[5].lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.shift_right.clone(),
                shr_output_index,
                self.enable_gates[5].clone(),
                enable_input_index,
            );

            let adder_output_index = self.adder.lock().unwrap().get_index_from_tag(output_tag.as_str());
            let enable_input_index = self.enable_gates[6].lock().unwrap().get_index_from_tag(input_tag.as_str());
            connect_gates(
                self.adder.clone(),
                adder_output_index,
                self.enable_gates[6].clone(),
                enable_input_index,
            );

            let z_input_index = self.z.lock().unwrap().get_index_from_tag(input_tag.as_str());

            let alu_output_index = self.get_index_from_tag(output_tag.as_str());
            for j in 0..7 {
                //Enable gates -> Enable splitters
                let enable_output_index = self.enable_gates[j].lock().unwrap().get_index_from_tag(output_tag.as_str());
                connect_gates(
                    self.enable_gates[j].clone(),
                    enable_output_index,
                    self.enable_splitters[j].clone(),
                    i,
                );

                //Enable splitters -> Z
                let splitter_output_index = self.enable_splitters[j].lock().unwrap().get_index_for_output(
                    i, 0,
                );
                connect_gates(
                    self.enable_splitters[j].clone(),
                    splitter_output_index,
                    self.z.clone(),
                    z_input_index,
                );

                //Enable splitters -> Output
                let splitter_output_index = self.enable_splitters[j].lock().unwrap().get_index_for_output(
                    i, 1,
                );
                connect_gates(
                    self.enable_splitters[j].clone(),
                    splitter_output_index,
                    output_gates[alu_output_index].clone(),
                    0,
                );
            }
        }


        //Carry In -> Signal gatekeepers & Signal gatekeepers -> Arithmetic gates
        connect_gates(
            c_in_input_gate.clone(),
            0,
            self.carry_in_signal_gatekeepers[0].clone(),
            0,
        );


        let shift_left_shift_in_index = self.shift_left.lock().unwrap().get_index_from_tag("S_IN");
        connect_gates(
            self.carry_in_signal_gatekeepers[0].clone(),
            0,
            self.shift_left.clone(),
            shift_left_shift_in_index,
        );


        connect_gates(
            c_in_input_gate.clone(),
            0,
            self.carry_in_signal_gatekeepers[1].clone(),
            0,
        );


        let shift_right_shift_in_index = self.shift_right.lock().unwrap().get_index_from_tag("S_IN");
        connect_gates(
            self.carry_in_signal_gatekeepers[1].clone(),
            0,
            self.shift_right.clone(),
            shift_right_shift_in_index,
        );


        connect_gates(
            c_in_input_gate.clone(),
            0,
            self.carry_in_signal_gatekeepers[2].clone(),
            0,
        );


        let adder_carry_in_index = self.adder.lock().unwrap().get_index_from_tag("C_IN");
        connect_gates(
            self.carry_in_signal_gatekeepers[2].clone(),
            0,
            self.adder.clone(),
            adder_carry_in_index,
        );


        //Simple outputs
        let xor_larger_output_index = self.xor_le.lock().unwrap().get_index_from_tag("L");
        let a_larger_output_index = self.get_index_from_tag("A_L");
        connect_gates(
            self.xor_le.clone(),
            xor_larger_output_index,
            output_gates[a_larger_output_index].clone(),
            0,
        );


        let xor_equal_output_index = self.xor_le.lock().unwrap().get_index_from_tag("E");
        let equal_output_index = self.get_index_from_tag("EQ");
        connect_gates(
            self.xor_le.clone(),
            xor_equal_output_index,
            output_gates[equal_output_index].clone(),
            0,
        );


        let z_output_index = self.z.lock().unwrap().get_index_from_tag("O");
        let alu_z_output_index = self.get_index_from_tag("Z");
        connect_gates(
            self.z.clone(),
            z_output_index,
            output_gates[alu_z_output_index].clone(),
            0,
        );


        //Shift/Carry out -> Controlled Buffers
        let shl_shift_out_index = self.shift_left.lock().unwrap().get_index_from_tag("S_OUT");
        connect_gates(
            self.shift_left.clone(),
            shl_shift_out_index,
            self.shl_controlled_buffer.clone(),
            0,
        );


        let shl_shift_out_index = self.shift_right.lock().unwrap().get_index_from_tag("S_OUT");
        connect_gates(
            self.shift_right.clone(),
            shl_shift_out_index,
            self.shr_controlled_buffer.clone(),
            0,
        );


        let adder_carry_out_index = self.adder.lock().unwrap().get_index_from_tag("C_OUT");
        connect_gates(
            self.adder.clone(),
            adder_carry_out_index,
            self.adder_controlled_buffer.clone(),
            0,
        );


        //Controlled Buffers -> Carry out output
        let alu_carry_output_index = self.get_index_from_tag("C_OUT");
        connect_gates(
            self.shl_controlled_buffer.clone(),
            0,
            output_gates[alu_carry_output_index].clone(),
            0,
        );


        connect_gates(
            self.shr_controlled_buffer.clone(),
            0,
            output_gates[alu_carry_output_index].clone(),
            0,
        );


        connect_gates(
            self.adder_controlled_buffer.clone(),
            0,
            output_gates[alu_carry_output_index].clone(),
            0,
        );


        //Decoder Inputs and set starting states to HIGH (HIGH, HIGH, HIGH will set all enables to
        // disabled).
        connect_gates(
            a_input_gate.clone(),
            0,
            self.decoder.clone(),
            2,
        );


        a_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );


        connect_gates(
            b_input_gate.clone(),
            0,
            self.decoder.clone(),
            1,
        );


        b_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );


        connect_gates(
            c_input_gate.clone(),
            0,
            self.decoder.clone(),
            0,
        );


        c_input_gate.lock().unwrap().update_input_signal(
            GateInput::new(
                0,
                HIGH,
                UniqueID::zero_id(),
            )
        );

        //Decoder & splitter values
        // Xor;   Decoder input: 6; Enable Gate idx: 0;  ---
        // Or;    Decoder input: 5; Enable Gate idx: 1; Splitter idx: 0; input_gatekeeper idx: 0 & 6; carry_gatekeeper idx: --;
        // And;   Decoder input: 4; Enable Gate idx: 2; Splitter idx: 1; input_gatekeeper idx: 1 & 7; carry_gatekeeper idx: --;
        // Not;   Decoder input: 3; Enable Gate idx: 3; Splitter idx: 2; input_gatekeeper idx: 2    ; carry_gatekeeper idx: --;
        // SHL;   Decoder input: 2; Enable Gate idx: 4; Splitter idx: 3; input_gatekeeper idx: 3    ; carry_gatekeeper idx:  0;
        // SHR;   Decoder input: 1; Enable Gate idx: 5; Splitter idx: 4; input_gatekeeper idx: 4    ; carry_gatekeeper idx:  1;
        // Adder; Decoder input: 0; Enable Gate idx: 6; Splitter idx: 5; input_gatekeeper idx: 5 & 8; carry_gatekeeper idx:  2;


        //Decoder -> Splitters
        for j in 0..6 {
            connect_gates(
                self.decoder.clone(),
                5 - j,
                self.decoder_splitters[j].clone(),
                0,
            );
        }

        //Splitters -> Enables


        //Xor does not need a splitter
        let enable_gate_enable_index = self.enable_gates[0].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder.clone(),
            6,
            self.enable_gates[0].clone(),
            enable_gate_enable_index,
        );


        //Or
        let decoder_splitter_output_index = self.decoder_splitters[0].lock().unwrap().get_index_for_output(
            0, 0,
        );
        let enable_gate_enable_index = self.enable_gates[1].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[0].clone(),
            decoder_splitter_output_index,
            self.enable_gates[1].clone(),
            enable_gate_enable_index,
        );


        let decoder_splitter_output_index = self.decoder_splitters[0].lock().unwrap().get_index_for_output(
            0, 1,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[0].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[0].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[0].clone(),
            gatekeeper_enable_index,
        );


        let decoder_splitter_output_index = self.decoder_splitters[0].lock().unwrap().get_index_for_output(
            0, 2,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[6].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[0].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[6].clone(),
            gatekeeper_enable_index,
        );


        //And
        let decoder_splitter_output_index = self.decoder_splitters[1].lock().unwrap().get_index_for_output(
            0, 0,
        );
        let enable_gate_enable_index = self.enable_gates[2].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[1].clone(),
            decoder_splitter_output_index,
            self.enable_gates[2].clone(),
            enable_gate_enable_index,
        );


        let decoder_splitter_output_index = self.decoder_splitters[1].lock().unwrap().get_index_for_output(
            0, 1,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[1].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[1].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[1].clone(),
            gatekeeper_enable_index,
        );


        let decoder_splitter_output_index = self.decoder_splitters[1].lock().unwrap().get_index_for_output(
            0, 2,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[7].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[1].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[7].clone(),
            gatekeeper_enable_index,
        );


        //Not
        let decoder_splitter_output_index = self.decoder_splitters[2].lock().unwrap().get_index_for_output(
            0, 0,
        );
        let enable_gate_enable_index = self.enable_gates[3].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[2].clone(),
            decoder_splitter_output_index,
            self.enable_gates[3].clone(),
            enable_gate_enable_index,
        );


        let decoder_splitter_output_index = self.decoder_splitters[2].lock().unwrap().get_index_for_output(
            0, 1,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[2].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[2].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[2].clone(),
            gatekeeper_enable_index,
        );

        //Shift Left
        let decoder_splitter_output_index = self.decoder_splitters[3].lock().unwrap().get_index_for_output(
            0, 0,
        );
        let enable_gate_enable_index = self.enable_gates[4].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[3].clone(),
            decoder_splitter_output_index,
            self.enable_gates[4].clone(),
            enable_gate_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[3].lock().unwrap().get_index_for_output(
            0, 1,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[3].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[3].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[3].clone(),
            gatekeeper_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[3].lock().unwrap().get_index_for_output(
            0, 2,
        );
        let gatekeeper_enable_index = self.carry_in_signal_gatekeepers[0].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[3].clone(),
            decoder_splitter_output_index,
            self.carry_in_signal_gatekeepers[0].clone(),
            gatekeeper_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[3].lock().unwrap().get_index_for_output(
            0, 3,
        );
        let controlled_buffer_enable_index = self.shl_controlled_buffer.lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[3].clone(),
            decoder_splitter_output_index,
            self.shl_controlled_buffer.clone(),
            controlled_buffer_enable_index,
        );

        //Shift right
        let decoder_splitter_output_index = self.decoder_splitters[4].lock().unwrap().get_index_for_output(
            0, 0,
        );
        let enable_gate_enable_index = self.enable_gates[5].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[4].clone(),
            decoder_splitter_output_index,
            self.enable_gates[5].clone(),
            enable_gate_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[4].lock().unwrap().get_index_for_output(
            0, 1,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[4].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[4].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[4].clone(),
            gatekeeper_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[4].lock().unwrap().get_index_for_output(
            0, 2,
        );
        let gatekeeper_enable_index = self.carry_in_signal_gatekeepers[1].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[4].clone(),
            decoder_splitter_output_index,
            self.carry_in_signal_gatekeepers[1].clone(),
            gatekeeper_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[4].lock().unwrap().get_index_for_output(
            0, 3,
        );
        let controlled_buffer_enable_index = self.shr_controlled_buffer.lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[4].clone(),
            decoder_splitter_output_index,
            self.shr_controlled_buffer.clone(),
            controlled_buffer_enable_index,
        );

        //Adder
        let decoder_splitter_output_index = self.decoder_splitters[5].lock().unwrap().get_index_for_output(
            0, 0,
        );
        let enable_gate_enable_index = self.enable_gates[6].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[5].clone(),
            decoder_splitter_output_index,
            self.enable_gates[6].clone(),
            enable_gate_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[5].lock().unwrap().get_index_for_output(
            0, 1,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[5].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[5].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[5].clone(),
            gatekeeper_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[5].lock().unwrap().get_index_for_output(
            0, 2,
        );
        let gatekeeper_enable_index = self.input_signal_gatekeepers[8].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[5].clone(),
            decoder_splitter_output_index,
            self.input_signal_gatekeepers[8].clone(),
            gatekeeper_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[5].lock().unwrap().get_index_for_output(
            0, 3,
        );
        let gatekeeper_enable_index = self.carry_in_signal_gatekeepers[2].lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[5].clone(),
            decoder_splitter_output_index,
            self.carry_in_signal_gatekeepers[2].clone(),
            gatekeeper_enable_index,
        );

        let decoder_splitter_output_index = self.decoder_splitters[5].lock().unwrap().get_index_for_output(
            0, 4,
        );
        let controlled_buffer_enable_index = self.adder_controlled_buffer.lock().unwrap().get_index_from_tag("E");
        connect_gates(
            self.decoder_splitters[5].clone(),
            decoder_splitter_output_index,
            self.adder_controlled_buffer.clone(),
            controlled_buffer_enable_index,
        );

        //Prime gates
        self.complex_gate.calculate_output_from_inputs_and_set_child_count(
            true,
        );
    }
}

impl LogicGate for ArithmeticLogicUnit {
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
        let start = Instant::now();

        let result = self.complex_gate.fetch_output_signals(
            &self.get_tag(),
        );

        unsafe {
            ALU_TIME += start.elapsed();
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use rand::Rng;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW_, NONE};
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
            (LOW_, 0)
        } else {
            (HIGH, 1)
        }
    }

    fn convert_num_to_signal(j: usize) -> Signal {
        if j == 0 {
            LOW_
        } else {
            HIGH
        }
    }

    fn convert_bool_to_signal(b: bool) -> Signal {
        if !b {
            LOW_
        } else {
            HIGH
        }
    }

    #[derive(Debug)]
    struct GenerateRandomReturns {
        a_input: usize,
        b_input: usize,
        result_num: usize,
        a_input_signals: Vec<Signal>,
        b_input_signals: Vec<Signal>,
        output: Vec<Signal>,
        carry_out: Signal,
    }

    impl GenerateRandomReturns {
        fn new(
            a_input: usize,
            b_input: usize,
            result_num: usize,
            a_input_signals: Vec<Signal>,
            b_input_signals: Vec<Signal>,
            output: Vec<Signal>,
            carry_out: Signal,
        ) -> Self {
            GenerateRandomReturns {
                a_input,
                b_input,
                result_num,
                a_input_signals,
                b_input_signals,
                output,
                carry_out,
            }
        }
    }

    fn run_alu(
        num_bits: usize,
        opt: AluOperations,
        mut gen_randoms_result: GenerateRandomReturns,
    ) {
        println!("result_num: {:#?}", gen_randoms_result);
        gen_randoms_result.output.push(
            convert_bool_to_signal(gen_randoms_result.a_input > gen_randoms_result.b_input)
        );  //A Larger  (A_L)
        gen_randoms_result.output.push(
            convert_bool_to_signal(gen_randoms_result.a_input == gen_randoms_result.b_input)
        ); //Equal     (EQ)
        gen_randoms_result.output.push(
            convert_bool_to_signal(gen_randoms_result.result_num == 0)
        ); //Zero      (Z)
        gen_randoms_result.output.push(gen_randoms_result.carry_out); //Carry Out (C_OUT)


        let alu = ArithmeticLogicUnit::new(num_bits);


        let alu_operation = AluOperations::get_vectors(opt);


        run_multi_input_output_logic_gate(
            vec![],
            vec![
                gen_randoms_result.output //A_L, EQ, Z, C_OUT;
            ],
            HashMap::from(
                [
                    ("a", vec![gen_randoms_result.a_input_signals]),
                    ("b", vec![gen_randoms_result.b_input_signals]),
                    ("A", vec![alu_operation.a]),
                    ("B", vec![alu_operation.b]),
                    ("C", vec![alu_operation.c]),
                    ("C_IN", vec![vec![LOW_]]),
                ]
            ),
            alu,
        );
    }

    fn generate_random_xor_le_inputs_outputs(
        num_bits: usize
    ) -> GenerateRandomReturns {
        let high_number_range = usize::pow(2, num_bits as u32);
        let a_input = rand::thread_rng().gen_range(0..high_number_range);
        let b_input = rand::thread_rng().gen_range(0..high_number_range);
        let result = a_input ^ b_input;

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
        let output = convert_binary_to_vec(&result_binary);

        GenerateRandomReturns::new(
            a_input,
            b_input,
            result,
            a_input_signals,
            b_input_signals,
            output,
            NONE,
        )
    }

    fn generate_random_and_or_inputs_outputs(
        and_gate: bool,
        num_bits: usize,
    ) -> GenerateRandomReturns {
        let high_number_range = usize::pow(2, num_bits as u32);
        let a_input = rand::thread_rng().gen_range(0..high_number_range);
        let b_input = rand::thread_rng().gen_range(0..high_number_range);

        let result =
            if and_gate {
                a_input & b_input
            } else {
                a_input | b_input
            };

        let a_binary = format!("{:0width$b}", a_input, width = num_bits);
        let b_binary = format!("{:0width$b}", b_input, width = num_bits);
        let result_binary = format!("{:0width$b}", result, width = num_bits);

        //Leave these here in case it fails the number will be reproducible.
        println!("num_bits: {}", num_bits);
        println!("{}", a_input);
        println!("{}", b_input);
        println!("{}", result_binary);

        let a_input_signals = convert_binary_to_vec(&a_binary);
        let b_input_signals = convert_binary_to_vec(&b_binary);
        let output = convert_binary_to_vec(&result_binary);

        GenerateRandomReturns::new(
            a_input,
            b_input,
            result,
            a_input_signals,
            b_input_signals,
            output,
            NONE,
        )
    }

    fn generate_random_not_inputs_outputs(num_bits: usize) -> GenerateRandomReturns {
        let high_number_range = usize::pow(2, num_bits as u32);
        let num: u32 = rand::thread_rng().gen_range(0..high_number_range) as u32;

        let first_binary = format!("{:0width$b}", num, width = num_bits);

        //This method is used because using the `!` operator will result in flipping all 32 bits of
        // the number, not just the relevant bits. This will leave a majority of the bits in the `1`
        // position.
        let mut result_binary = String::new();
        for c in first_binary.chars() {
            result_binary.push(
                if c == '0' {
                    '1'
                } else {
                    '0'
                }
            )
        }
        let result: u32 = u32::from_str_radix(result_binary.as_str(), 2).unwrap();

        //Leave these here in case it fails the number will be reproducible.
        println!("num_bits: {}", num_bits);
        println!("{}", first_binary);
        println!("{}", result_binary);

        let input = convert_binary_to_vec(&first_binary);
        let output = convert_binary_to_vec(&result_binary);

        GenerateRandomReturns::new(
            num as usize,
            0,
            result as usize,
            input,
            vec![LOW_; num_bits],
            output,
            NONE,
        )
    }

    fn generate_randoms_shl_shr_inputs_outputs(
        num_bits: usize,
        left_shift: bool,
    ) -> GenerateRandomReturns {
        let high_number_range = usize::pow(2, num_bits as u32);
        let first_num = rand::thread_rng().gen_range(0..high_number_range);

        let result =
            if left_shift {
                first_num << 1
            } else {
                first_num >> 1
            };

        let first_binary = format!("{:0width$b}", first_num, width = num_bits);
        let mut result_binary = format!("{:0width$b}", result, width = num_bits);

        if result_binary.len() > num_bits {
            result_binary = result_binary[result_binary.len() - num_bits..].to_string();
        }
        let result = usize::from_str_radix(result_binary.as_str(), 2).unwrap();

        //Leave these here in case it fails the number will be reproducible.
        println!("num_bits: {}", num_bits);
        println!("left_shift: {}", left_shift);
        println!("{}", first_binary);
        println!("{}", result_binary);

        let a_input = convert_binary_to_vec(&first_binary);
        let result_output = convert_binary_to_vec(&result_binary);

        let shift_char =
            if left_shift {
                first_binary.chars().next().unwrap()
            } else {
                first_binary.chars().last().unwrap()
            };

        let (shift_out, _) = convert_char_to_signal_and_num(shift_char);

        GenerateRandomReturns::new(
            first_num,
            0,
            result,
            a_input,
            vec![LOW_; num_bits],
            result_output,
            shift_out,
        )
    }

    fn generate_randoms_adder_inputs_outputs(num_bits: usize) -> GenerateRandomReturns {
        let high_number_range = usize::pow(2, num_bits as u32);
        let first_num = rand::thread_rng().gen_range(0..high_number_range);
        let second_num = rand::thread_rng().gen_range(0..high_number_range);
        let sum = first_num + second_num;

        let first_binary = format!("{:0width$b}", first_num, width = num_bits);
        let second_binary = format!("{:0width$b}", second_num, width = num_bits);
        let mut sum_binary = format!("{:0width$b}", sum, width = num_bits + 1);

        //Leave these here in case it fails the number will be reproducible.
        println!("num_bits: {}", num_bits);
        println!("{}", first_binary);
        println!("{}", second_binary);
        println!("{}", sum_binary);

        let a_input = convert_binary_to_vec(&first_binary);
        let b_input = convert_binary_to_vec(&second_binary);
        let mut output = convert_binary_to_vec(&sum_binary);

        let carry_out = output.last().unwrap().clone();
        output.pop();

        while sum_binary.len() > first_binary.len() {
            sum_binary.remove(0);
        }
        let sum = usize::from_str_radix(sum_binary.as_str(), 2).unwrap();

        GenerateRandomReturns::new(
            first_num,
            second_num,
            sum,
            a_input,
            b_input,
            output,
            carry_out,
        )
    }

    #[test]
    fn half_adder_low_low() {
        test_half_adder(
            LOW_,
            LOW_,
            LOW_,
            LOW_,
        );
    }

    #[test]
    fn half_adder_low_high() {
        test_half_adder(
            LOW_,
            HIGH,
            HIGH,
            LOW_,
        );
    }

    #[test]
    fn half_adder_high_low() {
        test_half_adder(
            HIGH,
            LOW_,
            HIGH,
            LOW_,
        );
    }

    #[test]
    fn half_adder_high_high() {
        test_half_adder(
            HIGH,
            HIGH,
            LOW_,
            HIGH,
        );
    }

    #[test]
    fn full_adder_low_low_low() {
        test_full_adder(
            LOW_,
            LOW_,
            LOW_,
            LOW_,
            LOW_,
        );
    }

    #[test]
    fn full_adder_low_low_high() {
        test_full_adder(
            LOW_,
            LOW_,
            HIGH,
            HIGH,
            LOW_,
        );
    }

    #[test]
    fn full_adder_low_high_low() {
        test_full_adder(
            LOW_,
            HIGH,
            LOW_,
            HIGH,
            LOW_,
        );
    }

    #[test]
    fn full_adder_low_high_high() {
        test_full_adder(
            LOW_,
            HIGH,
            HIGH,
            LOW_,
            HIGH,
        );
    }

    #[test]
    fn full_adder_high_low_low() {
        test_full_adder(
            HIGH,
            LOW_,
            LOW_,
            HIGH,
            LOW_,
        );
    }

    #[test]
    fn full_adder_high_low_high() {
        test_full_adder(
            HIGH,
            LOW_,
            HIGH,
            LOW_,
            HIGH,
        );
    }

    #[test]
    fn full_adder_high_high_low() {
        test_full_adder(
            HIGH,
            HIGH,
            LOW_,
            LOW_,
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
        for c in binary.chars().rev() {
            if c == '0' {
                vec.push(LOW_);
            } else {
                vec.push(HIGH);
            }
        }
        vec
    }

    #[test]
    fn variable_bit_adder_tests() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let mut gen_randoms_result = generate_randoms_adder_inputs_outputs(num_bits);
            let variable_bit_adder = VariableBitAdder::new(num_bits);

            gen_randoms_result.output.push(gen_randoms_result.carry_out);

            run_multi_input_output_logic_gate(
                vec![],
                vec![
                    gen_randoms_result.output
                ],
                HashMap::from(
                    [
                        ("a", vec![gen_randoms_result.a_input_signals]),
                        ("b", vec![gen_randoms_result.b_input_signals]),
                        ("C_IN", vec![vec![LOW_]]),
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

            let left_shift = rand::thread_rng().gen_bool(0.5);

            let variable_bit_left_shift: SharedMutex<dyn LogicGate> =
                if left_shift {
                    VariableBitShiftLeft::<true>::new(num_bits)
                } else {
                    VariableBitShiftLeft::<false>::new(num_bits)
                };

            let mut gen_randoms_result = generate_randoms_shl_shr_inputs_outputs(
                num_bits,
                left_shift,
            );

            gen_randoms_result.output.push(gen_randoms_result.carry_out);

            run_multi_input_output_logic_gate(
                vec![
                    gen_randoms_result.a_input_signals
                ],
                vec![
                    gen_randoms_result.output
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

            let gen_randoms_result = generate_random_not_inputs_outputs(num_bits);
            let variable_bit_not = VariableBitNot::new(num_bits);

            run_multi_input_output_logic_gate(
                vec![
                    gen_randoms_result.a_input_signals
                ],
                vec![
                    gen_randoms_result.output
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

            let and_gate = rand::thread_rng().gen_bool(0.5);
            let gen_randoms_result = generate_random_and_or_inputs_outputs(
                and_gate,
                num_bits,
            );

            let variable_bit_not: SharedMutex<dyn LogicGate> =
                if and_gate {
                    VariableBitAnd::new(num_bits)
                } else {
                    VariableBitOr::new(num_bits)
                };

            run_multi_input_output_logic_gate(
                vec![],
                vec![
                    gen_randoms_result.output
                ],
                HashMap::from(
                    [
                        ("a", vec![gen_randoms_result.a_input_signals]),
                        ("b", vec![gen_randoms_result.b_input_signals]),
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
            let mut xor_le_returns = generate_random_xor_le_inputs_outputs(num_bits);

            xor_le_returns.output.push(
                convert_bool_to_signal(xor_le_returns.a_input > xor_le_returns.b_input),
            );

            xor_le_returns.output.push(
                convert_bool_to_signal(xor_le_returns.a_input == xor_le_returns.b_input),
            );

            let variable_bit_xor_le = VariableBitXOrLE::new(num_bits);
            run_multi_input_output_logic_gate(
                vec![],
                vec![
                    xor_le_returns.output
                ],
                HashMap::from(
                    [
                        ("a", vec![xor_le_returns.a_input_signals]),
                        ("b", vec![xor_le_returns.b_input_signals]),
                    ]
                ),
                variable_bit_xor_le,
            );
        }
    }

    #[test]
    fn variable_z_gate_tests() {
        let num_bits = 4;
        for i in 0..=num_bits {
            let mut input_vector = vec![LOW_; num_bits];
            let mut output_vector = vec![HIGH];

            if i != num_bits {
                input_vector[i] = HIGH;
                output_vector[0] = LOW_;
            }

            let variable_z = VariableBitZ::new(num_bits);

            run_multi_input_output_logic_gate(
                vec![
                    input_vector
                ],
                vec![
                    output_vector
                ],
                HashMap::from([]),
                variable_z,
            );
        }
    }

    #[test]
    fn variable_bit_enable_tests() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let mut rng = rand::thread_rng();
            let signals = [NONE, LOW_, HIGH];

            let input: Vec<Signal> = (0..num_bits)
                .map(|_| {
                    let idx = rng.gen_range(0..signals.len());
                    signals[idx].clone()
                })
                .collect();

            let idx = rng.gen_range(1..3);
            let enable = vec![signals[idx].clone()];

            let output =
                if *enable.first().unwrap() == LOW_ {
                    vec![NONE; num_bits]
                } else {
                    input.clone()
                };

            //Leave these here in case test breaks.
            println!("num_bits {}", num_bits);
            println!("input {:?}", input);
            println!("output {:?}", output);
            println!("enable {:?}", enable);

            let variable_z = VariableBitEnable::new(num_bits);

            run_multi_input_output_logic_gate(
                vec![
                    input
                ],
                vec![
                    output
                ],
                HashMap::from(
                    [("E", vec![enable])]
                ),
                variable_z,
            );
        }
    }

    #[test]
    fn arithmetic_logic_unit_off_test() {
        let num_bits = rand::thread_rng().gen_range(2..16);

        let a_input_signals = vec![HIGH; num_bits];
        let b_input_signals = vec![HIGH; num_bits];
        let mut output = vec![NONE; num_bits];

        output.push(LOW_);  //A Larger  (A_L)
        output.push(HIGH); //Equal     (EQ)
        output.push(HIGH); //Zero      (Z)
        output.push(NONE); //Carry Out (C_OUT)

        let alu = ArithmeticLogicUnit::new(num_bits);
        let alu_operation = AluOperations::get_vectors(AluOperations::None);

        run_multi_input_output_logic_gate(
            vec![],
            vec![
                output //A_L, EQ, Z, C_OUT;
            ],
            HashMap::from(
                [
                    ("a", vec![a_input_signals]),
                    ("b", vec![b_input_signals]),
                    ("A", vec![alu_operation.a]),
                    ("B", vec![alu_operation.b]),
                    ("C", vec![alu_operation.c]),
                    ("C_IN", vec![vec![HIGH]]),
                ]
            ),
            alu,
        );
    }

    #[test]
    fn arithmetic_logic_unit_zero_test() {
        let num_bits = rand::thread_rng().gen_range(2..16);

        let a_input_signals = vec![LOW_; num_bits];
        let b_input_signals = vec![LOW_; num_bits];
        let mut output = vec![LOW_; num_bits];

        output.push(LOW_); //A Larger  (A_L)
        output.push(HIGH); //Equal     (EQ)
        output.push(HIGH); //Zero      (Z)
        output.push(NONE); //Carry Out (C_OUT)

        let alu = ArithmeticLogicUnit::new(num_bits);

        let alu_operation = AluOperations::get_vectors(AluOperations::Or);

        run_multi_input_output_logic_gate(
            vec![],
            vec![
                output //A_L, EQ, Z, C_OUT;
            ],
            HashMap::from(
                [
                    ("a", vec![a_input_signals]),
                    ("b", vec![b_input_signals]),
                    ("A", vec![alu_operation.a]),
                    ("B", vec![alu_operation.b]),
                    ("C", vec![alu_operation.c]),
                    ("C_IN", vec![vec![LOW_]]),
                ]
            ),
            alu,
        );
    }

    #[test]
    fn arithmetic_logic_unit_xor_test() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let xor_le_returns = generate_random_xor_le_inputs_outputs(num_bits);

            run_alu(num_bits, AluOperations::XOrLe, xor_le_returns);
        }
    }

    #[test]
    fn arithmetic_logic_unit_and_or_test() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let and_gate = rand::thread_rng().gen_bool(0.5);
            let gen_randoms_result = generate_random_and_or_inputs_outputs(
                and_gate,
                num_bits,
            );

            let opt =
                if and_gate {
                    AluOperations::And
                } else {
                    AluOperations::Or
                };

            run_alu(num_bits, opt, gen_randoms_result);
        }
    }

    #[test]
    fn arithmetic_logic_unit_not_test() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let gen_randoms_result = generate_random_not_inputs_outputs(num_bits);


            run_alu(num_bits, AluOperations::Not, gen_randoms_result);
        }
    }

    #[test]
    fn arithmetic_logic_unit_shift_left_right_test() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);
            let left_shift = rand::thread_rng().gen_bool(0.5);

            let gen_randoms_result = generate_randoms_shl_shr_inputs_outputs(
                num_bits,
                left_shift,
            );

            let opt =
                if left_shift {
                    AluOperations::Shl
                } else {
                    AluOperations::Shr
                };

            run_alu(num_bits, opt, gen_randoms_result);
        }
    }

    #[test]
    fn arithmetic_logic_unit_adder_test() {
        for _ in 0..20 {
            let num_bits = rand::thread_rng().gen_range(2..16);

            let gen_randoms_result = generate_randoms_adder_inputs_outputs(num_bits);

            run_alu(num_bits, AluOperations::Adder, gen_randoms_result);
        }
    }
}

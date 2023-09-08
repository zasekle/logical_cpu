use std::cell::RefCell;
use std::rc::Rc;

use crate::logic::foundations::{
    GateInput,
    GateOutputState,
    LogicGate,
    UniqueID,
    GateLogicError,
    GateType,
    GateLogic,
    BasicGateMembers,
    InputSignalReturn
};

pub struct Or {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Or {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Or {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::Or,
                    )
                }
            )
        )
    }
}

impl LogicGate for Or {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct And {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl And {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                And {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::And,
                    )
                }
            )
        )
    }
}

impl LogicGate for And {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct Not {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Not {
    pub fn new(output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Not {
                    members: BasicGateMembers::new(
                        1,
                        output_num,
                        GateType::Not,
                    )
                }
            )
        )
    }
}

impl LogicGate for Not {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct Nor {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Nor {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Nor {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::Nor,
                    )
                }
            )
        )
    }
}

impl LogicGate for Nor {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

pub struct Nand {
    members: BasicGateMembers,
}

#[allow(dead_code)]
impl Nand {
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Nand {
                    members: BasicGateMembers::new(
                        input_num,
                        output_num,
                        GateType::Nand,
                    )
                }
            )
        )
    }
}

impl LogicGate for Nand {
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.members.connect_output_to_next_gate(
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        self.members.update_input_signal(input)
    }

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError> {
        GateLogic::fetch_output_signals_basic_gate(&mut self.members)
    }

    fn get_gate_type(&self) -> GateType {
        self.members.gate_type
    }

    fn get_unique_id(&self) -> UniqueID {
        self.members.unique_id
    }

    fn toggle_output_printing(&mut self, print_output: bool) {
        self.members.should_print_output = print_output;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::logic::foundations::Signal;
    use crate::logic::foundations::Signal::{HIGH, LOW};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::check_for_single_element_signal;
    use super::*;

    fn test_simple_gate(
        gate: Rc<RefCell<dyn LogicGate>>,
        first_input: Signal,
        second_input: Option<Signal>,
        output: Signal,
    ) {
        let first_pin_input = AutomaticInput::new(vec![first_input], 1, "");
        let output_gate = SimpleOutput::new("");

        let mut input_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGate>>> = HashMap::new();
        let mut output_gates: HashMap<UniqueID, Rc<RefCell<dyn LogicGateAndOutputGate>>> = HashMap::new();

        input_gates.insert(first_pin_input.borrow_mut().get_unique_id(), first_pin_input.clone());
        output_gates.insert(output_gate.borrow_mut().get_unique_id(), output_gate.clone());

        first_pin_input.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            gate.clone(),
        );

        if let Some(second_input) = second_input {
            let second_pin_input = AutomaticInput::new(vec![second_input], 1, "");

            second_pin_input.borrow_mut().connect_output_to_next_gate(
                0,
                1,
                gate.clone(),
            );

            input_gates.insert(second_pin_input.borrow_mut().get_unique_id(), second_pin_input.clone());
        }

        gate.borrow_mut().connect_output_to_next_gate(
            0,
            0,
            output_gate.clone(),
        );

        run_circuit(
            input_gates,
            output_gates,
            |_clock_tick_inputs, output_gates| {
                check_for_single_element_signal(output_gates, output.clone());
            },
        );
    }

    #[test]
    fn test_or_gate_low_low() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            LOW,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_or_gate_low_high() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            LOW,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_or_gate_high_low() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            HIGH,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_or_gate_high_high() {
        let or_gate = Or::new(2, 1);

        test_simple_gate(
            or_gate,
            HIGH,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_and_gate_low_low() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            LOW,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_and_gate_low_high() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            LOW,
            Some(HIGH),
            LOW,
        );
    }

    #[test]
    fn test_and_gate_high_low() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            HIGH,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_and_gate_high_high() {
        let and_gate = And::new(2, 1);

        test_simple_gate(
            and_gate,
            HIGH,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_not_gate_low() {
        let not_gate = Not::new(1);

        test_simple_gate(
            not_gate,
            LOW,
            None,
            HIGH,
        );
    }

    #[test]
    fn test_not_gate_high() {
        let not_gate = Not::new(1);

        test_simple_gate(
            not_gate,
            HIGH,
            None,
            LOW,
        );
    }

    #[test]
    fn test_nor_gate_low_low() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            LOW,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_nor_gate_low_high() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            LOW,
            Some(HIGH),
            LOW,
        );
    }

    #[test]
    fn test_nor_gate_high_low() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            HIGH,
            Some(LOW),
            LOW,
        );
    }

    #[test]
    fn test_nor_gate_high_high() {
        let nor_gate = Nor::new(2, 1);

        test_simple_gate(
            nor_gate,
            HIGH,
            Some(HIGH),
            LOW,
        );
    }

    #[test]
    fn test_nand_gate_low_low() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            LOW,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_low_high() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            HIGH,
            Some(LOW),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_high_low() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            LOW,
            Some(HIGH),
            HIGH,
        );
    }

    #[test]
    fn test_nand_gate_high_high() {
        let nand_gate = Nand::new(2, 1);

        test_simple_gate(
            nand_gate,
            HIGH,
            Some(HIGH),
            LOW,
        );
    }

}
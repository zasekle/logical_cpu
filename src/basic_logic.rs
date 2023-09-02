use std::cell::RefCell;
use std::rc::Rc;
use crate::basic_logic::Signal::{HIGH, LOW};
use crate::{get_clock_tick_number, MAX_NUMBER_TIMES_INPUTS_CHANGE};

#[derive(PartialEq, Debug, Clone)]
pub enum Signal {
    LOW,
    HIGH,
}

#[derive(PartialEq, Debug, Clone)]
pub enum GateLogicError {}

#[derive(Debug, Clone)]
pub struct GateInput {
    pub input_index: usize,
    pub signal: Signal,
}

impl GateInput {
    pub fn new(input_index: usize, signal: Signal) -> Self {
        GateInput {
            input_index,
            signal,
        }
    }
}

pub trait LogicGate {
    fn connect_output(&mut self, output_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>, next_gate_input_index: usize);

    ///Returns true if something changed.
    fn change_input(&mut self, input: &GateInput) -> bool;

    fn collect_output(&mut self) -> Result<Vec<GateOutput>, GateLogicError>;

    fn get_tag(&self) -> String;
}

#[derive(Clone)]
pub enum GateOutput {
    NotConnected(Signal),
    Connected(OutputNode),
}

#[derive(Clone)]
pub struct OutputNode {
    pub input: GateInput,
    pub gate: Rc<RefCell<dyn LogicGate>>,
}

pub struct Or {
    inputs: Vec<Signal>,
    outputs: Vec<GateOutput>,
    current_clock_tick_num: usize,
    num_times_changed_current_clock_tick: usize,
}

impl Or {
    /// Default all inputs to LOW. Output will be GateOutput::NotConnected with the signal that is
    /// relevant.
    pub fn new(input_num: usize, output_num: usize) -> Rc<RefCell<Self>> {
        let clock_tick_number= get_clock_tick_number();

        let or_gate = Rc::new(
            RefCell::new(
                Or {
                    inputs: Vec::with_capacity(input_num),
                    outputs: Vec::with_capacity(output_num),
                    current_clock_tick_num: clock_tick_number,
                    num_times_changed_current_clock_tick: 0,
                }
            )
        );

        let mut mut_or_gate = or_gate.borrow_mut();

        mut_or_gate.inputs.resize_with(
            input_num,
            || LOW,
        );

        let output_signal = mut_or_gate.calculate_output_from_inputs();

        mut_or_gate.outputs.resize_with(
            output_num,
            || GateOutput::NotConnected(output_signal.clone()),
        );

        drop(mut_or_gate);

        or_gate
    }

    fn calculate_output_from_inputs(&self) -> Signal {
        let mut output_signal = LOW;
        for s in self.inputs.iter() {
            if *s == HIGH {
                output_signal = HIGH;
                break;
            }
        }

        output_signal
    }
}

impl LogicGate for Or {
    fn connect_output(&mut self, output_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>, next_gate_input_index: usize) {
        let output_signal = self.calculate_output_from_inputs();

        self.outputs[output_index] =
            GateOutput::Connected(
                OutputNode {
                    input: GateInput::new(next_gate_input_index, output_signal),
                    gate: next_gate,
                }
            );
    }

    fn change_input(&mut self, input: &GateInput) -> bool {
        let clock_tick_number= get_clock_tick_number();

        if self.current_clock_tick_num == clock_tick_number {
            self.num_times_changed_current_clock_tick += 1;

            if self.num_times_changed_current_clock_tick >= MAX_NUMBER_TIMES_INPUTS_CHANGE {
                panic!(
                    "Oscillation (a loop) was detected on the current {} gate",
                    self.get_tag(),
                );
            }
        } else {
            self.current_clock_tick_num = clock_tick_number;
            self.num_times_changed_current_clock_tick = 0;
        }

        if self.inputs[input.input_index] == input.signal {
            false
        } else {
            self.inputs[input.input_index] = input.signal.clone();
            true
        }
    }

    fn collect_output(&mut self) -> Result<Vec<GateOutput>, GateLogicError> {
        let output_signal = self.calculate_output_from_inputs();

        self.outputs.iter_mut().for_each(
            |f|
                match f {
                    GateOutput::NotConnected(gate) => {
                        *gate = output_signal.clone();
                    }
                    GateOutput::Connected(output) => {
                        output.input.signal = output_signal.clone();
                    }
                }
        );

        let output_clone = self.outputs.clone();

        Ok(output_clone)
    }

    fn get_tag(&self) -> String {
        String::from("OR")
    }
}

pub struct Clock {
    outputs: Vec<GateOutput>,
}

impl Clock {
    /// Default all inputs to LOW. Output will be GateOutput::NotConnected with the signal that is
    /// relevant.
    pub fn new(output_num: usize) -> Rc<RefCell<Self>> {
        let clock_gate = Rc::new(
            RefCell::new(
                Clock {
                    outputs: Vec::with_capacity(output_num),
                }
            )
        );

        let mut mut_clock = clock_gate.borrow_mut();

        //All clock outputs are always high
        mut_clock.outputs.resize_with(
            output_num,
            || GateOutput::NotConnected(HIGH),
        );

        drop(mut_clock);

        clock_gate
    }
}

impl LogicGate for Clock {
    fn connect_output(&mut self, output_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>, next_gate_input_index: usize) {
        self.outputs[output_index] =
            GateOutput::Connected(
                OutputNode {
                    input: GateInput::new(next_gate_input_index, HIGH),
                    gate: next_gate,
                }
            );
    }

    fn change_input(&mut self, _input: &GateInput) -> bool {
        false
    }

    fn collect_output(&mut self) -> Result<Vec<GateOutput>, GateLogicError> {
        self.outputs.iter_mut().for_each(
            |f|
                match f {
                    GateOutput::NotConnected(gate) => {
                        *gate = HIGH;
                    }
                    GateOutput::Connected(output) => {
                        output.input.signal = HIGH;
                    }
                }
        );

        let output_clone = self.outputs.clone();

        Ok(output_clone)
    }

    fn get_tag(&self) -> String {
        String::from("AND")
    }
}

pub struct Not {
    inputs: Vec<Signal>,
    outputs: Vec<GateOutput>,
    current_clock_tick_num: usize,
    num_times_changed_current_clock_tick: usize,
}

impl Not {
    /// Default all inputs to LOW. Output will be GateOutput::NotConnected with the signal that is
    /// relevant. This can only have one input.
    pub fn new(output_num: usize) -> Rc<RefCell<Self>> {
        let clock_tick_number= get_clock_tick_number();

        let not_gate = Rc::new(
            RefCell::new(
                Not {
                    inputs: Vec::with_capacity(1),
                    outputs: Vec::with_capacity(output_num),
                    current_clock_tick_num: clock_tick_number,
                    num_times_changed_current_clock_tick: 0,
                }
            )
        );

        let mut mut_not_gate = not_gate.borrow_mut();

        mut_not_gate.inputs.resize_with(
            1,
            || LOW,
        );

        let output_signal = mut_not_gate.calculate_output_from_inputs();

        mut_not_gate.outputs.resize_with(
            output_num,
            || GateOutput::NotConnected(output_signal.clone()),
        );

        drop(mut_not_gate);

        not_gate
    }

    fn calculate_output_from_inputs(&self) -> Signal {
        //This only has a single input.
        let current_signal = self.inputs.first().unwrap();

        if *current_signal == HIGH {
            LOW
        } else {
            HIGH
        }
    }
}

impl LogicGate for Not {
    fn connect_output(&mut self, output_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>, next_gate_input_index: usize) {
        let output_signal = self.calculate_output_from_inputs();

        self.outputs[output_index] =
            GateOutput::Connected(
                OutputNode {
                    input: GateInput::new(next_gate_input_index, output_signal),
                    gate: next_gate,
                }
            );
    }

    fn change_input(&mut self, input: &GateInput) -> bool {
        let clock_tick_number= get_clock_tick_number();

        if self.current_clock_tick_num == clock_tick_number {
            self.num_times_changed_current_clock_tick += 1;

            if self.num_times_changed_current_clock_tick >= MAX_NUMBER_TIMES_INPUTS_CHANGE {
                panic!(
                    "Oscillation (a loop) was detected on the current {} gate",
                    self.get_tag(),
                );
            }
        } else {
            self.current_clock_tick_num = clock_tick_number;
            self.num_times_changed_current_clock_tick = 0;
        }

        if self.inputs[input.input_index] == input.signal {
            false
        } else {
            self.inputs[input.input_index] = input.signal.clone();
            true
        }
    }

    fn collect_output(&mut self) -> Result<Vec<GateOutput>, GateLogicError> {
        let output_signal = self.calculate_output_from_inputs();

        self.outputs.iter_mut().for_each(
            |f|
                match f {
                    GateOutput::NotConnected(gate) => {
                        *gate = output_signal.clone();
                    }
                    GateOutput::Connected(output) => {
                        output.input.signal = output_signal.clone();
                    }
                }
        );

        let output_clone = self.outputs.clone();

        Ok(output_clone)
    }

    fn get_tag(&self) -> String {
        String::from("OR")
    }
}

//TODO:
// And
// Not (must test for cycles here)
// Nand
// Nor
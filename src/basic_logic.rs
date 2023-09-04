use core::fmt;
use std::cell::RefCell;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use crate::basic_logic::Signal::{HIGH, LOW};
use crate::{get_clock_tick_number, MAX_NUMBER_TIMES_INPUTS_CHANGE, UNIQUE_INDEXING_NUMBER};

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

#[derive(Debug, Copy, Clone)]
pub struct UniqueID {
    id: usize,
}

impl UniqueID {
    fn new() -> Self {
        let id = UNIQUE_INDEXING_NUMBER.fetch_add(
            1,
            Ordering::SeqCst
        );
        Self { id }
    }
}

impl PartialEq for UniqueID {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for UniqueID {}  // Eq requires that you've implemented PartialEq

impl Hash for UniqueID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
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
    fn connect_output(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>);

    ///Returns true if something changed.
    fn change_input(&mut self, input: &GateInput) -> bool;

    fn collect_output(&mut self) -> Result<Vec<GateOutput>, GateLogicError>;

    fn get_tag(&self) -> String;

    fn get_id(&self) -> UniqueID;

    fn print_output(&mut self, print_output: bool);
}

#[derive(Debug, Clone)]
pub enum GateOutput {
    NotConnected(Signal),
    Connected(OutputNode),
}

#[derive(Clone)]
pub struct OutputNode {
    pub input: GateInput,
    pub gate: Rc<RefCell<dyn LogicGate>>,
}

impl fmt::Debug for OutputNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut_gate = self.gate.borrow_mut();
        let tag = mut_gate.get_tag();
        let id = mut_gate.get_id();

        drop(mut_gate);

        f.debug_struct("OutputNode")
            .field("input", &self.input)
            .field("gate", &format!("{} gate with id {}", tag, id.id))
            .finish()
    }
}

pub struct Or {
    inputs: Vec<Signal>,
    outputs: Vec<GateOutput>,
    id: UniqueID,
    current_clock_tick_num: usize,
    num_times_changed_current_clock_tick: usize,
    print_output: bool,
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
                    id: UniqueID::new(),
                    current_clock_tick_num: clock_tick_number,
                    num_times_changed_current_clock_tick: 0,
                    print_output: false,
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
    fn connect_output(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        let output_signal = self.calculate_output_from_inputs();

        self.outputs[current_gate_output_index] =
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

        if self.print_output {
            println!(
                "{} gate id {} output is\n{:#?}",
                self.get_tag(),
                self.get_id().id,
                output_clone
            );
        }

        Ok(output_clone)
    }

    fn get_tag(&self) -> String {
        String::from("OR")
    }

    fn get_id(&self) -> UniqueID {
        self.id.clone()
    }

    fn print_output(&mut self, print_output: bool) {
        self.print_output = print_output;
    }
}

pub struct Clock {
    outputs: Vec<GateOutput>,
    id: UniqueID,
    print_output: bool,
}

impl Clock {
    /// Default all inputs to LOW. Output will be GateOutput::NotConnected with the signal that is
    /// relevant.
    pub fn new(output_num: usize) -> Rc<RefCell<Self>> {
        let clock_gate = Rc::new(
            RefCell::new(
                Clock {
                    outputs: Vec::with_capacity(output_num),
                    id: UniqueID::new(),
                    print_output: false,
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
    fn connect_output(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        self.outputs[current_gate_output_index] =
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

        if self.print_output {
            println!(
                "{} gate id {} output is\n{:#?}",
                self.get_tag(),
                self.get_id().id,
                output_clone
            );
        }

        Ok(output_clone)
    }

    fn get_tag(&self) -> String {
        String::from("AND")
    }


    fn get_id(&self) -> UniqueID {
        self.id.clone()
    }

    fn print_output(&mut self, print_output: bool) {
        self.print_output = print_output;
    }
}

pub struct Not {
    inputs: Vec<Signal>,
    outputs: Vec<GateOutput>,
    id: UniqueID,
    current_clock_tick_num: usize,
    num_times_changed_current_clock_tick: usize,
    print_output: bool,
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
                    id: UniqueID::new(),
                    current_clock_tick_num: clock_tick_number,
                    num_times_changed_current_clock_tick: 0,
                    print_output: false,
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
    fn connect_output(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        let output_signal = self.calculate_output_from_inputs();

        self.outputs[current_gate_output_index] =
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

        if self.print_output {
            println!(
                "{} gate id {} output is\n{:#?}",
                self.get_tag(),
                self.get_id().id,
                output_clone
            );
        }

        Ok(output_clone)
    }

    fn get_tag(&self) -> String {
        String::from("NOT")
    }


    fn get_id(&self) -> UniqueID {
        self.id.clone()
    }

    fn print_output(&mut self, print_output: bool) {
        self.print_output = print_output;
    }
}

//TODO:
// And
// Not (must test for cycles here)
// Nand
// Nor
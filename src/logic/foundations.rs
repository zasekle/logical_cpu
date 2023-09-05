use std::cell::RefCell;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use crate::{get_clock_tick_number, MAX_INPUT_CHANGES, NEXT_UNIQUE_ID};
use crate::logic::foundations::Signal::{HIGH, LOW};

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
    pub fn generate() -> Self {
        let id = NEXT_UNIQUE_ID.fetch_add(
            1,
            Ordering::SeqCst,
        );
        Self { id }
    }

    pub fn id(&self) -> usize {
        self.id
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
    fn connect_output_to_next_gate(&mut self, current_gate_output_index: usize, next_gate_input_index: usize, next_gate: Rc<RefCell<dyn LogicGate>>);

    ///Returns true if something changed.
    fn update_input_signal(&mut self, input: GateInput) -> bool;

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError>;

    fn get_gate_type(&self) -> GateType;

    fn get_unique_id(&self) -> UniqueID;

    fn toggle_output_printing(&mut self, print_output: bool);
}

#[derive(Debug, Clone)]
pub enum GateOutputState {
    NotConnected(Signal),
    Connected(ConnectedOutput),
}

#[derive(Clone)]
pub struct ConnectedOutput {
    pub throughput: GateInput,
    pub gate: Rc<RefCell<dyn LogicGate>>,
}

impl fmt::Debug for ConnectedOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut_gate = self.gate.borrow_mut();
        let tag = mut_gate.get_gate_type();
        let id = mut_gate.get_unique_id();

        drop(mut_gate);

        f.debug_struct("OutputNode")
            .field("input", &self.throughput)
            .field("gate", &format!("{} gate with id {}", tag, id.id))
            .finish()
    }
}

pub struct OscillationDetection {
    current_tick: usize,
    changed_count_this_tick: usize,
}

impl OscillationDetection {
    pub fn new() -> Self {
        let clock_tick_number = get_clock_tick_number();
        OscillationDetection {
            current_tick: clock_tick_number,
            changed_count_this_tick: 0,
        }
    }

    pub fn detect_oscillation(&mut self, gate_type: &GateType)
    {
        let clock_tick_number = get_clock_tick_number();

        if self.current_tick == clock_tick_number {
            self.changed_count_this_tick += 1;

            if self.changed_count_this_tick >= MAX_INPUT_CHANGES {
                panic!(
                    "Oscillation (a loop) was detected on the current {} gate",
                    gate_type,
                );
            }
        } else {
            self.current_tick = clock_tick_number;
            self.changed_count_this_tick = 0;
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GateType {
    NOT,
    OR,
    AND,
    NOR,
    NAND,
    CLOCK,
}

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            GateType::NOT => "NOT",
            GateType::OR => "OR",
            GateType::AND => "AND",
            GateType::NOR => "NOR",
            GateType::NAND => "NAND",
            GateType::CLOCK => "CLOCK",
        };
        write!(f, "{}", printable)
    }
}

pub struct BasicGateMembers {
    pub input_signals: Vec<Signal>,
    pub output_states: Vec<GateOutputState>,
    pub unique_id: UniqueID,
    pub oscillation_detection: OscillationDetection,
    pub should_print_output: bool,
    pub gate_type: GateType,
}

impl BasicGateMembers {
    pub fn new(input_num: usize, output_num: usize, gate_type: GateType) -> Self {
        let mut result = BasicGateMembers {
            input_signals: vec![LOW; input_num],
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            oscillation_detection: OscillationDetection::new(),
            should_print_output: false,
            gate_type,
        };

        let output_signal = GateLogic::calculate_output_from_inputs(
            &gate_type,
            Some(&result.input_signals),
        );

        result.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(output_signal.clone()),
        );

        result
    }

    pub fn update_input_signal(&mut self, input: GateInput) -> bool {
        self.oscillation_detection.detect_oscillation(&self.gate_type);

        if self.input_signals[input.input_index] == input.signal {
            false
        } else {
            self.input_signals[input.input_index] = input.signal.clone();
            true
        }
    }

    pub fn connect_output_to_next_gate(
        &mut self,
        current_gate_output_index: usize,
        next_gate_input_index: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        GateLogic::connect_output_to_next_gate(
            self.gate_type,
            Some(&self.input_signals),
            &mut self.output_states,
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
        );
    }
}

pub struct GateLogic;

impl GateLogic {
    pub fn calculate_output_for_or(input_signals: &Vec<Signal>) -> Signal {
        let mut output_signal = LOW;
        for s in input_signals.iter() {
            if *s == HIGH {
                output_signal = HIGH;
                break;
            }
        }

        output_signal
    }

    pub fn calculate_output_for_not(input_signals: &Vec<Signal>) -> Signal {
        //This only has a single input.
        let current_signal = input_signals.first().unwrap();

        if *current_signal == HIGH {
            LOW
        } else {
            HIGH
        }
    }

    pub fn calculate_output_for_clock() -> Signal {
        HIGH
    }

    pub fn fetch_output_signals_basic_gate(
        basic_gate: &mut BasicGateMembers
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        Self::fetch_output_signals(
            &basic_gate.gate_type,
            Some(&basic_gate.input_signals),
            &mut basic_gate.output_states,
            basic_gate.unique_id,
            basic_gate.should_print_output,
        )
    }

    pub fn fetch_output_signals(
        gate_type: &GateType,
        input_signals: Option<&Vec<Signal>>,
        output_states: &mut Vec<GateOutputState>,
        unique_id: UniqueID,
        should_print_output: bool,
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        let output_signal = GateLogic::calculate_output_from_inputs(gate_type, input_signals);

        output_states.iter_mut().for_each(
            |f|
                match f {
                    GateOutputState::NotConnected(gate) => {
                        *gate = output_signal.clone();
                    }
                    GateOutputState::Connected(output) => {
                        output.throughput.signal = output_signal.clone();
                    }
                }
        );

        let output_clone = output_states.clone();

        if should_print_output {
            println!(
                "{} gate id {} output is\n{:#?}",
                gate_type,
                unique_id.id(),
                output_clone
            );
        }

        Ok(output_clone)
    }

    pub fn connect_output_to_next_gate(
        gate_type: GateType,
        input_signals: Option<&Vec<Signal>>,
        output_states: &mut Vec<GateOutputState>,
        current_gate_output_index: usize,
        next_gate_input_index: usize,
        next_gate: Rc<RefCell<dyn LogicGate>>,
    ) {
        let output_signal = GateLogic::calculate_output_from_inputs(
            &gate_type,
            input_signals,
        );

        output_states[current_gate_output_index] =
            GateOutputState::Connected(
                ConnectedOutput {
                    throughput: GateInput::new(next_gate_input_index, output_signal),
                    gate: next_gate,
                }
            );
    }

    pub fn calculate_output_from_inputs(
        gate_type: &GateType,
        input_signals: Option<&Vec<Signal>>,
    ) -> Signal {
        match gate_type {
            GateType::NOT => GateLogic::calculate_output_for_not(input_signals.unwrap()),
            GateType::OR => GateLogic::calculate_output_for_or(input_signals.unwrap()),
            GateType::AND => panic!(),
            GateType::NOR => panic!(),
            GateType::NAND => panic!(),
            GateType::CLOCK => GateLogic::calculate_output_for_clock(),
        }
    }
}

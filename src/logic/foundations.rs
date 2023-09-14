use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use crate::globals::{get_clock_tick_number, MAX_INPUT_CHANGES, NEXT_UNIQUE_ID};
use crate::logic::foundations::Signal::{HIGH, LOW};
use crate::logic::output_gates::LogicGateAndOutputGate;
use crate::run_circuit::run_circuit;

#[derive(PartialEq, Debug, Clone)]
pub enum Signal {
    LOW,
    HIGH,
}

#[derive(PartialEq, Debug, Clone)]
pub enum GateLogicError {
    NoMoreAutomaticInputsRemaining,
}

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

pub struct InputSignalReturn {
    pub changed_count_this_tick: usize,
    pub input_signal_updated: bool,
}

pub trait LogicGate {
    fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>);

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn;

    fn fetch_output_signals(&mut self) -> Result<Vec<GateOutputState>, GateLogicError>;

    fn get_gate_type(&self) -> GateType;

    fn get_unique_id(&self) -> UniqueID;

    fn toggle_output_printing(&mut self, print_output: bool);

    ///Currently only used with input gates.
    fn get_tag(&self) -> String {
        String::new()
    }

    fn is_input_gate(&self) -> bool {
        false
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        panic!("Gate {} using tag {} id {} did not implement get_index_from_tag()", self.get_tag(), tag, self.get_unique_id().id)
    }
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

    pub fn detect_oscillation(&mut self, gate_type: &GateType) -> usize
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
            self.changed_count_this_tick = 1;
        }

        self.changed_count_this_tick
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum GateType {
    NotType,
    OrType,
    AndType,
    NorType,
    NandType,
    ControlledBufferType,
    ClockType,
    AutomaticInputType,
    SimpleOutputType,
    SimpleInputType,
    SRLatchType,
    ActiveLowSRLatchType,
    OneBitMemoryCellType,
    VariableBitMemoryCellType,
    VariableCPUEnableType,
    VariableBitRegisterType,
    VariableDecoderType,
}

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let printable = match *self {
            GateType::NotType => "NOT",
            GateType::OrType => "OR",
            GateType::AndType => "AND",
            GateType::NorType => "NOR",
            GateType::NandType => "NAND",
            GateType::ControlledBufferType => "CONTROLLED_BUFFER",
            GateType::ClockType => "CLOCK",
            GateType::AutomaticInputType => "AUTOMATIC_INPUT",
            GateType::SimpleOutputType => "SIMPLE_OUTPUT",
            GateType::SimpleInputType => "SIMPLE_INPUT",
            GateType::SRLatchType => "SR_LATCH",
            GateType::ActiveLowSRLatchType => "ACTIVE_LOW_SR_LATCH",
            GateType::OneBitMemoryCellType => "ONE_BIT_MEMORY_CELL",
            GateType::VariableBitMemoryCellType => "VARIABLE_BIT_MEMORY_CELL",
            GateType::VariableCPUEnableType => "VARIABLE_CPU_ENABLE",
            GateType::VariableBitRegisterType => "VARIABLE_BIT_REGISTER",
            GateType::VariableDecoderType => "VARIABLE_DECODER",
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
    pub fn new(input_num: usize, output_num: usize, gate_type: GateType, output_signal: Option<Signal>) -> Self {

        //Must have at least one input.
        assert_ne!(input_num, 0);

        let mut result = BasicGateMembers {
            input_signals: vec![LOW; input_num],
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            oscillation_detection: OscillationDetection::new(),
            should_print_output: false,
            gate_type,
        };

        let output_signal = if let Some(signal) = output_signal {
            signal
        } else {
            GateLogic::calculate_output_from_inputs(
                &gate_type,
                Some(&result.input_signals),
            )
        };

        result.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(output_signal.clone()),
        );

        result
    }

    pub fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        let changed_count_this_tick = self.oscillation_detection.detect_oscillation(&self.gate_type);

        let input_signal_updated = if self.input_signals[input.input_index] == input.signal {
            false
        } else {
            self.input_signals[input.input_index] = input.signal.clone();
            true
        };

        InputSignalReturn {
            changed_count_this_tick,
            input_signal_updated,
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

pub struct ComplexGateMembers {
    pub simple_gate: BasicGateMembers,
    pub input_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
    pub output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
    pub gate_tags_to_index: HashMap<String, usize>,
}

impl ComplexGateMembers {
    pub fn new(
        input_num: usize,
        output_num: usize,
        gate_type: GateType,
        input_gates: Vec<Rc<RefCell<dyn LogicGate>>>,
        output_gates: Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
    ) -> Self {

        //Must have at least one input.
        assert_ne!(input_num, 0);
        assert_ne!(output_num, 0);

        let mut gate_tags_to_index = HashMap::new();

        for (i, gate) in input_gates.iter().enumerate() {
            gate_tags_to_index.insert(
                gate.borrow_mut().get_tag(),
                i,
            );
        }

        for (i, gate) in output_gates.iter().enumerate() {
            gate_tags_to_index.insert(
                gate.borrow_mut().get_tag(),
                i,
            );
        }

        //Make sure there are enough tags for each gate.
        assert_eq!(gate_tags_to_index.len(), input_num + output_num);

        ComplexGateMembers {
            simple_gate: BasicGateMembers::new(
                input_num,
                output_num,
                gate_type,
                Some(LOW),
            ),
            input_gates,
            output_gates,
            gate_tags_to_index,
        }
    }

    pub fn calculate_output_from_inputs(&mut self, propagate_signal_through_circuit: bool) {
        run_circuit(
            &self.input_gates,
            &self.output_gates,
            propagate_signal_through_circuit,
            &mut |clock_tick_inputs, output_gates| {
                let clock_tick_number = get_clock_tick_number();
                let input_string = format!("Gate {} id {} inputs for clock-tick #{}", self.simple_gate.gate_type, self.simple_gate.unique_id.id, clock_tick_number);
                let output_string = format!("Gate {} id {} outputs for clock-tick #{}", self.simple_gate.gate_type, self.simple_gate.unique_id.id, clock_tick_number);

                pretty_print_output(
                    self.simple_gate.should_print_output,
                    clock_tick_inputs,
                    output_gates,
                    input_string.as_str(),
                    output_string.as_str(),
                );
            },
        );

        self.convert_output_gates_to_output_states();
    }

    pub fn convert_output_gates_to_output_states(&mut self) {
        //simple_gate.output_states represents the actual wrapper around the complex circuit and
        // the outputs associated with it.
        for (i, output_state) in self.simple_gate.output_states.iter_mut().enumerate() {
            let mut output_gate = self.output_gates[i].borrow_mut();

            let output_signals = output_gate.fetch_output_signals().unwrap();

            //The SimpleOutput should always have exactly one output.
            let gate_output_state = output_signals.first().unwrap();

            let new_signal = match gate_output_state {
                GateOutputState::NotConnected(signal) => {
                    signal.clone()
                }
                GateOutputState::Connected(connected_output) => {
                    connected_output.throughput.signal.clone()
                }
            };

            match output_state {
                GateOutputState::NotConnected(ref mut signal) => {
                    *signal = new_signal
                }
                GateOutputState::Connected(ref mut connected_output) => {
                    connected_output.throughput.signal = new_signal
                }
            }
        }
    }

    pub fn get_index_from_tag(&self, tag: &str) -> usize {
        match self.gate_tags_to_index.get(tag) {
            None => {
                panic!("Gate {} id {} did not contain tag {}.", self.simple_gate.gate_type, self.simple_gate.unique_id.id, tag)
            }
            Some(index) => index.clone()
        }
    }

    pub fn connect_output_to_next_gate(&mut self, current_gate_output_key: usize, next_gate_input_key: usize, next_gate: Rc<RefCell<dyn LogicGate>>) {
        //Do not need to run calculate_output_from_inputs() here. It is run in simple gates for the
        // sake of getting the output. However, in complex gates it can be time consuming.

        let signal = match &self.simple_gate.output_states[current_gate_output_key] {
            GateOutputState::NotConnected(signal) => {
                signal.clone()
            }
            GateOutputState::Connected(connected_output) => {
                connected_output.throughput.signal.clone()
            }
        };

        self.simple_gate.output_states[current_gate_output_key] =
            GateOutputState::Connected(
                ConnectedOutput {
                    throughput: GateInput::new(next_gate_input_key, signal),
                    gate: next_gate,
                }
            );
    }

    pub fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        //Updating the inner 'input_signals' vector for consistency.
        self.simple_gate.update_input_signal(input.clone());

        let mut simple_input_gate = self.input_gates[input.input_index].borrow_mut();

        simple_input_gate.update_input_signal(
            GateInput {
                input_index: 0,
                signal: input.signal,
            }
        )
    }

    pub fn fetch_output_signals(&mut self, tag: &String) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.calculate_output_from_inputs(false);

        let output_clone = self.simple_gate.output_states.clone();

        if self.simple_gate.should_print_output {
            GateLogic::print_gate_output(
                &self.simple_gate.gate_type,
                &self.simple_gate.unique_id,
                tag,
                &output_clone,
            );
        }

        Ok(output_clone)
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

    pub fn calculate_output_for_and(input_signals: &Vec<Signal>) -> Signal {
        for s in input_signals.iter() {
            if *s == LOW {
                return LOW;
            }
        }

        HIGH
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

    pub fn calculate_output_for_nor(input_signals: &Vec<Signal>) -> Signal {
        let or_signal = vec![GateLogic::calculate_output_for_or(input_signals)];
        GateLogic::calculate_output_for_not(&or_signal)
    }

    pub fn calculate_output_for_nand(input_signals: &Vec<Signal>) -> Signal {
        let and_signal = vec![GateLogic::calculate_output_for_and(input_signals)];
        GateLogic::calculate_output_for_not(&and_signal)
    }

    pub fn calculate_output_for_controlled_buffer(input_signals: &Vec<Signal>) -> Signal {
        input_signals.first().unwrap().clone()
    }

    pub fn calculate_output_for_clock() -> Signal {
        HIGH
    }

    pub fn calculate_output_for_automatic_input(input_signals: &Vec<Signal>) -> Signal {
        input_signals.first().unwrap().clone()
    }

    pub fn calculate_output_for_simple_input(input_signals: &Vec<Signal>) -> Signal {
        input_signals.first().unwrap().clone()
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
            GateLogic::print_gate_output(
                gate_type,
                &unique_id,
                &String::from(""),
                &output_clone,
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
            GateType::NotType => GateLogic::calculate_output_for_not(input_signals.unwrap()),
            GateType::OrType => GateLogic::calculate_output_for_or(input_signals.unwrap()),
            GateType::AndType => GateLogic::calculate_output_for_and(input_signals.unwrap()),
            GateType::NorType => GateLogic::calculate_output_for_nor(input_signals.unwrap()),
            GateType::NandType => GateLogic::calculate_output_for_nand(input_signals.unwrap()),
            GateType::ControlledBufferType => GateLogic::calculate_output_for_controlled_buffer(input_signals.unwrap()),
            GateType::ClockType => GateLogic::calculate_output_for_clock(),
            GateType::AutomaticInputType => GateLogic::calculate_output_for_automatic_input(input_signals.unwrap()),
            GateType::SimpleOutputType => panic!(),
            GateType::SimpleInputType => GateLogic::calculate_output_for_simple_input(input_signals.unwrap()),
            GateType::SRLatchType => panic!(),
            GateType::ActiveLowSRLatchType => panic!(),
            GateType::OneBitMemoryCellType => panic!(),
            GateType::VariableBitMemoryCellType => panic!(),
            GateType::VariableCPUEnableType => panic!(),
            GateType::VariableBitRegisterType => panic!(),
            GateType::VariableDecoderType => panic!(),
        }
    }

    pub fn print_gate_output<T: fmt::Debug>(
        gate_type: &GateType,
        unique_id: &UniqueID,
        tag: &String,
        output: &T,
    ) {
        if tag.is_empty() {
            println!(
                "{} gate id {} output is\n{:#?}",
                gate_type,
                unique_id.id(),
                output,
            );
        } else {
            println!(
                "{} gate; tag {}; id {}; output is\n{:#?}",
                gate_type,
                tag,
                unique_id.id(),
                output,
            );
        }
    }
}

pub fn pretty_print_output(
    should_print_output: bool,
    clock_tick_inputs: &Vec<(String, Vec<GateOutputState>)>,
    output_gates: &Vec<Rc<RefCell<dyn LogicGateAndOutputGate>>>,
    input_string: &str,
    output_string: &str,
) {

    if should_print_output {
        println!("{}", input_string);
        for (tag, gate_input_state) in clock_tick_inputs.iter() {
            let output_states: Vec<Signal> = gate_input_state
                .iter()
                .map(|g| {
                    match g {
                        GateOutputState::NotConnected(signal) => {
                            signal.clone()
                        }
                        GateOutputState::Connected(connected_output) => {
                            connected_output.throughput.signal.clone()
                        }
                    }
                })
                .collect();
            println!("   tag: {:?} signals: {:?}", tag, output_states);
        }

        println!("{}", output_string);
        for output_gate in output_gates.iter() {
            let mut output_gate = output_gate.borrow_mut();
            let fetched_signal = output_gate.fetch_output_signals().unwrap();
            let output = fetched_signal.first().unwrap();

            if let GateOutputState::NotConnected(signal) = output {
                println!("   tag: {:?} signal: {:?}", output_gate.get_output_tag(), signal);
            } else {
                panic!("An output gate did not have any output.");
            }
        }
    }
}

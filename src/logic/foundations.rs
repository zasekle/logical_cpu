use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::sync::atomic::Ordering;
use crate::globals::{get_clock_tick_number, MAX_INPUT_CHANGES, NEXT_UNIQUE_ID};
use crate::logic::basic_gates::And;
use crate::logic::foundations::Signal::{HIGH, LOW_, NONE};
use crate::logic::input_gates::SimpleInput;
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
use crate::run_circuit::{count_gates_in_circuit, run_circuit};
use crate::shared_mutex::SharedMutex;

//NONE includes some complications. For example when two connections are made to the same
// input, NONE must not ever override another signal. However, a single input can have
// multiple NONE values connected and a single other signal type. In order to fix this, all
// inputs for the round are saved inside input_signals in BasicGateMembers (or something
// similar) and the inputs are checked whenever fetch_output() is called. When
// connect_output() is called, it will add the value to input_signals. Then it
// will update the value when update_input_signal() is called.
#[derive(PartialEq, Debug, Clone)]
pub enum Signal {
    NONE,
    LOW_,
    //The _ is just to line up the width for visual purposes.
    HIGH,
}

#[derive(PartialEq, Debug, Clone)]
pub enum GateLogicError {
    NoMoreAutomaticInputsRemaining,
    MultipleValidSignalsWhenCalculating,
}

#[derive(Debug, Clone)]
pub struct GateInput {
    pub input_index: usize,
    pub signal: Signal,
    pub sending_id: UniqueID,
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

    pub fn zero_id() -> Self {
        Self { id: 0 }
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
    pub fn new(
        input_index: usize,
        signal: Signal,
        sending_id: UniqueID,
    ) -> Self {
        GateInput {
            input_index,
            signal,
            sending_id,
        }
    }
}

#[derive(Debug)]
pub struct InputSignalReturn {
    pub changed_count_this_tick: usize,
    pub input_signal_updated: bool,
}

pub trait LogicGate: Send {
    fn internal_connect_output(
        &mut self,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: SharedMutex<dyn LogicGate>
    ) -> Signal;

    fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal);

    fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn;

    fn fetch_output_signals_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError>;

    fn fetch_output_signals_no_calculate(&mut self) -> Result<Vec<GateOutputState>, GateLogicError>;

    fn get_gate_type(&self) -> GateType;

    fn get_unique_id(&self) -> UniqueID;

    fn toggle_output_printing(&mut self, print_output: bool);

    fn get_tag(&self) -> String;

    fn set_tag(&mut self, tag: &str);

    fn is_input_gate(&self) -> bool {
        false
    }

    fn get_index_from_tag(&self, tag: &str) -> usize {
        panic!("Gate {} using tag {} id {} did not implement get_index_from_tag()", self.get_tag(), tag, self.get_unique_id().id)
    }

    fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID);

    fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool);

    fn num_children_gates(&self) -> usize;

    fn get_input_gates(&self) -> Vec<SharedMutex<dyn LogicGate>>;
}

#[derive(Debug, Clone)]
pub enum GateOutputState {
    NotConnected(Signal),
    Connected(ConnectedOutput),
}

#[derive(Clone)]
pub struct ConnectedOutput {
    pub throughput: GateInput,
    pub gate: SharedMutex<dyn LogicGate>,
}

impl fmt::Debug for ConnectedOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        //todo delete println
        println!("ConnectedOutput about to lock");
        let mut_gate = self.gate.lock().unwrap();
        println!("ConnectedOutput gate_type");
        let gate_type = mut_gate.get_gate_type();
        println!("ConnectedOutput tag");
        let tag = mut_gate.get_tag();
        println!("ConnectedOutput id");
        let id = mut_gate.get_unique_id();

        drop(mut_gate);
        println!("ConnectedOutput unlocked");

        let output_str =
            if tag.is_empty() {
                format!("{} gate with id {}", gate_type, id.id)
            } else {
                format!("{} gate with tag {}; id {}", gate_type, tag, id.id)
            };

        f.debug_struct("OutputNode")
            .field("input", &self.throughput)
            .field("gate", &output_str)
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

    pub fn detect_oscillation(
        &mut self,
        gate_type: &GateType,
        id: &UniqueID,
        tag: &str,
        input_id: &UniqueID,
    ) -> usize
    {
        //If the zero id was used, don't detect oscillation. This is because when a gate needs to
        // update its own input, it can update the clock tick and cause problems for higher level
        // gates that are trying to prime their circuit (it won't propagate).
        if input_id.id == 0 {
            return self.changed_count_this_tick;
        }

        let clock_tick_number = get_clock_tick_number();

        if self.current_tick == clock_tick_number {
            self.changed_count_this_tick += 1;

            if self.changed_count_this_tick >= MAX_INPUT_CHANGES {
                panic!(
                    "Oscillation (a loop) was detected on the current {} gate id {} tag {}",
                    gate_type,
                    id.id,
                    tag,
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
    #[allow(dead_code)]
    UnknownType,
    //Used in testing.
    NotType,
    OrType,
    AndType,
    NorType,
    NandType,
    XOrType,
    SplitterType,
    ControlledBufferType,
    SignalGatekeeperType,
    ClockType,
    AutomaticInputType,
    SimpleOutputType,
    SimpleInputType,
    SRLatchType,
    ActiveLowSRLatchType,
    OneBitMemoryCellType,
    VariableBitMemoryCellType,
    VariableCPUEnableType,
    MasterSlaveJKFlipFlopType,
    FourCycleClockHookupType,
    VariableBitCounterType,
    VariableBitMultiplexerType,
    VariableBitRegisterType,
    VariableDecoderType,
    VariableSingleRAMCellType,
    RAMUnitType,
    HalfAdderType,
    FullAdderType,
    VariableBitAdderType,
    VariableBitShiftLeftType,
    VariableBitNotType,
    VariableBitAndType,
    VariableBitOrType,
    XOrLEType,
    VariableBitXOrLEType,
    VariableBitZType,
    VariableBitEnableType,
    ArithmeticLogicUnitType,
    VariableBitBusOneType,
    VariableOutputStepperType,
    ControlSectionType,
    VariableBitCPUType,
}

impl fmt::Display for GateType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let printable = match *self {
            GateType::UnknownType => "UNKNOWN_TYPE",
            GateType::NotType => "NOT",
            GateType::OrType => "OR",
            GateType::AndType => "AND",
            GateType::NorType => "NOR",
            GateType::NandType => "NAND",
            GateType::XOrType => "XOR",
            GateType::SplitterType => "SPLITTER",
            GateType::ControlledBufferType => "CONTROLLED_BUFFER",
            GateType::SignalGatekeeperType => "SIGNAL_GATEKEEPER",
            GateType::ClockType => "CLOCK",
            GateType::AutomaticInputType => "AUTOMATIC_INPUT",
            GateType::SimpleOutputType => "SIMPLE_OUTPUT",
            GateType::SimpleInputType => "SIMPLE_INPUT",
            GateType::SRLatchType => "SR_LATCH",
            GateType::ActiveLowSRLatchType => "ACTIVE_LOW_SR_LATCH",
            GateType::OneBitMemoryCellType => "ONE_BIT_MEMORY_CELL",
            GateType::VariableBitMemoryCellType => "VARIABLE_BIT_MEMORY_CELL",
            GateType::VariableCPUEnableType => "VARIABLE_CPU_ENABLE",
            GateType::MasterSlaveJKFlipFlopType => "MASTER_SLAVE_JK_FLIP_FLOP",
            GateType::FourCycleClockHookupType => "FOUR_CYCLE_CLOCK_HOOKUP",
            GateType::VariableBitCounterType => "VARIABLE_BIT_COUNTER",
            GateType::VariableBitMultiplexerType => "VARIABLE_BIT_MULTIPLEXER",
            GateType::VariableBitRegisterType => "VARIABLE_BIT_REGISTER",
            GateType::VariableDecoderType => "VARIABLE_DECODER",
            GateType::VariableSingleRAMCellType => "VARIABLE_SINGLE_RAM_CELL",
            GateType::RAMUnitType => "RAM_UNIT",
            GateType::HalfAdderType => "HALF_ADDER",
            GateType::FullAdderType => "FULL_ADDER",
            GateType::VariableBitAdderType => "VARIABLE_BIT_ADDER",
            GateType::VariableBitShiftLeftType => "VARIABLE_BIT_SHIFT_LEFT",
            GateType::VariableBitNotType => "VARIABLE_BIT_NOT",
            GateType::VariableBitAndType => "VARIABLE_BIT_AND",
            GateType::VariableBitOrType => "VARIABLE_BIT_OR",
            GateType::XOrLEType => "XOR_LE",
            GateType::VariableBitXOrLEType => "VARIABLE_BIT_XOR_LE",
            GateType::VariableBitZType => "VARIABLE_BIT_Z",
            GateType::VariableBitEnableType => "VARIABLE_BIT_ENABLE",
            GateType::ArithmeticLogicUnitType => "ARITHMETIC_LOGIC_UNIT",
            GateType::VariableBitBusOneType => "VARIABLE_BIT_BUS_ONE",
            GateType::VariableOutputStepperType => "VARIABLE_OUTPUT_STEPPER",
            GateType::ControlSectionType => "CONTROL_SECTION",
            GateType::VariableBitCPUType => "VARIABLE_BIT_CPU",
        };
        write!(f, "{}", printable)
    }
}

pub struct BasicGateMembers {
    pub input_signals: Vec<HashMap<UniqueID, Signal>>,
    pub output_states: Vec<GateOutputState>,
    pub unique_id: UniqueID,
    pub oscillation_detection: OscillationDetection,
    pub should_print_output: bool,
    pub print_each_input_output_gate: bool,
    pub gate_type: GateType,
    pub tag: String,
    pub number_child_gates: usize,
}

impl BasicGateMembers {
    pub fn new(
        input_num: usize,
        output_num: usize,
        gate_type: GateType,
        number_child_gates: usize,
        output_signal: Option<Signal>
    ) -> Self {

        //Must have at least one input.
        assert_ne!(input_num, 0);

        let mut result = BasicGateMembers {
            input_signals: vec![HashMap::from([(UniqueID::zero_id(), LOW_)]); input_num],
            output_states: Vec::with_capacity(output_num),
            unique_id: UniqueID::generate(),
            oscillation_detection: OscillationDetection::new(),
            should_print_output: false,
            print_each_input_output_gate: true,
            gate_type,
            tag: String::new(),
            number_child_gates,
        };

        let output_signal = if let Some(signal) = output_signal {
            signal
        } else {
            GateLogic::calculate_output_from_inputs(
                &gate_type,
                &result.input_signals,
            ).unwrap()
        };

        result.output_states.resize_with(
            output_num,
            || GateOutputState::NotConnected(output_signal.clone()),
        );

        result
    }

    pub fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        let changed_count_this_tick = self.oscillation_detection.detect_oscillation(
            &self.gate_type,
            &self.unique_id,
            self.tag.as_str(),
            &input.sending_id,
        );

        let input_signal_updated = if self.input_signals[input.input_index][&input.sending_id] == input.signal {
            false
        } else {
            self.input_signals[input.input_index].insert(input.sending_id, input.signal);
            true
        };

        InputSignalReturn {
            changed_count_this_tick,
            input_signal_updated,
        }
    }

    pub fn connect_output(
        &mut self,
        current_gate_output_index: usize,
        next_gate_input_index: usize,
        next_gate: SharedMutex<dyn LogicGate>,
    ) -> Signal {
        GateLogic::connect_output(
            self.gate_type,
            self.unique_id,
            &mut self.input_signals,
            &mut self.output_states,
            current_gate_output_index,
            &self.tag,
            next_gate_input_index,
            next_gate,
            self.should_print_output,
        )
    }

    pub fn internal_update_index_to_id(
        &mut self,
        sending_id: UniqueID,
        gate_input_index: usize,
        signal: Signal,
    ) {

        if self.should_print_output {
            println!(
                "Connection TO\n   type {} tag {} id {} index {}",
                self.gate_type,
                self.tag,
                self.unique_id.id,
                gate_input_index,
            );
        }
        //Whenever an input is updated, remove the zero index. Even adding the zero index it will
        // simply be inserted immediately afterwards.
        self.input_signals[gate_input_index].remove(&UniqueID::zero_id());

        //This is a temporary signal. When the input is updated afterwards, it will add it.
        self.input_signals[gate_input_index].insert(sending_id, signal);

        #[cfg(feature = "high_restriction")]
        if self.input_signals[gate_input_index].len() > 1 {
            panic!("A gate had multiple connections to the same input")
        }
    }

    pub fn get_index_from_tag(&self, tag: &str) -> usize {
        if tag.starts_with("i_") || tag.starts_with("o_") {
            let tag_slice = &tag[2..];
            let index_num = tag_slice.parse().unwrap();
            index_num
        } else {
            panic!("Gate {} using tag {} id {} did not exist.", self.gate_type, self.tag, self.unique_id.id())
        }
    }

    pub fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        let input_map = self.input_signals.get_mut(input_index).unwrap();
        let returned_signal = input_map
            .remove(&connected_id)
            .expect(
                format!(
                    "When attempting to disconnect a gate, the gate with type {} id {} tag {} was not connected.",
                    self.gate_type,
                    self.unique_id.id,
                    self.tag
                ).as_str()
            );

        if input_map.is_empty() {
            input_map.insert(UniqueID::zero_id(), returned_signal);
        }
    }

    pub fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.print_each_input_output_gate = print_each_input_output_gate;
    }
}

#[derive(PartialEq)]
pub enum GateTagType {
    Input,
    Output,
}

pub struct GateTagInfo {
    pub index: usize,
    pub tag_type: GateTagType,
}

impl GateTagInfo {
    fn new(index: usize, tag_type: GateTagType) -> Self {
        GateTagInfo {
            index,
            tag_type,
        }
    }
}

pub struct ComplexGateMembers {
    pub simple_gate: BasicGateMembers,
    pub input_gates: Vec<SharedMutex<dyn LogicGate>>,
    pub output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    pub gate_tags_to_index: HashMap<String, GateTagInfo>,
}

impl ComplexGateMembers {
    pub fn new(
        input_num: usize,
        output_num: usize,
        gate_type: GateType,
        mut input_gates: Vec<SharedMutex<dyn LogicGate>>,
        mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    ) -> Self {

        //Must have at least one input.
        assert_ne!(input_num, 0);
        assert_ne!(output_num, 0);


        let mut gate_tags_to_index = HashMap::new();

        for (i, gate) in input_gates.iter_mut().enumerate() {
            gate_tags_to_index.insert(
                gate.lock().unwrap().get_tag(),
                GateTagInfo::new(
                    i,
                    GateTagType::Input,
                ),
            );
        }

        for (i, gate) in output_gates.iter_mut().enumerate() {
            gate_tags_to_index.insert(
                gate.lock().unwrap().get_tag(),
                GateTagInfo::new(
                    i,
                    GateTagType::Output,
                ),
            );
        }

        //Make sure there are enough tags for each gate.
        assert_eq!(gate_tags_to_index.len(), input_num + output_num);

        ComplexGateMembers {
            simple_gate: BasicGateMembers::new(
                input_num,
                output_num,
                gate_type,
                0,
                Some(LOW_),
            ),
            input_gates,
            output_gates,
            gate_tags_to_index,
        }
    }

    fn calculate_output_from_inputs(
        &mut self,
        propagate_signal_through_circuit: bool,
    ) {
        run_circuit(
            &self.input_gates,
            &self.output_gates,
            propagate_signal_through_circuit,
            &mut |clock_tick_inputs, output_gates| {
                let clock_tick_number = get_clock_tick_number();
                let input_string = format!("Gate {} id {} tag {} inputs for clock-tick #{}", self.simple_gate.gate_type, self.simple_gate.unique_id.id, self.simple_gate.tag, clock_tick_number);
                let output_string = format!("Gate {} id {} tag {} outputs for clock-tick #{}", self.simple_gate.gate_type, self.simple_gate.unique_id.id, self.simple_gate.tag, clock_tick_number);

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

    pub fn calculate_output_from_inputs_and_set_child_count(
        &mut self,
        propagate_signal_through_circuit: bool,
    ) {
        self.calculate_output_from_inputs(
            propagate_signal_through_circuit
        );

        self.simple_gate.number_child_gates = count_gates_in_circuit(
            &self.input_gates
        );
    }

    pub fn convert_output_gates_to_output_states(&mut self) {
        //simple_gate.output_states represents the actual wrapper around the complex circuit and
        // the outputs associated with it.
        for (i, output_state) in self.simple_gate.output_states.iter_mut().enumerate() {
            let mut output_gate = self.output_gates[i].lock().unwrap();

            let output_signals = output_gate.fetch_output_signals_calculate().unwrap();

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
            Some(gate_tag_info) => gate_tag_info.index.clone()
        }
    }

    pub fn connect_output(
        &mut self,
        current_gate_id: UniqueID,
        current_gate_output_key: usize,
        next_gate_input_key: usize,
        next_gate: SharedMutex<dyn LogicGate>,
    ) -> Signal {
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

        if self.simple_gate.should_print_output {
            println!(
                "Connection FROM\n   type {} tag {} id {} index {}",
                self.simple_gate.gate_type,
                self.simple_gate.tag,
                self.simple_gate.unique_id.id,
                current_gate_output_key,
            );
        }

        #[cfg(feature = "high_restriction")]
        if let GateOutputState::Connected(output) = self.simple_gate.output_states[current_gate_output_key].clone() {
            panic!("output was already connect and it got connected again\n{:#?}", output)
        }

        self.simple_gate.output_states[current_gate_output_key] =
            GateOutputState::Connected(
                ConnectedOutput {
                    throughput: GateInput::new(
                        next_gate_input_key,
                        signal.clone(),
                        current_gate_id,
                    ),
                    gate: next_gate.clone(),
                }
            );

        signal
    }

    pub fn update_input_signal(&mut self, input: GateInput) -> InputSignalReturn {
        println!("update_input_signal.simple_gate");
        //Updating the inner 'input_signals' vector for consistency.
        self.simple_gate.update_input_signal(input.clone());

        println!("update_input_signal.lock()");
        let mut simple_input_gate = self.input_gates[input.input_index].lock().unwrap();
        println!("update_input_signal.update_input_signal");

        simple_input_gate.update_input_signal(
            GateInput::new(
                0,
                input.signal,
                input.sending_id,
            )
        )
    }

    pub fn fetch_output_signals_calculate(
        &mut self,
        tag: &String,
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        self.calculate_output_from_inputs(
            false,
        );

        self.fetch_output_signals_no_calculate(
            tag
        )
    }

    pub fn fetch_output_signals_no_calculate(
        &mut self,
        tag: &String,
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        //This must be run because the multithreaded version will not calculate the output states
        // themselves.
        self.convert_output_gates_to_output_states();

        let output_clone = self.simple_gate.output_states.clone();

        if self.simple_gate.should_print_output && self.simple_gate.print_each_input_output_gate {
            GateLogic::print_gate_output(
                &self.simple_gate.gate_type,
                &self.simple_gate.unique_id,
                tag,
                &self.simple_gate.input_signals,
                &output_clone,
            );
        }

        Ok(output_clone)
    }

    pub fn internal_update_index_to_id(&mut self, sending_id: UniqueID, gate_input_index: usize, signal: Signal) {
        self.input_gates[gate_input_index].lock().unwrap().internal_update_index_to_id(
            sending_id,
            0,
            signal.clone(),
        );
        self.simple_gate.internal_update_index_to_id(sending_id, gate_input_index, signal);
    }

    pub fn remove_connected_input(&mut self, input_index: usize, connected_id: UniqueID) {
        //SimpleInput input index is always 0.
        self.input_gates[input_index].lock().unwrap().remove_connected_input(
            0, connected_id,
        );

        self.simple_gate.remove_connected_input(
            input_index,
            connected_id,
        );
    }

    pub fn toggle_print_each_input_output_gate(&mut self, print_each_input_output_gate: bool) {
        self.simple_gate.toggle_print_each_input_output_gate(print_each_input_output_gate);
    }

    //TODO: rename this
    // pub fn get_input_gates(&mut self) -> Vec<SharedMutex<dyn LogicGate>> {
    //     //TODO: Is this actually possible?
    //     //Cannot just copy the input_gates Vec. This is because the input_gates are locked inside
    //     // the ComplexGate when calling various methods. This means that if the input_gates are
    //     // returned they can be locked first, call their parent and deadlock can occur.
    //     // for input_gate in self.input_gates {
    //     //     //This is a SimpleInput gate
    //     //     let output = input_gate.lock().unwrap().fetch_output_signals_calculate();
    //     //
    //     //     //TODO: In order to do this, I would need to call update_input_signal on the next gate.
    //     //     // This means that I would be locking the next gate inside this lock as well which would
    //     //     // potentially cause the same problem.
    //     // }
    //     // self.input_gates.clone()
    // }
}

pub struct GateLogic;

impl GateLogic {
    pub fn calculate_output_for_or(input_signals: &Vec<Signal>) -> Signal {
        let mut output_signal = LOW_;
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
            if *s == LOW_ {
                return LOW_;
            }
        }

        HIGH
    }

    pub fn calculate_output_for_not(input_signals: &Vec<Signal>) -> Signal {
        //This only has a single input.
        let current_signal = input_signals.first().unwrap();

        if *current_signal == HIGH {
            LOW_
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

    pub fn calculate_output_for_xor(input_signals: &Vec<Signal>) -> Signal {
        let mut high_signal_exists = false;
        let mut low_signal_exists = false;
        for input in input_signals.iter() {
            match *input {
                NONE => {}
                LOW_ => {
                    low_signal_exists = true;
                }
                HIGH => {
                    high_signal_exists = true;
                }
            }

            if high_signal_exists && low_signal_exists {
                return HIGH;
            }
        }

        if !high_signal_exists && !low_signal_exists {
            NONE
        } else {
            LOW_
        }
    }

    pub fn calculate_output_for_clock(input_signals: &Vec<Signal>) -> Signal {
        if *input_signals.first().unwrap() == LOW_ {
            HIGH
        } else {
            LOW_
        }
    }

    pub fn calculate_output_for_automatic_input(input_signals: &Vec<Signal>) -> Signal {
        input_signals.first().unwrap().clone()
    }

    pub fn calculate_output_for_simple_input(input_signals: &Vec<Signal>) -> Signal {
        input_signals.first().unwrap().clone()
    }

    pub fn fetch_output_signals_calculate_basic_gate(
        basic_gate: &mut BasicGateMembers,
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        Self::fetch_output_signals_calculate(
            &basic_gate.gate_type,
            &basic_gate.input_signals,
            &mut basic_gate.output_states,
            basic_gate.unique_id,
            basic_gate.should_print_output,
            basic_gate.print_each_input_output_gate,
            basic_gate.tag.as_str(),
        )
    }

    pub fn fetch_output_signals_calculate(
        gate_type: &GateType,
        input_signals: &Vec<HashMap<UniqueID, Signal>>,
        output_states: &mut Vec<GateOutputState>,
        unique_id: UniqueID,
        should_print_output: bool,
        print_each_input_output_gate: bool,
        tag: &str,
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        let output_signal = GateLogic::calculate_output_from_inputs(gate_type, input_signals)?;

        set_all_gate_output_to_signal(
            output_states,
            output_signal.clone(),
        );

        let output_clone = output_states.clone();

        if should_print_output && print_each_input_output_gate {
            GateLogic::print_gate_output(
                gate_type,
                &unique_id,
                tag,
                &input_signals,
                &output_clone,
            );
        }

        Ok(output_clone)
    }

    pub fn fetch_output_signals_no_calculate_basic_gate(
        basic_gate: &mut BasicGateMembers,
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        Self::fetch_output_signals_no_calculate(
            &basic_gate.gate_type,
            &basic_gate.input_signals,
            &mut basic_gate.output_states,
            basic_gate.unique_id,
            basic_gate.should_print_output,
            basic_gate.print_each_input_output_gate,
            basic_gate.tag.as_str(),
        )
    }

    pub fn fetch_output_signals_no_calculate(
        gate_type: &GateType,
        input_signals: &Vec<HashMap<UniqueID, Signal>>,
        output_states: &mut Vec<GateOutputState>,
        unique_id: UniqueID,
        should_print_output: bool,
        print_each_input_output_gate: bool,
        tag: &str,
    ) -> Result<Vec<GateOutputState>, GateLogicError> {
        let output_clone = output_states.clone();

        if should_print_output && print_each_input_output_gate {
            GateLogic::print_gate_output(
                gate_type,
                &unique_id,
                tag,
                &input_signals,
                &output_clone,
            );
        }

        Ok(output_clone)
    }

    pub fn connect_output(
        gate_type: GateType,
        current_gate_id: UniqueID,
        input_signals: &Vec<HashMap<UniqueID, Signal>>,
        output_states: &mut Vec<GateOutputState>,
        current_gate_output_index: usize,
        current_gate_tag: &str,
        next_gate_input_index: usize,
        next_gate: SharedMutex<dyn LogicGate>,
        should_print_output: bool,
    ) -> Signal {
        //When gates are being connected, there should be no issues with this error. So unwrapping
        // it.
        let output_signal = GateLogic::calculate_output_from_inputs(
            &gate_type,
            input_signals,
        ).unwrap();

        GateLogic::connect_output_no_calculate(
            current_gate_id,
            output_states,
            current_gate_output_index,
            next_gate_input_index,
            next_gate,
            output_signal.clone(),
            gate_type,
            current_gate_tag,
            should_print_output,
        );

        output_signal
    }

    pub fn connect_output_no_calculate(
        current_gate_id: UniqueID,
        output_states: &mut Vec<GateOutputState>,
        current_gate_output_index: usize,
        next_gate_input_index: usize,
        next_gate: SharedMutex<dyn LogicGate>,
        output_signal: Signal,
        current_gate_type: GateType,
        current_gate_tag: &str,
        should_print_output: bool,
    ) {

        if should_print_output {
            if current_gate_tag.is_empty() {
                println!(
                    "Connection for\n   type {} id {} index {}",
                    current_gate_type,
                    current_gate_id.id(),
                    current_gate_output_index,
                );
            } else {
                println!(
                    "Connection for\n   type {} tag {} id {} index {}",
                    current_gate_type,
                    current_gate_tag,
                    current_gate_id.id(),
                    current_gate_output_index,
                );
            }
        }

        #[cfg(feature = "high_restriction")]
        if let GateOutputState::Connected(output) = output_states[current_gate_output_index].clone() {
            panic!("output was already connect and it got connected again\n{:#?}", output)
        }

        output_states[current_gate_output_index] =
            GateOutputState::Connected(
                ConnectedOutput {
                    throughput: GateInput::new(
                        next_gate_input_index,
                        output_signal,
                        current_gate_id,
                    ),
                    gate: next_gate.clone(),
                }
            );
    }

    pub fn calculate_output_from_inputs(
        gate_type: &GateType,
        input_signals: &Vec<HashMap<UniqueID, Signal>>,
    ) -> Result<Signal, GateLogicError> {
        let input_signals = calculate_input_signals_from_all_inputs(input_signals)?;

        let output_signal = match gate_type {
            GateType::NotType => GateLogic::calculate_output_for_not(&input_signals),
            GateType::OrType => GateLogic::calculate_output_for_or(&input_signals),
            GateType::AndType => GateLogic::calculate_output_for_and(&input_signals),
            GateType::NorType => GateLogic::calculate_output_for_nor(&input_signals),
            GateType::NandType => GateLogic::calculate_output_for_nand(&input_signals),
            GateType::XOrType => GateLogic::calculate_output_for_xor(&input_signals),
            GateType::ClockType => GateLogic::calculate_output_for_clock(&input_signals),
            GateType::AutomaticInputType => GateLogic::calculate_output_for_automatic_input(&input_signals),
            GateType::SimpleInputType => GateLogic::calculate_output_for_simple_input(&input_signals),
            _ => panic!("calculate_outputs_from_inputs called with invalid gate_type of {}", gate_type)
        };

        Ok(output_signal)
    }

    pub fn print_gate_output<T: fmt::Debug, U: fmt::Debug>(
        gate_type: &GateType,
        unique_id: &UniqueID,
        tag: &str,
        input: &T,
        output: &U,
    ) {
        if tag.is_empty() {
            println!(
                "{} gate id {}\ninput is {:#?}\noutput is {:#?}",
                gate_type,
                unique_id.id(),
                input,
                output,
            );
        } else {
            println!(
                "{} gate tag {} id {}\ninput is {:#?}\noutput is {:#?}",
                gate_type,
                tag,
                unique_id.id(),
                input,
                output,
            );
        }
    }
}

pub fn pretty_print_output(
    should_print_output: bool,
    clock_tick_inputs: &Vec<(String, Vec<GateOutputState>)>,
    output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
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
            let mut output_gate = output_gate.lock().unwrap();
            let fetched_signal = output_gate.fetch_output_signals_calculate().unwrap();
            let output = fetched_signal.first().unwrap();

            if let GateOutputState::NotConnected(signal) = output {
                println!("   tag: {:?} signal: {:?}", output_gate.get_output_tag(), signal);
            } else {
                panic!("An output gate did not have any output.");
            }
        }
    }
}

pub fn calculate_input_signals_from_all_inputs(
    input_signals: &Vec<HashMap<UniqueID, Signal>>,
) -> Result<Vec<Signal>, GateLogicError> {
    let mut final_signals = Vec::new();
    for input in input_signals {
        final_signals.push(calculate_input_signal_from_single_inputs(input)?);
    }
    Ok(final_signals)
}

pub fn calculate_input_signal_from_single_inputs(
    input_signal: &HashMap<UniqueID, Signal>,
) -> Result<Signal, GateLogicError> {
    let mut final_signal = NONE;
    for (_id, signal) in input_signal {
        if final_signal == NONE {
            final_signal = signal.clone();
        } else if *signal != NONE {
            // There is a problem that can occur here when multiple signals are found going into
            // the same input. This is an unknown state. However, that does not mean an error
            // occurred
            // As an example, say I have Gate A and Gate B that are connected to Output 1.
            // Initially, Gate A is LOW and Gate B is NONE. Then next tick they are meant to switch
            // so Gate A is NONE and Gate B is HIGH, if Gate B sends in its signal first, then
            // Output 1 will have a LOW and a HIGH signal which is an unknown state.
            // The way this is handled is inside the run_circuit() function. It will handle this
            // error and delay continuing with the gate until the state can be determined.
            return Err(GateLogicError::MultipleValidSignalsWhenCalculating);
        }
    }
    Ok(final_signal)
}

pub fn build_simple_inputs_and_outputs(
    number_inputs_outputs: usize,
    input_gates: &mut Vec<SharedMutex<dyn LogicGate>>,
    output_gates: &mut Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    output_gates_logic: &mut Vec<SharedMutex<dyn LogicGate>>,
) {
    for i in 0..number_inputs_outputs {
        let input_tag = format!("i_{}", i);
        input_gates.push(SimpleInput::new(1, input_tag.as_str()));

        let output_tag = format!("o_{}", i);
        let output_gate = SimpleOutput::new(output_tag.as_str());
        output_gates.push(output_gate.clone());
        output_gates_logic.push(output_gate);
    }
}

pub fn build_simple_inputs_and_outputs_with_and(
    number_inputs_outputs: usize,
    input_gates: &mut Vec<SharedMutex<dyn LogicGate>>,
    output_gates: &mut Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    output_gates_logic: &mut Vec<SharedMutex<dyn LogicGate>>,
    and_gates: &mut Vec<SharedMutex<And>>,
) {
    for i in 0..number_inputs_outputs {
        let input_tag = format!("i_{}", i);
        input_gates.push(SimpleInput::new(1, input_tag.as_str()));

        let output_tag = format!("o_{}", i);
        let output_gate = SimpleOutput::new(output_tag.as_str());
        output_gates.push(output_gate.clone());
        output_gates_logic.push(output_gate);

        and_gates.push(
            And::new(2, 1)
        );
    }
}

pub fn push_reg_outputs_to_output_gates(
    number_inputs_outputs: usize,
    output_gates: &mut Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    output_gates_logic: &mut Vec<SharedMutex<dyn LogicGate>>,
) {
    //These must be pushed to the array AFTER all the output gates are.
    for i in 0..number_inputs_outputs {
        let reg_output_tag = format!("reg_{}", i);
        let reg_output_gate = SimpleOutput::new(reg_output_tag.as_str());
        output_gates.push(reg_output_gate.clone());
        output_gates_logic.push(reg_output_gate);
    }
}

pub fn set_all_gate_output_to_signal(
    output_states: &mut Vec<GateOutputState>,
    new_signal: Signal
) {
    for output in output_states.iter_mut() {
        match output {
            GateOutputState::NotConnected(signal) => {
                *signal = new_signal.clone();
            }
            GateOutputState::Connected(connected_output) => {
                connected_output.throughput.signal = new_signal.clone();
            }
        }
    }
}

pub fn connect_gates(
    output_gate: SharedMutex<dyn LogicGate>,
    output_index: usize,
    input_gate: SharedMutex<dyn LogicGate>,
    input_index: usize,
) {


    let output_signal = output_gate.lock().unwrap().internal_connect_output(
        output_index,
        input_index,
        input_gate.clone(),
    );


    let output_id = output_gate.lock().unwrap().get_unique_id();


    input_gate.lock().unwrap().internal_update_index_to_id(
        output_id,
        input_index,
        output_signal
    );

}
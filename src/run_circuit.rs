use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::{fmt, thread};
use std::fmt::Formatter;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use crate::globals::{CLOCK_TICK_NUMBER, END_OUTPUT_GATE_TAG, get_clock_tick_number, RUN_CIRCUIT_IS_HIGH_LEVEL};
use crate::logic::foundations::{connect_gates, GateInput, GateLogicError, GateOutputState, InputSignalReturn, LogicGate, Signal, UniqueID};
use crate::logic::foundations::Signal::{HIGH, LOW_};
use crate::logic::input_gates::{AutomaticInput, Clock};
use crate::logic::output_gates::{LogicGateAndOutputGate, SimpleOutput};
use crate::logic::processor_components::RAMUnit;
use crate::logic::variable_bit_cpu::VariableBitCPU;
use crate::{ALU_TIME, CONTROL_SECTION_TIME, RAM_TIME};
use crate::shared_mutex::SharedMutex;
use crate::test_stuff::extract_output_tags_sorted_by_index;

//TODO: set this to a higher value
//TODO: assert somewhere that this value is greater than 0
static NUM_CHILDREN_GATES_FOR_LARGE_GATE: usize = 7;

pub struct CondvarWrapper {
    cond: Condvar,
    mutex: Mutex<()>,
}

impl CondvarWrapper {
    fn new() -> Self {
        CondvarWrapper {
            cond: Condvar::new(),
            mutex: Mutex::new(()),
        }
    }

    fn wait(&self) {
        let guard = self.mutex.lock().unwrap();
        let _unused_guard = self.cond.wait(guard).unwrap();
    }
}

#[derive(Clone, Debug)]
enum ProcessingGateStatus {
    Running,
    Redo,
    Cancel,
}

enum ProcessingSizeOfGate {
    Large {
        outstanding_children: usize,
        gate: SharedMutex<dyn LogicGate>,
        multiple_valid_input_gates: Vec<SharedMutex<dyn LogicGate>>,
    },
    Small,
}

impl fmt::Debug for ProcessingSizeOfGate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output_str =
            match self {
                ProcessingSizeOfGate::Large {
                    outstanding_children,
                    gate,
                    multiple_valid_input_gates,
                } => {
                    let mut_gate = gate.lock().unwrap();
                    let gate_type = mut_gate.get_gate_type();
                    let tag = mut_gate.get_tag();
                    let id = mut_gate.get_unique_id();

                    drop(mut_gate);
                    format!("'ProcessingSizeOfGate::Large' {} gate with tag {}; id {}\noutstanding_children: {}\nmultiple_valid_input_gates.len(): {}", gate_type, tag, id.id(), outstanding_children, multiple_valid_input_gates.len())
                }
                ProcessingSizeOfGate::Small => {
                    format!("'ProcessingSizeOfGate::Small'")
                }
            };

        f.debug_struct(&output_str)
            .finish()
    }
}

#[derive(Debug)]
enum WaitingSizeOfGate {
    Large {
        outstanding_children: usize,
        initially_added: bool,
    },
    Small,
}

impl WaitingSizeOfGate {
    fn convert_from_processing(size: &ProcessingSizeOfGate) -> Self {
        match size {
            ProcessingSizeOfGate::Large { outstanding_children, .. } => Self::Large {
                outstanding_children: *outstanding_children,
                initially_added: false,
            },
            ProcessingSizeOfGate::Small => Self::Small
        }
    }
}

#[derive(Debug)]
struct ProcessingGate {
    parent_id: UniqueID,
    processing_type: ProcessingSizeOfGate,
    status: ProcessingGateStatus,
}

struct ParentIdAndGate {
    parent_id: UniqueID,
    gate: SharedMutex<dyn LogicGate>,
}

pub struct ThreadPoolLists {
    //The key is the parent ID and the set is the child IDs.
    parental_tree: HashMap<UniqueID, HashSet<UniqueID>>,
    //The gates that are currently being run by a thread. These are not in the 'gates' Vec.
    processing_set: HashMap<UniqueID, ProcessingGate>,
    //The gates that are waiting to be processed by a thread. These are in the 'gates' Vec. The
    // usize is 'outstanding_children' it will be 0 if the gate is completed.
    waiting_to_be_processed_set: HashMap<UniqueID, WaitingSizeOfGate>,
    gates: VecDeque<ParentIdAndGate>,
    input_gate_output_states: Vec<(String, Vec<GateOutputState>)>,
}

impl ThreadPoolLists {
    pub fn clear(&mut self) {
        //This needs to be cleared, all gates are assumed to be removed from parental_tree when
        // Cancel is set as the status.
        self.parental_tree = HashMap::from([(UniqueID::zero_id(), HashSet::new())]);

        for (_id, processing_gate) in self.processing_set.iter_mut() {
            processing_gate.status = ProcessingGateStatus::Cancel;
        }

        self.waiting_to_be_processed_set.clear();
        self.gates.clear();
    }
}

pub struct RunCircuitThreadPool {
    thread_pool_lists: Arc<Mutex<ThreadPoolLists>>,

    threads: Vec<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    propagate_signal: Arc<AtomicBool>,
    condvar_wrapper: Arc<CondvarWrapper>,
    num_threads_running: Arc<AtomicI32>,
    processing_completed: Arc<Mutex<bool>>,
    wait_for_completion: Arc<Condvar>,
}

pub struct QueueElement {
    gate: SharedMutex<dyn LogicGate>,
    parent_id: UniqueID,
    gate_id: UniqueID,
    number_children_in_gate: usize,
}

//TODO: For now the goal here is to get a working interface. Performance can be improved upon later.
impl RunCircuitThreadPool {
    pub fn new(size: usize) -> Self {
        assert_ne!(size, 0);
        let mut thread_pool = RunCircuitThreadPool {
            thread_pool_lists: Arc::new(
                Mutex::new(
                    ThreadPoolLists {
                        parental_tree: HashMap::from([(UniqueID::zero_id(), HashSet::new())]),
                        processing_set: HashMap::new(),
                        waiting_to_be_processed_set: HashMap::new(),
                        gates: VecDeque::new(),
                        input_gate_output_states: Vec::new(),
                    }
                )
            ),
            threads: Vec::new(),
            shutdown: Arc::new(AtomicBool::from(false)),
            propagate_signal: Arc::new(AtomicBool::from(false)),
            condvar_wrapper: Arc::new(CondvarWrapper::new()),
            num_threads_running: Arc::new(AtomicI32::new(0)),
            processing_completed: Arc::new(Mutex::new(false)),
            wait_for_completion: Arc::new(Condvar::new()),
        };

        for i in 0..size {
            let mut shutdown_clone = thread_pool.shutdown.clone();
            let mut thread_pool_lists_clone = thread_pool.thread_pool_lists.clone();
            let mut signal_clone = thread_pool.condvar_wrapper.clone();
            let propagate_signal_clone = thread_pool.propagate_signal.clone();
            let num_threads_running_clone = thread_pool.num_threads_running.clone();

            let mut processing_completed_clone = thread_pool.processing_completed.clone();
            let mut wait_for_completion_clone = thread_pool.wait_for_completion.clone();
            //This must be set before the thread starts. That way it can be used below in this function
            // to check when all threads are initialized.
            num_threads_running_clone.fetch_add(1, Ordering::Acquire);

            thread_pool.threads.push(
                thread::spawn(move || {
                    println!("Thread {i} started");

                    loop {
                        if shutdown_clone.load(Ordering::Acquire) {
                            println!("Thread {i} shutting down");
                            break;
                        }

                        let mut large_gate_completed = false;
                        let popped_element;
                        loop {
                            //The lock will be held as long as the MutexGuard is alive. So I
                            // need to create a scope to make sure the lock is not held for the
                            // duration of the task being run.
                            let mut thread_pool_lists = thread_pool_lists_clone.lock().unwrap();

                            let front_gate = thread_pool_lists.gates.pop_front();

                            if let Some(gate) = &front_gate {
                                let mutable_gate = gate.gate.lock().unwrap();
                                let gate_id = mutable_gate.get_unique_id().clone();
                                let gate_num_children = mutable_gate.num_children_gates();
                                drop(mutable_gate);

                                println!("\n\npopped_element gate_id {} parent_id {}", gate_id.id(), gate.parent_id.id());

                                let waiting_to_be_processed = thread_pool_lists.waiting_to_be_processed_set.remove(
                                    &gate_id
                                );

                                if let Some(waiting_element) = waiting_to_be_processed {
                                    if let WaitingSizeOfGate::Large { outstanding_children, initially_added } = waiting_element
                                    {
                                        if outstanding_children == 0 && !initially_added {
                                            println!("0 outstanding_children gate_id {}", gate_id.id());
                                            println!("waiting_to_be_processed_set {:#?}", thread_pool_lists.waiting_to_be_processed_set);

                                            // popped_element = front_gate;
                                            large_gate_completed = true;
                                            // break;
                                        }
                                    }
                                } else {
                                    println!("not in waiting_to_be_processed_set");
                                    //If the gate was canceled, then it will not exist inside the
                                    // waiting_to_be_processed_set.
                                    continue;
                                }

                                let processing_type =
                                    if gate_num_children < NUM_CHILDREN_GATES_FOR_LARGE_GATE {
                                        ProcessingSizeOfGate::Small
                                    } else {
                                        //outstanding_children will be updated below when the input gates are added.
                                        ProcessingSizeOfGate::Large {
                                            outstanding_children: 0,
                                            gate: gate.gate.clone(),
                                            multiple_valid_input_gates: Vec::new(),
                                        }
                                    };

                                thread_pool_lists.processing_set.insert(
                                    gate_id,
                                    ProcessingGate {
                                        parent_id: gate.parent_id.clone(),
                                        processing_type,
                                        status: ProcessingGateStatus::Running,
                                    },
                                );
                            }

                            popped_element = front_gate;
                            break;
                        };

                        if let Some(running_gate) = popped_element {
                            println!("Thread {i} running task");

                            let element_num_children = running_gate.gate.lock().unwrap().num_children_gates();

                            let mut clock_tick_inputs = Vec::new();
                            let mut next_gates = Vec::new();
                            let mut next_gates_set = HashSet::new();
                            let mut multiple_valid_signals = Vec::new();
                            let mut number_gates_that_ran = 0;
                            let mut parent_id = running_gate.parent_id;
                            if element_num_children < NUM_CHILDREN_GATES_FOR_LARGE_GATE
                                || large_gate_completed {
                                let mut mutable_running_gate = running_gate.gate.lock().unwrap();
                                let fetched_signals =
                                    if large_gate_completed {
                                        mutable_running_gate.fetch_output_signals_no_calculate()
                                    } else {
                                        mutable_running_gate.fetch_output_signals_calculate()
                                    };

                                let is_input_gate = mutable_running_gate.is_input_gate();
                                let gate_tag = mutable_running_gate.get_tag();

                                drop(mutable_running_gate);

                                match fetched_signals {
                                    Ok(output_states) => {
                                        if is_input_gate {
                                            clock_tick_inputs.push(
                                                (gate_tag.clone(), output_states.clone())
                                            );
                                        }
                                        println!("output_states {:#?}", output_states);

                                        for gate_output_state in output_states {
                                            match gate_output_state {
                                                GateOutputState::NotConnected(signal) => {
                                                    if gate_tag == END_OUTPUT_GATE_TAG
                                                        && signal == HIGH {
                                                        println!("End of program reached on clock-tick {}. Stopping execution.", get_clock_tick_number());
                                                        Self::internal_shutdown(
                                                            &mut shutdown_clone,
                                                            &mut signal_clone,
                                                            &mut thread_pool_lists_clone,
                                                        );
                                                    }
                                                }
                                                GateOutputState::Connected(next_gate_info) => {
                                                    let next_gate = next_gate_info.gate.clone();
                                                    let mut mutable_next_gate = next_gate.lock().unwrap();

                                                    let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                                                        mutable_next_gate.update_input_signal(next_gate_info.throughput.clone());
                                                    let gate_id = mutable_next_gate.get_unique_id();

                                                    let contains_id = next_gates_set.contains(&gate_id);

                                                    let should_update_gate = check_if_next_gate_should_be_stored(
                                                        input_signal_updated,
                                                        changed_count_this_tick,
                                                        contains_id,
                                                        propagate_signal_clone.load(Ordering::Relaxed),
                                                    );

                                                    if should_update_gate {
                                                        let number_children_in_gate = mutable_next_gate.num_children_gates();
                                                        drop(mutable_next_gate);
                                                        next_gates_set.insert(gate_id);
                                                        next_gates.push(
                                                            QueueElement {
                                                                gate: next_gate,
                                                                parent_id: parent_id.clone(),
                                                                gate_id,
                                                                number_children_in_gate,
                                                            }
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        match err {
                                            GateLogicError::NoMoreAutomaticInputsRemaining => {
                                                println!("No More AutomaticInputs remaining. Shutting down.");
                                                Self::internal_shutdown(
                                                    &mut shutdown_clone,
                                                    &mut signal_clone,
                                                    &mut thread_pool_lists_clone,
                                                );
                                            }
                                            GateLogicError::MultipleValidSignalsWhenCalculating => {
                                                multiple_valid_signals.push(running_gate.gate.clone());
                                            }
                                        }
                                    }
                                }
                                number_gates_that_ran = 1;
                            } else {
                                let mutable_running_gate = running_gate.gate.lock().unwrap();
                                //When the gates are added below, the parent id will be the current gate for a large gate.
                                parent_id = mutable_running_gate.get_unique_id();
                                let input_gates = mutable_running_gate.get_input_gates();

                                println!("LARGE GATE {} input_gates_size {}", mutable_running_gate.get_unique_id().id(), input_gates.len());

                                drop(mutable_running_gate);

                                for input_gate in input_gates.into_iter() {
                                    let mutable_input_gate = input_gate.lock().unwrap();

                                    let gate_id = mutable_input_gate.get_unique_id();

                                    let contains_id = next_gates_set.contains(&gate_id);

                                    if !contains_id {
                                        let number_children_in_gate = mutable_input_gate.num_children_gates();
                                        drop(mutable_input_gate);
                                        next_gates_set.insert(gate_id);
                                        next_gates.push(
                                            QueueElement {
                                                gate: input_gate,
                                                parent_id,
                                                gate_id,
                                                number_children_in_gate,
                                            }
                                        );
                                    }
                                }
                            }

                            println!("next_gates.len() {}", next_gates.len());

                            let mut thread_pool_lists_guard = thread_pool_lists_clone.lock().unwrap();

                            let gate_id = running_gate.gate.lock().unwrap().get_unique_id();

                            let mut thread_pool_lists: &mut ThreadPoolLists = &mut thread_pool_lists_guard;

                            let processing_element = thread_pool_lists.processing_set.get(
                                &gate_id
                            ).expect("A gate was removed from the processing_set while it was running.");

                            thread_pool_lists.input_gate_output_states.append(
                                &mut clock_tick_inputs
                            );

                            match processing_element.status {
                                ProcessingGateStatus::Running => {
                                    let num_new_children =
                                        if number_gates_that_ran > 0 {
                                            //Small gates do not have their own set in the parental tree.
                                            // thread_pool_lists.parental_tree.remove(
                                            //     &gate_id
                                            // );

                                            thread_pool_lists.processing_set.remove(
                                                &gate_id
                                            );

                                            //Remove gate as a sibling.
                                            let siblings = thread_pool_lists.parental_tree.get_mut(
                                                &parent_id
                                            ).expect(
                                                "A sibling completed when its parent tree \
                                                    was removed. This should never happen because the parent tree should \
                                                    never be completed before the children tree is."
                                            );

                                            siblings.remove(
                                                &gate_id
                                            );

                                            //Keep only the gates that are not already being processed.
                                            let mut num_new_children = next_gates.len();

                                            for next_gate in next_gates.iter() {
                                                if siblings.contains(&next_gate.gate_id) {
                                                    num_new_children -= 1;
                                                }
                                            }

                                            num_new_children
                                        } else { //Large gate was inserted into processing.
                                            //Insert gate as a parent.
                                            thread_pool_lists.parental_tree.insert(
                                                gate_id.clone(), HashSet::new(),
                                            );

                                            next_gates.len()
                                        };

                                    println!("num_new_children {} number_gates_that_ran {}", next_gates.len(), number_gates_that_ran);
                                    Self::update_parent_gate(
                                        thread_pool_lists,
                                        num_new_children,
                                        number_gates_that_ran,
                                        &parent_id,
                                        multiple_valid_signals,
                                    );

                                    Self::add_to_queue_internal(
                                        &mut thread_pool_lists,
                                        next_gates,
                                        &mut signal_clone,
                                    );
                                }
                                ProcessingGateStatus::Redo => {
                                    thread_pool_lists.waiting_to_be_processed_set.insert(
                                        gate_id,
                                        WaitingSizeOfGate::convert_from_processing(
                                            &processing_element.processing_type
                                        ),
                                    );

                                    thread_pool_lists.processing_set.remove(
                                        &gate_id
                                    );

                                    //Add this to the front so it will immediately re-run.
                                    thread_pool_lists.gates.push_front(running_gate);

                                    //If this gate needs to redo, it is still in parental_tree.
                                }
                                ProcessingGateStatus::Cancel => {
                                    thread_pool_lists.processing_set.remove(
                                        &gate_id
                                    );

                                    //If this gate was canceled, it was already removed from
                                    // parental_tree.
                                }
                            }

                            println!("waiting_to_be_processed_set.is_empty() {} num_threads_running {}", thread_pool_lists.waiting_to_be_processed_set.is_empty(), num_threads_running_clone.load(Ordering::Relaxed));
                            println!("waiting_to_be_processed_set {:#?}", thread_pool_lists.waiting_to_be_processed_set);
                            println!("gates.len() {:#?}", thread_pool_lists.gates.len());
                            //This is the only thread running and there are no more elements to be
                            // processed.
                            if thread_pool_lists.waiting_to_be_processed_set.is_empty()
                                && num_threads_running_clone.load(Ordering::Relaxed) == 1
                            {
                                println!("complete_queue() called");
                                Self::complete_queue(
                                    &mut processing_completed_clone,
                                    &mut wait_for_completion_clone,
                                );
                            }
                        } else {
                            println!("Thread {i} sleeping");

                            let waiting_set_is_empty = {
                                let mut thread_pool_lists_guard = thread_pool_lists_clone.lock().unwrap();
                                println!("gates.len() {}", thread_pool_lists_guard.gates.len());

                                thread_pool_lists_guard.waiting_to_be_processed_set.is_empty()
                            };

                            //TODO: the problem is that if I do this, then it will immediately end
                            // if only one thread exists
                            //This is the only thread running and there are no more elements to be
                            // processed.
                            // if waiting_set_is_empty
                            //     && num_threads_running_clone.load(Ordering::Relaxed) == 1
                            // {
                            //     println!("complete_queue() called");
                            //     Self::complete_queue(
                            //         &mut processing_completed_clone,
                            //         &mut wait_for_completion_clone,
                            //     );
                            // }

                            //todo: delete
                            let processing_set_len = thread_pool_lists_clone.lock().unwrap().processing_set.len();
                            let waiting_to_pro_set_len = thread_pool_lists_clone.lock().unwrap().waiting_to_be_processed_set.len();

                            println!("processing_set_len {processing_set_len} waiting_to_pro_set_len {waiting_to_pro_set_len} parental_tree {:#?}", thread_pool_lists_clone.lock().unwrap().parental_tree);

                            num_threads_running_clone.fetch_add(-1, Ordering::Acquire);
                            signal_clone.wait();
                            num_threads_running_clone.fetch_add(1, Ordering::Release);
                        }
                    }
                })
            );
        }

        while thread_pool.num_threads_running.load(Ordering::Relaxed) > 0 {
            thread::sleep(Duration::from_nanos(1));
        }

        thread_pool
    }

    fn update_propagate_signal(&mut self, propagate_signal: bool) {
        self.propagate_signal.store(
            propagate_signal,
            Ordering::Relaxed,
        );
    }

    fn update_parent_gate(
        thread_pool_lists: &mut ThreadPoolLists,
        num_new_children: usize,
        num_completed_children: usize,
        parent_id: &UniqueID,
        mut passed_multiple_valid_input_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        if parent_id.id() == 0 {
            return;
        }

        println!("parent_id {} processing_set {:#?}", parent_id.id(), thread_pool_lists.processing_set);

        let parent_processing_gate = thread_pool_lists
            .processing_set
            .get_mut(parent_id)
            .expect("The parent gate should always exist when the child gate is \
                                         still in the Running status.");

        let (parent_outstanding_children,
            parent_gate,
            multiple_valid_input_gates
        ) = match &mut parent_processing_gate.processing_type {
            ProcessingSizeOfGate::Large {
                outstanding_children,
                gate,
                multiple_valid_input_gates,
            } => {
                //Subtract the current gate from the number of outstanding children.
                *outstanding_children += num_new_children;
                *outstanding_children -= num_completed_children;

                multiple_valid_input_gates.append(&mut passed_multiple_valid_input_gates);

                (outstanding_children, gate, multiple_valid_input_gates)
            }
            ProcessingSizeOfGate::Small => panic!("A parent gate should always be a large gate")
        };

        println!("parent_id {} parent_outstanding_children {parent_outstanding_children}", parent_id.id());
        if *parent_outstanding_children == 0 {
            //This was the last gate that needed to be run. There are no children to push into the
            // queue.

            let parent_gate = parent_gate.clone();

            if multiple_valid_input_gates.is_empty() { //No invalid gates.

                println!("No invalid gates");

                let processing_parent = thread_pool_lists.processing_set.remove(
                    parent_id
                ).expect("The parent gate was found above and now it is not found.");

                thread_pool_lists.waiting_to_be_processed_set.insert(
                    parent_id.clone(),
                    WaitingSizeOfGate::Large {
                        outstanding_children: 0,
                        initially_added: false,
                    },
                );

                thread_pool_lists.gates.push_back(
                    ParentIdAndGate {
                        parent_id: processing_parent.parent_id.clone(),
                        gate: parent_gate,
                    }
                )
            } else { //Invalid gates were found.

                println!("Invalid gates");

                *parent_outstanding_children = 0;

                let mut gates = Vec::new();
                gates.append(multiple_valid_input_gates);

                for gate in gates.into_iter() {
                    let gate_id = gate.lock().unwrap().get_unique_id();
                    thread_pool_lists.waiting_to_be_processed_set.insert(
                        gate_id,
                        WaitingSizeOfGate::Small,
                    );
                    thread_pool_lists.gates.push_back(
                        ParentIdAndGate {
                            parent_id: *parent_id,
                            gate,
                        }
                    );
                }
            }
        }
    }

    fn internal_shutdown(
        shutdown: &mut Arc<AtomicBool>,
        condvar_wrapper: &mut Arc<CondvarWrapper>,
        thread_pool_lists: &mut Arc<Mutex<ThreadPoolLists>>,
    ) {
        let mut thread_pool_lists = thread_pool_lists.lock().unwrap();
        thread_pool_lists.clear();
        shutdown.store(true, Ordering::Release);
        condvar_wrapper.cond.notify_all();
    }

    pub fn shutdown(&mut self) {
        Self::internal_shutdown(
            &mut self.shutdown,
            &mut self.condvar_wrapper,
            &mut self.thread_pool_lists,
        );
    }

    pub fn complete_queue(
        processing_completed: &mut Arc<Mutex<bool>>,
        wait_for_completion: &mut Arc<Condvar>,
    ) {
        let mut completed = processing_completed.lock().unwrap();
        *completed = true;

        wait_for_completion.notify_all();
    }

    pub fn join(&mut self) -> bool {
        //Pause until the thread pool is completed.
        let mut guard = self.processing_completed.lock().unwrap();
        while !*guard {
            guard = self.wait_for_completion.wait(guard).unwrap();
        }

        let thread_pool_list = self.thread_pool_lists.lock().unwrap();
        if !thread_pool_list.processing_set.is_empty() {
            panic!(
                "There were gates still processing when the thread pool completed. This could \
                mean that some gates had multiple inputs.\n{:#?}", thread_pool_list.processing_set
            )
        }

        //If shutdown was called internally, this is completed.
        !self.shutdown.load(Ordering::Relaxed)
    }

    pub fn add_to_queue(
        &mut self,
        queue_elements: Vec<QueueElement>,
    ) {
        let mut thread_pool_lists_guard = self.thread_pool_lists.lock().unwrap();
        let mut signal_clone = self.condvar_wrapper.clone();
        Self::add_to_queue_internal(
            &mut thread_pool_lists_guard,
            queue_elements,
            &mut signal_clone,
        );
    }

    pub fn get_input_gate_outputs(&self) -> Vec<(String, Vec<GateOutputState>)> {
        let thread_pool_lists_guard = self.thread_pool_lists.lock().unwrap();
        thread_pool_lists_guard.input_gate_output_states.clone()
    }

    //The mutex is expected to be locked over thread_pool_lists to call this function.
    fn add_to_queue_internal(
        mut thread_pool_lists: &mut ThreadPoolLists,
        queue_elements: Vec<QueueElement>,
        condvar_wrapper: &mut Arc<CondvarWrapper>,
    ) {
        for gate_element in queue_elements.into_iter() {
            let inserted = thread_pool_lists.waiting_to_be_processed_set.insert(
                gate_element.gate_id.clone(),
                if gate_element.number_children_in_gate < NUM_CHILDREN_GATES_FOR_LARGE_GATE {
                    WaitingSizeOfGate::Small
                } else {
                    WaitingSizeOfGate::Large {
                        outstanding_children: 0,
                        initially_added: true,
                    }
                },
            );

            if let Some(_) = inserted {
                println!("add_to_queue() Already in waiting_to_be_processed_set");
                //If the gate is already in the queue, but not being processed, no need to do anything.
                continue;
            }

            let mut processing = thread_pool_lists.processing_set.get_mut(&gate_element.gate_id);

            if let Some(processing_gate) = processing.take() {
                println!("add_to_queue() Inside processing_set");
                if gate_element.number_children_in_gate < NUM_CHILDREN_GATES_FOR_LARGE_GATE { //small gate
                    processing_gate.status = ProcessingGateStatus::Cancel;
                } else {
                    processing_gate.status = ProcessingGateStatus::Redo;
                }
            } else { //Gate is not in queue.

                println!("add_to_queue() Not yet inside processing_set");

                let parental_set = thread_pool_lists
                    .parental_tree
                    .get_mut(&gate_element.parent_id)
                    .expect("Parental tree should always exist when calling a child tree.");

                parental_set.insert(gate_element.gate_id.clone());

                //waiting_to_be_processed_set was inserted to above.

                thread_pool_lists.gates.push_back(
                    ParentIdAndGate {
                        parent_id: gate_element.parent_id,
                        gate: gate_element.gate,
                    }
                );

                // println!("notify_one Called line: {}", line!());
                condvar_wrapper.cond.notify_one();

                continue;
            }

            //TODO: Do outstanding children need to be updated here?
            fn remove_children(
                thread_pool_lists: &mut ThreadPoolLists,
                gate_id: UniqueID,
            ) {
                let children = thread_pool_lists
                    .parental_tree
                    .remove(&gate_id)
                    .expect("Child ID did not exist in parental_tree.");

                for child_id in children.into_iter() {
                    let removed = thread_pool_lists.waiting_to_be_processed_set.remove(
                        &child_id
                    );

                    if let Some(_) = removed {
                        //Instead of searching through the gates Vec, it will instead skip any values inside
                        // the gates Vec that is not inside waiting_to_be_processed.
                        continue;
                    }

                    let child_processing_gate = thread_pool_lists.processing_set.get_mut(
                        &child_id
                    );

                    if let Some(child_processing_gate) = child_processing_gate {
                        child_processing_gate.status = ProcessingGateStatus::Cancel;
                    } else {
                        panic!("A child ID was found inside \"parental_id\" structure but was not in processing or waiting to be processed sets.")
                    }

                    remove_children(
                        thread_pool_lists,
                        child_id,
                    );
                }
            }

            remove_children(
                &mut thread_pool_lists,
                gate_element.gate_id.clone(),
            );

            //Do not want the current gate removed because it is going to redo. But want it to be empty.
            thread_pool_lists
                .parental_tree
                .insert(
                    gate_element.gate_id,
                    HashSet::new(),
                );

            // println!("notify_one Called line: {}", line!());
            condvar_wrapper.cond.notify_one();

            //No need to add the gate, it is currently processing. This means that it will 'Redo'
            // itself.
        }
    }
}

pub fn start_clock<F>(
    input_gates: &Vec<SharedMutex<dyn LogicGate>>,
    output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    mut handle_output: F,
) where
    F: FnMut(&Vec<(String, Vec<GateOutputState>)>, &Vec<SharedMutex<dyn LogicGateAndOutputGate>>)
{
    assert!(!input_gates.is_empty());
    assert!(!output_gates.is_empty());

    let mut propagate_signal_through_circuit = true;
    let mut continue_clock = true;

    while continue_clock {
        //This should be the ONLY place this is ever updated.
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        continue_clock = run_circuit(
            input_gates,
            output_gates,
            propagate_signal_through_circuit,
            &mut handle_output,
        );

        propagate_signal_through_circuit = false;
    }
}

pub fn run_circuit_multi_thread<F>(
    input_gates: &Vec<SharedMutex<dyn LogicGate>>,
    output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    thread_pool: &mut RunCircuitThreadPool,
    propagate_signal_through_circuit: bool,
    handle_output: &mut F,
) -> bool
    where
        F: FnMut(&Vec<(String, Vec<GateOutputState>)>, &Vec<SharedMutex<dyn LogicGateAndOutputGate>>)
{
    // let mut thread_pool = RunCircuitThreadPool::new(num_cpus::get());

    let mut queue_gates = Vec::new();
    for input_gate in input_gates.iter() {
        let gate_id = input_gate.lock().unwrap().get_unique_id();
        let number_children_in_gate = input_gate.lock().unwrap().num_children_gates();

        queue_gates.push(
            QueueElement {
                gate: input_gate.clone(),
                parent_id: UniqueID::zero_id(),
                gate_id,
                number_children_in_gate,
            }
        );
    }

    thread_pool.update_propagate_signal(
        propagate_signal_through_circuit
    );

    thread_pool.add_to_queue(
        queue_gates
    );

    let completed = thread_pool.join();

    let input_gate_outputs = thread_pool.get_input_gate_outputs();

    handle_output(
        &input_gate_outputs,
        &output_gates,
    );

    completed
}

//TODO: Will need to rename this to spread the multi-thread to all of the different places it should be used.
//Returns true if the circuit has input remaining, false if it does not.
//Note that elements must be ordered so that some of the undetermined gates such as SR latches can
// have a defined starting state. Therefore, vectors are used even though they must be iterated
// through to guarantee uniqueness.
pub fn run_circuit<F>(
    input_gates: &Vec<SharedMutex<dyn LogicGate>>,
    output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>,
    propagate_signal_through_circuit: bool,
    handle_output: &mut F,
) -> bool where
    F: FnMut(&Vec<(String, Vec<GateOutputState>)>, &Vec<SharedMutex<dyn LogicGateAndOutputGate>>)
{
    let mut continue_clock = true;

    let print_output =
        if RUN_CIRCUIT_IS_HIGH_LEVEL.load(Ordering::SeqCst) {
            RUN_CIRCUIT_IS_HIGH_LEVEL.store(false, Ordering::SeqCst);
            true
        } else {
            false
        };

    // let mut unique_gates = HashSet::new();
    let mut clock_tick_inputs = Vec::new();
    let mut next_gates: Vec<SharedMutex<dyn LogicGate>> = input_gates.clone();

    if print_output {
        println!("run_circuit");
    }
    while !next_gates.is_empty() {
        if print_output {
            println!("next_gates.len() = {}", next_gates.len());
        }
        let gates = next_gates;
        next_gates = Vec::new();
        let mut next_gates_set = HashSet::new();
        let mut num_invalid_gates: usize = 0;

        for gate_cell in gates.into_iter() {
            let mut gate = gate_cell.lock().unwrap();
            // unique_gates.insert(gate.get_unique_id());

            let gate_output = gate.fetch_output_signals_calculate();

            let gate_output = if let Err(err) = gate_output {
                match err {
                    GateLogicError::NoMoreAutomaticInputsRemaining => {
                        return false;
                    }
                    GateLogicError::MultipleValidSignalsWhenCalculating => {
                        num_invalid_gates += 1;
                        drop(gate);
                        next_gates.push(gate_cell);
                        continue;
                    }
                };
            } else {
                gate_output.unwrap()
            };

            if gate.is_input_gate() {
                if print_output {
                    println!("input_gate_called: {}", gate.get_tag());
                }
                clock_tick_inputs.push(
                    (gate.get_tag(), gate_output.clone())
                );
            }

            if print_output {
                println!("gate_output.len(): {:?}", gate_output.len());
            }

            drop(gate);
            for output in gate_output.into_iter() {
                match output {
                    GateOutputState::NotConnected(signal) => {
                        if print_output {
                            println!("NOT_CONNECTED gate_tag {}", gate_cell.lock().unwrap().get_tag());
                        }

                        if gate_cell.lock().unwrap().get_tag() == END_OUTPUT_GATE_TAG
                            && signal == HIGH {
                            println!("End of program reached on clock-tick {}. Stopping execution.", get_clock_tick_number());
                            continue_clock = false;
                        }
                        if print_output {
                            println!("NotConnected(gate_output): {:?}", signal);
                        }
                    }
                    GateOutputState::Connected(next_gate_info) => {
                        if print_output {
                            println!("Connected(gate_output): {:?}", next_gate_info);
                        }

                        let next_gate = next_gate_info.gate.clone();
                        let mut mutable_next_gate = next_gate.lock().unwrap();

                        let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                            mutable_next_gate.update_input_signal(next_gate_info.throughput.clone());
                        let gate_id = mutable_next_gate.get_unique_id();

                        let contains_id = next_gates_set.contains(&gate_id);

                        let should_update_gate = check_if_next_gate_should_be_stored(
                            input_signal_updated,
                            changed_count_this_tick,
                            contains_id,
                            propagate_signal_through_circuit,
                        );

                        if print_output {
                            println!("checking gate {} tag {} signal {:?}", mutable_next_gate.get_gate_type(), mutable_next_gate.get_tag(), next_gate_info.throughput.signal.clone());
                            // println!("input_signal_updated: {} contains_key(): {:#?} changed_count_this_tick: {:?}", input_signal_updated, next_gates.contains_key(&gate_id), changed_count_this_tick);
                            // println!("input_signal_updated: {input_signal_updated} propagate_signal_through_circuit: {propagate_signal_through_circuit} changed_count_this_tick {changed_count_this_tick} contains_id {contains_id}");
                        }

                        if should_update_gate {
                            if print_output {
                                println!("Pushing gate {} tag {}", mutable_next_gate.get_gate_type(), mutable_next_gate.get_tag());
                            }
                            drop(mutable_next_gate);
                            // println!("next_gates.insert()");
                            next_gates_set.insert(gate_id);
                            next_gates.push(next_gate);
                        }
                    }
                }
            }
        }

        //This is set up to handle invalid states. If all gates are in an invalid state the app will
        // panic. See calculate_input_signal_from_single_inputs() in foundations.rs for more
        // details.
        if num_invalid_gates > 0 && num_invalid_gates == next_gates.len() {
            let mut gates = Vec::new();
            for gate in next_gates {
                let mut_gate = gate.lock().unwrap();
                gates.push(
                    format!("Gate {} id {} with tag {}.", mut_gate.get_gate_type(), mut_gate.get_unique_id().id(), mut_gate.get_tag())
                );
            }
            panic!("All gates inside the circuit have returned invalid input, aborting.\nInvalid Gate List\n{:#?}", gates);
        }
    }

    handle_output(
        &clock_tick_inputs,
        &output_gates,
    );

    continue_clock
}

fn check_if_next_gate_should_be_stored(
    input_signal_updated: bool,
    changed_count_this_tick: usize,
    contains_id: bool,
    propagate_signal: bool,
) -> bool {

    //It is important to remember that a situation such as an OR gate feeding
    // back into itself is perfectly valid. This can be interpreted that if the
    // input was not changed, the output was not changed either and so nothing
    // needs to be done with this gate.
    //The first tick is a bit special, because the circuit needs to propagate
    // the signal regardless of if the gates change or not. This leads to
    // checking if it is the first time the gate is updated on the first
    // clock tick.
    //Also each gate only needs to be stored inside the map once. All changed
    // inputs are saved as part of the state, so collect_output() only needs
    // to run once.
    input_signal_updated || (propagate_signal && changed_count_this_tick == 1) && !contains_id
}

pub fn count_gates_in_circuit(
    input_gates: &Vec<SharedMutex<dyn LogicGate>>,
) -> usize {
    let mut unique_gates = HashSet::new();
    let mut next_gates: Vec<SharedMutex<dyn LogicGate>> = input_gates.clone();

    while !next_gates.is_empty() {
        let gates = next_gates;
        next_gates = Vec::new();
        let mut next_gates_set = HashSet::new();
        let mut num_invalid_gates: usize = 0;

        for gate_cell in gates.into_iter() {
            let mut gate = gate_cell.lock().unwrap();
            unique_gates.insert(gate.get_unique_id());

            let gate_output = gate.fetch_output_signals_calculate();

            let gate_output = if let Err(err) = gate_output {
                match err {
                    GateLogicError::NoMoreAutomaticInputsRemaining => {
                        panic!("AutomaticInput should not be used with count_gates_in_circuit().")
                    }
                    GateLogicError::MultipleValidSignalsWhenCalculating => {
                        num_invalid_gates += 1;
                        drop(gate);
                        next_gates.push(gate_cell);
                        continue;
                    }
                };
            } else {
                gate_output.unwrap()
            };

            drop(gate);
            for output in gate_output.into_iter() {
                match output {
                    GateOutputState::NotConnected(_signal) => {}
                    GateOutputState::Connected(next_gate_info) => {
                        let next_gate = Arc::clone(&next_gate_info.gate);

                        let gate_id = next_gate.lock().unwrap().get_unique_id();

                        let inserted = unique_gates.insert(gate_id.clone());

                        if inserted {
                            next_gates_set.insert(gate_id);
                            next_gates.push(next_gate);
                        }
                    }
                }
            }
        }

        //This is set up to handle invalid states. If all gates are in an invalid state the app will
        // panic. See calculate_input_signal_from_single_inputs() in foundations.rs for more
        // details.
        if num_invalid_gates > 0 && num_invalid_gates == next_gates.len() {
            let mut gates = Vec::new();
            for gate in next_gates {
                let mut_gate = gate.lock().unwrap();
                gates.push(
                    format!("Gate {} id {} with tag {}.", mut_gate.get_gate_type(), mut_gate.get_unique_id().id(), mut_gate.get_tag())
                );
            }
            panic!("All gates inside the circuit have returned invalid input, aborting.\nInvalid Gate List\n{:#?}", gates);
        }
    }

    unique_gates.len()
}

pub fn generate_default_output(cpu: &SharedMutex<VariableBitCPU>) -> Vec<Signal> {

    // Multi-bit outputs
    // VariableBitCPU::R0
    // VariableBitCPU::R1
    // VariableBitCPU::R2
    // VariableBitCPU::R3
    // VariableBitCPU::IR
    // VariableBitCPU::IAR
    // VariableBitCPU::ACC
    // VariableBitCPU::TMP
    // VariableBitCPU::BUS
    // RAM_registers (no constant)
    //
    // Single-bit outputs
    // VariableBitCPU::CLK
    // VariableBitCPU::CLKE
    // VariableBitCPU::CLKS
    // VariableBitCPU::IO
    // VariableBitCPU::DA
    // VariableBitCPU::END
    // VariableBitCPU::IO_CLK_E
    // VariableBitCPU::IO_CLK_S

    let mut generated_signals = vec![LOW_; cpu.lock().unwrap().get_complex_gate().output_gates.len()];
    let clke_index = cpu.lock().unwrap().get_complex_gate().gate_tags_to_index[VariableBitCPU::CLKE].index;
    generated_signals[clke_index] = HIGH;
    generated_signals
}

pub fn convert_binary_to_inputs_for_load(
    binary_strings: Vec<&str>,
    num_ram_cells: usize,
) -> Vec<SharedMutex<AutomaticInput>> {
    assert_ne!(binary_strings.len(), 0);
    assert!(binary_strings.len() <= num_ram_cells);

    let mut ram_inputs = vec![vec![]; binary_strings.first().unwrap().len()];
    for (i, string) in binary_strings.iter().enumerate() {
        for (j, c) in string.chars().rev().enumerate() {
            let signal =
                if c == '0' {
                    LOW_
                } else {
                    HIGH
                };

            let num_pushes =
                if i != 0 {
                    4
                } else {
                    2
                };

            for _ in 0..num_pushes {
                ram_inputs[j].push(signal.clone());
            }
        }
    }

    //The vector is filled up so that it runs for each ram cell. Then there are two extra inputs
    // needed to put the clock from the end of LOAD to the starting clock state.
    let num_extra_inputs = (num_ram_cells - binary_strings.len()) * 4 + 2;
    for i in 0..ram_inputs.len() {
        for _ in 0..num_extra_inputs {
            ram_inputs[i].push(LOW_);
        }
    }

    let mut automatic_inputs = Vec::new();
    for (i, inp) in ram_inputs.iter().enumerate() {
        let input_tag = format!("Input_bit_{}", i);
        automatic_inputs.push(
            AutomaticInput::new(inp.clone(), 1, input_tag.as_str())
        );
    }

    automatic_inputs
}

pub fn collect_signals_from_logic_gate(
    gate: SharedMutex<dyn LogicGate>
) -> Vec<Signal> {
    let cpu_output = gate.lock().unwrap().fetch_output_signals_calculate().unwrap();
    let mut collected_signals = Vec::new();
    for out in cpu_output.into_iter() {
        match out {
            GateOutputState::NotConnected(signal) => {
                collected_signals.push(signal);
            }
            GateOutputState::Connected(connected_output) => {
                collected_signals.push(connected_output.throughput.signal);
            }
        }
    }
    collected_signals
}

pub fn run_instructions(
    number_bits: usize,
    decoder_input_size: usize,
    binary_strings: &Vec<&str>,
) -> SharedMutex<VariableBitCPU> {
    let cpu = VariableBitCPU::new(number_bits, decoder_input_size);

    let num_ram_cells = usize::pow(2, (decoder_input_size * 2) as u32);
    assert!(binary_strings.len() <= num_ram_cells);
    if !binary_strings.is_empty() {
        assert_eq!(binary_strings[0].len(), number_bits);
    }

    println!("Beginning to load values into RAM");

    let start_load = Instant::now();

    load_values_into_ram(
        &cpu,
        binary_strings,
        num_ram_cells,
    );

    let complete_load = Instant::now();

    let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
    let clock = Clock::new(1, "PRIMARY_CLOCK");
    let clk_in_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::CLK_IN);
    cpu.lock().unwrap().get_clock_synced_with_cpu(&clock);

    connect_gates(
        clock.clone(),
        0,
        cpu.clone(),
        clk_in_index,
    );

    input_gates.push(clock.clone());

    let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
    let end_output_gate = SimpleOutput::new(END_OUTPUT_GATE_TAG);

    let cpu_end_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::END);
    connect_gates(
        cpu.clone(),
        cpu_end_index,
        end_output_gate.clone(),
        0,
    );

    output_gates.push(end_output_gate.clone());

    println!("\nCompleted load in {} clock-ticks. Beginning program.\n", get_clock_tick_number());
    unsafe {
        CLOCK_TICK_NUMBER = 0;
        RAM_TIME = Duration::new(0, 0);
        CONTROL_SECTION_TIME = Duration::new(0, 0);
        ALU_TIME = Duration::new(0, 0);
    }
    let mut continue_load_operation = true;
    let mut propagate_signal = true;
    while continue_load_operation {
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }

        continue_load_operation = run_circuit(
            &input_gates,
            &output_gates,
            propagate_signal,
            &mut |_clock_tick_inputs, _output_gates| {},
        );

        propagate_signal = false;
    }

    let complete_run = Instant::now();

    let run_time = complete_run.duration_since(complete_load);
    println!("Loading took {:?}", complete_load.duration_since(start_load));
    println!("Run took {:?}", run_time);
    println!("Total took {:?}", complete_run.duration_since(start_load));
    println!(
        "CPU ran at {}Hz",
        if run_time.as_secs() == 0 {
            0
        } else {
            get_clock_tick_number() as u64 / complete_run.duration_since(complete_load).as_secs()
        }
    );

    cpu
}

//This should leave the cpu in the same state as it started in. The only difference is that
// there will now be values loaded into RAM. It should be run without any inputs connected to
// the cpu itself.
pub fn load_values_into_ram(
    cpu: &SharedMutex<VariableBitCPU>,
    binary_strings: &Vec<&str>,
    num_ram_cells: usize,
) {
    let automatic_inputs = convert_binary_to_inputs_for_load(
        binary_strings.clone(),
        num_ram_cells,
    );

    let num_cycles = num_ram_cells * 4 - 2;

    //The last cycle is to advance the clock to the starting position. AND to get the splitter
    // to the correct position.
    let output_values = vec![HIGH; num_cycles + 1];

    let load_automatic_input = AutomaticInput::new(
        output_values.clone(),
        1,
        "LOAD",
    );

    let memory_address_register_automatic_input = AutomaticInput::new(
        output_values,
        1,
        "MEMORY_ADDRESS_REGISTER",
    );

    let load_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::LOAD);
    connect_gates(
        load_automatic_input.clone(),
        0,
        cpu.clone(),
        load_index,
    );

    let memory_address_register_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::MARS);
    connect_gates(
        memory_address_register_automatic_input.clone(),
        0,
        cpu.clone(),
        memory_address_register_index,
    );

    let mut automatic_input_gates: Vec<SharedMutex<AutomaticInput>> = Vec::new();
    let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
    let clock = Clock::new(1, "PRIMARY_CLOCK");
    cpu.lock().unwrap().get_clock_synced_with_cpu(&clock);

    let clk_in_index = cpu.lock().unwrap().get_index_from_tag(VariableBitCPU::CLK_IN);
    connect_gates(
        clock.clone(),
        0,
        cpu.clone(),
        clk_in_index,
    );

    input_gates.push(clock.clone());
    input_gates.push(load_automatic_input.clone());
    input_gates.push(memory_address_register_automatic_input.clone());
    automatic_input_gates.push(load_automatic_input);
    automatic_input_gates.push(memory_address_register_automatic_input);

    for (i, input) in automatic_inputs.iter().enumerate() {
        let ram_input_tag = format!("{}_{}", VariableBitCPU::RAM, i);
        let ram_input_index = cpu.lock().unwrap().get_index_from_tag(ram_input_tag.as_str());
        connect_gates(
            input.clone(),
            0,
            cpu.clone(),
            ram_input_index,
        );
        input_gates.push(input.clone());
        automatic_input_gates.push(input.clone());
    }

    let mut continue_load_operation = true;
    let mut propagate_signal = true;
    while continue_load_operation {
        unsafe {
            CLOCK_TICK_NUMBER += 1;
        }
        // println!("CLOCK TICK {}", get_clock_tick_number());

        continue_load_operation = run_circuit(
            &input_gates,
            &Vec::new(),
            propagate_signal,
            &mut |_clock_tick_inputs, _output_gates| {},
        );

        propagate_signal = false;
    }

    //Disconnect all inputs so that future connections can be made.
    for automatic_input_gate in automatic_input_gates.into_iter() {
        automatic_input_gate.lock().unwrap().disconnect_gate(0);
    }

    clock.lock().unwrap().disconnect_gate(0);

    //LOAD and MAR_S must be tied back to LOW before completing. They have already been
    // disconnected so the zero id is used.
    cpu.lock().unwrap().update_input_signal(
        GateInput::new(
            load_index,
            LOW_,
            UniqueID::zero_id(),
        )
    );

    cpu.lock().unwrap().update_input_signal(
        GateInput::new(
            memory_address_register_index,
            LOW_,
            UniqueID::zero_id(),
        )
    );

    let mut generated_output = generate_default_output(&cpu);

    for (i, binary_string) in binary_strings.iter().enumerate() {
        for (j, c) in binary_string.chars().rev().enumerate() {
            let output_tag = RAMUnit::get_ram_output_string(i, j);
            let output_index = cpu.lock().unwrap().get_complex_gate().gate_tags_to_index[&output_tag.to_string()].index;

            let signal =
                if c == '0' {
                    LOW_
                } else {
                    HIGH
                };

            generated_output[output_index] = signal.clone();
        }
    }

    let collected_signals = collect_signals_from_logic_gate(cpu.clone());

    let failed = compare_generate_and_collected_output(&cpu, generated_output, collected_signals);

    assert!(!failed);
}

pub fn compare_generate_and_collected_output(
    cpu: &SharedMutex<VariableBitCPU>,
    generated_output: Vec<Signal>,
    collected_signals: Vec<Signal>,
) -> bool {
    let tags_sorted_by_index = extract_output_tags_sorted_by_index(&cpu.lock().unwrap().get_complex_gate());

    assert_eq!(collected_signals.len(), generated_output.len());
    assert_eq!(collected_signals.len(), tags_sorted_by_index.len());

    let mut failed = false;
    for i in 0..collected_signals.len() {
        let mut failed_map = HashMap::new();

        if (tags_sorted_by_index[i].clone(), generated_output[i].clone()) != (tags_sorted_by_index[i].clone(), collected_signals[i].clone()) {
            failed_map.insert(tags_sorted_by_index[i].clone(), (generated_output[i].clone(), collected_signals[i].clone()));
            failed = true;
        };

        if !failed_map.is_empty() {
            println!("E (passed, collected): {:?}", failed_map);
        }
    }
    failed
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::logic::basic_gates::{And, Not, Or};
    use crate::logic::foundations::Signal::{HIGH, LOW_};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::memory_gates::{OneBitMemoryCell, VariableBitMemoryCell};
    use crate::logic::output_gates::SimpleOutput;
    use crate::run_circuit::run_circuit;
    use crate::test_stuff::{check_for_single_element_signal, run_test_with_timeout};
    use super::*;

    #[test]
    fn minimum_system() {
        let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
        let output_gate = SimpleOutput::new("");

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        connect_gates(
            input_gate.clone(),
            0,
            output_gate.clone(),
            0,
        );

        run_circuit(
            &input_gates,
            &output_gates,
            false,
            &mut |_clock_tick_inputs, output_gates| {
                check_for_single_element_signal(output_gates, HIGH);
            },
        );
    }

    #[test]
    #[should_panic]
    fn test_oscillation() {
        run_test_with_timeout(
            Duration::from_millis(500),
            || {
                let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
                let output_gate = SimpleOutput::new("");
                let not_gate = Not::new(2);

                let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
                let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

                input_gates.push(input_gate.clone());
                output_gates.push(output_gate.clone());

                connect_gates(
                    input_gate.clone(),
                    0,
                    not_gate.clone(),
                    0,
                );

                connect_gates(
                    not_gate.clone(),
                    0,
                    output_gate.clone(),
                    0,
                );

                //Create a loop.
                connect_gates(
                    not_gate.clone(),
                    1,
                    not_gate.clone(),
                    0,
                );

                run_circuit(
                    &input_gates,
                    &output_gates,
                    false,
                    &mut |_clock_tick_inputs, _output_gates| {
                        //An oscillation should panic! before it ever reaches this point. Cannot use the
                        // panic! macro because the test will not be able to check if it failed properly or
                        // not.
                        assert!(false);
                    },
                );
            },
        );
    }

    #[test]
    fn test_simple_loop() {
        run_test_with_timeout(
            Duration::from_millis(500),
            || {
                let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
                let output_gate = SimpleOutput::new("");
                let or_gate = Or::new(2, 2);

                let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
                let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

                input_gates.push(input_gate.clone());
                output_gates.push(output_gate.clone());

                connect_gates(
                    input_gate.clone(),
                    0,
                    or_gate.clone(),
                    0,
                );

                connect_gates(
                    or_gate.clone(),
                    0,
                    output_gate.clone(),
                    0,
                );

                //Create a loop.
                connect_gates(
                    or_gate.clone(),
                    1,
                    or_gate.clone(),
                    1,
                );

                run_circuit(
                    &input_gates,
                    &output_gates,
                    false,
                    &mut |_clock_tick_inputs, output_gates| {
                        check_for_single_element_signal(output_gates, HIGH);
                    },
                );
            },
        );
    }

    //Because this `not` gate has the default input value, its initial state will be set to LOW and
    // not be change under normal circumstances. However, the first clock tick everything must
    // propagate through the system to properly set the outputs. This means that the final output
    // should be changed to HIGH.
    #[test]
    fn first_tick_propagates() {
        let input_gate = AutomaticInput::new(vec![LOW_], 1, "");
        let output_gate = SimpleOutput::new("");
        let not_gate = Not::new(1);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        connect_gates(
            input_gate.clone(),
            0,
            not_gate.clone(),
            0,
        );

        connect_gates(
            not_gate.clone(),
            0,
            output_gate.clone(),
            0,
        );

        start_clock(
            &input_gates,
            &output_gates,
            &mut |_: &Vec<(String, Vec<GateOutputState>)>, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
                check_for_single_element_signal(output_gates, HIGH);
            },
        );
    }

    #[test]
    fn multiple_ticks() {
        let input_gate = AutomaticInput::new(vec![LOW_, HIGH, HIGH], 1, "");
        let output_gate = SimpleOutput::new("");
        let not_gate = Not::new(1);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        connect_gates(
            input_gate.clone(),
            0,
            not_gate.clone(),
            0,
        );

        connect_gates(
            not_gate.clone(),
            0,
            output_gate.clone(),
            0,
        );

        let expected_outputs = vec![HIGH, LOW_, LOW_];
        let mut current_index = 0;

        start_clock(
            &input_gates,
            &output_gates,
            &mut |_: &Vec<(String, Vec<GateOutputState>)>, output_gates: &Vec<SharedMutex<dyn LogicGateAndOutputGate>>| {
                assert!(current_index < expected_outputs.len());
                assert_eq!(output_gates.len(), 1);

                let value = output_gates.into_iter().next().unwrap();
                let mut value = value.lock().unwrap();
                let output_signals = value.fetch_output_signals_calculate().unwrap();

                assert_eq!(output_signals.len(), 1);

                let gate_output_state = output_signals.first().unwrap();

                match gate_output_state {
                    GateOutputState::NotConnected(signal) => {
                        if let Some(output) = expected_outputs.get(current_index) {
                            assert_eq!(*signal, *output)
                        } else {
                            panic!("The number of outputs exceeded the maximum number.");
                        }
                    }
                    GateOutputState::Connected(_) => {
                        panic!("The output gate should never be connected.");
                    }
                }

                current_index += 1;
            },
        );

        assert_eq!(current_index, expected_outputs.len());
    }

    #[test]
    fn minimum_system_multi_thread() {
        let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
        let output_gate = SimpleOutput::new("");

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        connect_gates(
            input_gate.clone(),
            0,
            output_gate.clone(),
            0,
        );

        let mut thread_pool = RunCircuitThreadPool::new(num_cpus::get() - 1);

        let completed = run_circuit_multi_thread(
            &input_gates,
            &output_gates,
            &mut thread_pool,
            false,
            &mut |_clock_tick_inputs, output_gates| {
                check_for_single_element_signal(output_gates, HIGH);
            },
        );

        assert!(completed);
    }

    #[test]
    fn single_small_gate_multi_thread() {
        let input_gate_a = AutomaticInput::new(vec![HIGH], 1, "");
        let input_gate_b = AutomaticInput::new(vec![HIGH], 1, "");
        let output_gate = SimpleOutput::new("");
        let and_gate = And::new(2, 1);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(input_gate_a.clone());
        input_gates.push(input_gate_b.clone());
        output_gates.push(output_gate.clone());

        connect_gates(
            input_gate_a.clone(),
            0,
            and_gate.clone(),
            0,
        );

        connect_gates(
            input_gate_b.clone(),
            0,
            and_gate.clone(),
            1,
        );

        connect_gates(
            and_gate.clone(),
            0,
            output_gate.clone(),
            0,
        );

        let mut thread_pool = RunCircuitThreadPool::new(num_cpus::get() - 1);

        let completed = run_circuit_multi_thread(
            &input_gates,
            &output_gates,
            &mut thread_pool,
            false,
            &mut |_clock_tick_inputs, output_gates| {
                check_for_single_element_signal(output_gates, HIGH);
            },
        );

        assert!(completed);
    }

    #[test]
    fn single_large_gate_multi_thread() {
        //TODO: Run this test over and over, sometimes it gets stuck.

        //TODO: Extract this to a function. create_large_gate() or something, maybe hook up the input
        // and output as well.
        // let mut number_bits = 1;
        // let mut variable_bit_mem_cell = VariableBitMemoryCell::new(number_bits);
        // let mut number_children = variable_bit_mem_cell.lock().unwrap().num_children_gates();
        // while number_children < NUM_CHILDREN_GATES_FOR_LARGE_GATE {
        //     number_bits += 1;
        //     variable_bit_mem_cell = VariableBitMemoryCell::new(number_bits);
        //     number_children = variable_bit_mem_cell.lock().unwrap().num_children_gates();
        // }
        //
        // let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        // let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();
        //
        // let input_gate = AutomaticInput::new(vec![HIGH], 1, "S");
        // input_gates.push(input_gate.clone());
        //
        // let input_index = variable_bit_mem_cell.lock().unwrap().get_index_from_tag(
        //     "S"
        // );
        //
        // connect_gates(
        //     input_gate.clone(),
        //     0,
        //     variable_bit_mem_cell.clone(),
        //     input_index,
        // );
        //
        // for i in 0..number_bits {
        //     let input_tag = format!("i_{}", i);
        //     let input_gate = AutomaticInput::new(
        //         vec![HIGH], 1, input_tag.as_str(),
        //     );
        //     input_gates.push(input_gate.clone());
        //
        //     let input_index = variable_bit_mem_cell.lock().unwrap().get_index_from_tag(
        //         input_tag.as_str()
        //     );
        //
        //     connect_gates(
        //         input_gate.clone(),
        //         0,
        //         variable_bit_mem_cell.clone(),
        //         input_index,
        //     );
        //
        //     let output_tag = format!("o_{}", i);
        //     let output_gate = SimpleOutput::new(output_tag.as_str());
        //     output_gates.push(output_gate.clone());
        //
        //     let output_index = variable_bit_mem_cell.lock().unwrap().get_index_from_tag(
        //         output_tag.as_str()
        //     );
        //
        //     connect_gates(
        //         variable_bit_mem_cell.clone(),
        //         output_index,
        //         output_gate.clone(),
        //         0,
        //     );
        // }

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        let one_bit_memory_cell = OneBitMemoryCell::new(1);

        let set_input_gate = AutomaticInput::new(vec![HIGH], 1, "S");
        input_gates.push(set_input_gate.clone());

        let input_index = one_bit_memory_cell.lock().unwrap().get_index_from_tag(
            "S"
        );

        connect_gates(
            set_input_gate.clone(),
            0,
            one_bit_memory_cell.clone(),
            input_index,
        );

        let enable_input_gate = AutomaticInput::new(vec![HIGH], 1, "E");
        input_gates.push(enable_input_gate.clone());

        let input_index = one_bit_memory_cell.lock().unwrap().get_index_from_tag(
            "E"
        );

        connect_gates(
            enable_input_gate.clone(),
            0,
            one_bit_memory_cell.clone(),
            input_index,
        );

        let output_gate = SimpleOutput::new("o");
        output_gates.push(output_gate.clone());

        connect_gates(
            one_bit_memory_cell.clone(),
            0,
            output_gate.clone(),
            0,
        );

        println!("one_bit_memory_cell id {}", one_bit_memory_cell.lock().unwrap().get_unique_id().id());
        println!("one_bit_memory_cell num_children_gates {}", one_bit_memory_cell.lock().unwrap().num_children_gates());

        let mut thread_pool = RunCircuitThreadPool::new(1); //todo num_cpus::get() - 1);

        let completed = run_circuit_multi_thread(
            &input_gates,
            &output_gates,
            &mut thread_pool,
            true,
            &mut |_clock_tick_inputs, output_gates| {
                for output_gate in output_gates {
                    let mut output_gate = output_gates.first().unwrap().lock().unwrap();
                    let output_signals = output_gate.fetch_output_signals_calculate().unwrap();

                    assert_eq!(output_signals.len(), 1);

                    let gate_output_state = output_signals.first().unwrap();

                    //TODO: All of the inputs including the set input are HIGH, this means that the
                    // outputs should all be HIGH as well. So in other words something is wrong.
                    println!("line {}", line!());
                    match gate_output_state {
                        GateOutputState::NotConnected(signal) => {
                            println!("tag {} signal {:?}", output_gate.get_tag(), *signal);
                            assert_eq!(*signal, HIGH)
                        }
                        GateOutputState::Connected(_) => {
                            panic!("The output gate should never be connected.");
                        }
                    }
                }
            },
        );

        assert!(completed);
    }

    //TODO: tests
    // small and large gate

    //TODO: finish this
    // #[test]
    // #[should_panic]
    fn oscillation_multi_thread() {
        let input_gate = AutomaticInput::new(vec![HIGH], 1, "");
        let output_gate = SimpleOutput::new("");
        let not_gate = Not::new(2);

        let mut input_gates: Vec<SharedMutex<dyn LogicGate>> = Vec::new();
        let mut output_gates: Vec<SharedMutex<dyn LogicGateAndOutputGate>> = Vec::new();

        input_gates.push(input_gate.clone());
        output_gates.push(output_gate.clone());

        connect_gates(
            input_gate.clone(),
            0,
            not_gate.clone(),
            0,
        );

        connect_gates(
            not_gate.clone(),
            0,
            output_gate.clone(),
            0,
        );

        //Create a loop.
        connect_gates(
            not_gate.clone(),
            1,
            not_gate.clone(),
            0,
        );

        let mut thread_pool = RunCircuitThreadPool::new(num_cpus::get() - 1);

        let completed = run_circuit_multi_thread(
            &input_gates,
            &output_gates,
            &mut thread_pool,
            false,
            &mut |_clock_tick_inputs, output_gates| {
                check_for_single_element_signal(output_gates, HIGH);
            },
        );

        assert!(completed);
    }

    //TODO: tests
    // simple loop
    // first tick propagates
    // multiple ticks
    // GateLogicError::NoMoreAutomaticInputsRemaining works
    // END_OUTPUT_GATE_TAG
}
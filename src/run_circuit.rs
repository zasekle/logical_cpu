use std::collections::{HashMap, HashSet, VecDeque};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread;
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

static NUM_CHILDREN_GATES_FOR_LARGE_GATE: usize = 7;

struct CondvarWrapper {
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

#[derive(Clone)]
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

enum WaitingSizeOfGate {
    Large {
        outstanding_children: usize,
    },
    Small,
}

impl WaitingSizeOfGate {
    fn convert_from_processing(size: &ProcessingSizeOfGate) -> Self {
        match size {
            ProcessingSizeOfGate::Large { outstanding_children, .. } => Self::Large {
                outstanding_children: *outstanding_children,
            },
            ProcessingSizeOfGate::Small => Self::Small
        }
    }
}

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
}

impl ThreadPoolLists {
    pub fn clear(&mut self) {
        self.parental_tree.clear();
        self.processing_set.clear();
        self.waiting_to_be_processed_set.clear();
        self.gates.clear();
    }
}

pub struct RunCircuitThreadPool {
    thread_pool_lists: Arc<Mutex<ThreadPoolLists>>,

    threads: Vec<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    condvar_wrapper: Arc<CondvarWrapper>,
}

//TODO: For now the goal here is to get a working interface. Performance can be improved upon later.
impl RunCircuitThreadPool {

    //TODO: Steps
    // Need to check if this gate should run in a single-threaded or multi-threaded way.
    // If single-threaded, run it (should be cancellable by checking the status of its parent)
    // If multi-threaded, store it in a separate Vec and run its children gates
    //  This needs to include a status (for CANCELLED or ACTIVE)
    //  When its outstanding children gates hit 0, it will need to be reloaded and be
    //   waiting_to_be_processed (maybe something different for ACTIVE and CANCELLED)

    //TODO: So I need
    // Some way to get/measure the number of outstanding gates.

    //TODO: Will there be a problem with the edge cases that I has to deal with in run_circuit?
    pub fn new(size: NonZeroUsize) -> Self {
        let mut thread_pool = RunCircuitThreadPool {
            thread_pool_lists: Arc::new(
                Mutex::new(
                    ThreadPoolLists {
                        parental_tree: HashMap::new(),
                        processing_set: HashMap::new(),
                        waiting_to_be_processed_set: HashMap::new(),
                        gates: VecDeque::new(),
                    }
                )
            ),
            threads: Vec::new(),
            shutdown: Arc::new(AtomicBool::from(false)),
            condvar_wrapper: Arc::new(CondvarWrapper::new()),
        };

        for i in 0..size.into() {
            let mut shutdown_clone = thread_pool.shutdown.clone();
            let mut thread_pool_lists_clone = thread_pool.thread_pool_lists.clone();
            let mut signal_clone = thread_pool.condvar_wrapper.clone();
            thread_pool.threads.push(
                thread::spawn(move || {
                    println!("Thread {i} started");

                    loop {
                        if shutdown_clone.load(Ordering::Acquire) {
                            println!("Thread {i} shutting down");
                            break;
                        }

                        let popped_element;
                        loop {
                            //The lock will be held as long as the MutexGuard is alive. So I
                            // need to create a scope to make sure the lock is not held for the
                            // duration of the task being run.
                            let mut thread_pool_lists = thread_pool_lists_clone.lock().unwrap();

                            let front_gate = thread_pool_lists.gates.pop_front();

                            if let Some(gate) = &front_gate {
                                let gate_id = gate.gate.lock().unwrap().get_unique_id().clone();
                                let gate_num_children = gate.gate.lock().unwrap().num_children_gates();

                                let waiting_to_be_processed = thread_pool_lists.waiting_to_be_processed_set.remove(
                                    &gate_id
                                );

                                if let Some(waiting_element) = waiting_to_be_processed {
                                    if let WaitingSizeOfGate::Large { outstanding_children } = waiting_element
                                    {
                                        if outstanding_children == 0 {
                                            let parent_id = &gate.parent_id;

                                            if parent_id.id() == 0 {
                                                //TODO: Do something if no parent, maybe continue?
                                            }

                                            Self::update_parent_gate(
                                                &mut thread_pool_lists,
                                                0,
                                                &gate_id,
                                                parent_id,
                                                Vec::new(),
                                            );

                                            continue;
                                        }
                                    }
                                } else {
                                    //If the gate was canceled, then it will not exist inside the
                                    // waiting_to_be_processed_set.
                                    continue;
                                }

                                let processing_type =
                                    if gate_num_children < NUM_CHILDREN_GATES_FOR_LARGE_GATE {
                                        ProcessingSizeOfGate::Small
                                    } else {
                                        ProcessingSizeOfGate::Large {
                                            outstanding_children: 0, //TODO: set this somewhere
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

                            let mut next_gates = Vec::new();
                            let mut multiple_valid_signals = Vec::new();
                            let mut num_gates_added = 0;
                            if element_num_children < NUM_CHILDREN_GATES_FOR_LARGE_GATE {
                                let fetched_signals =
                                    running_gate.gate.lock().unwrap().fetch_output_signals();

                                match fetched_signals {
                                    Ok(output_states) => {
                                        for gate_output_state in output_states {
                                            match gate_output_state {
                                                GateOutputState::NotConnected(_) => {}
                                                GateOutputState::Connected(connected_gate) => {
                                                    //TODO: need to increment num_gates_added
                                                    //TODO: Need to save the gates to add them to the queue below, use the
                                                    // same credentials as run_circuit. How do I prevent duplicates? There
                                                    // are duplicate at two levels to worry about. First is inside the gates
                                                    // queue (add_queue function maybe), second is duplicates that are
                                                    // returned from fetch itself (is this even possible).
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
                            } else {
                                //TODO: Large gate, extract input gates and add em on.

                                //TODO: change num_gates_added;
                            }

                            let mut thread_pool_lists_guard = thread_pool_lists_clone.lock().unwrap();

                            let gate_id = running_gate.gate.lock().unwrap().get_unique_id();

                            let thread_pool_lists: &mut ThreadPoolLists = &mut thread_pool_lists_guard;

                            let processing_element = thread_pool_lists.processing_set.get(
                                &gate_id
                            ).expect("A gate was removed from the processing_set while it was running.");

                            match processing_element.status {
                                ProcessingGateStatus::Running => {
                                    thread_pool_lists.parental_tree.remove(
                                        &gate_id
                                    );

                                    let parent_id = &running_gate.parent_id;

                                    //TODO: Remember that the highest level gate will have a parent_id of ZERO
                                    if parent_id.id() != 0 {
                                        //TODO: what to do here?
                                    }

                                    Self::update_parent_gate(
                                        thread_pool_lists,
                                        num_gates_added,
                                        &gate_id,
                                        parent_id,
                                        multiple_valid_signals,
                                    );

                                    //TODO: Should push the next gates into the lists, assuming
                                    // something about large/small.
                                    //TODO: How is this different than the add_to_queue function
                                    // below? It kind of is because it doesn't need to check the other
                                    // stuff right? I know its Running, so therefore its inside the
                                    // processing queue.
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
                        } else {
                            println!("Thread {i} sleeping");
                            signal_clone.wait();
                        }
                    }
                })
            );
        }

        thread_pool
    }

    fn update_parent_gate(
        mut thread_pool_lists: &mut ThreadPoolLists,
        num_new_children: usize,
        gate_id: &UniqueID,
        parent_id: &UniqueID,
        mut passed_multiple_valid_input_gates: Vec<SharedMutex<dyn LogicGate>>,
    ) {
        let siblings = thread_pool_lists.parental_tree.get_mut(
            parent_id
        ).expect(
        "A sibling completed when its parent tree \
                was removed. This should never happen because the parent tree should \
                never be completed before the children tree is."
        );

        siblings.remove(&gate_id);

        let parent_processing_gate = thread_pool_lists
            .processing_set
            .get_mut(parent_id)
            .expect("The parent gate should always exist when the child gate is \
                                         still in the Running status.");

        let (parent_outstanding_children, parent_gate, multiple_valid_input_gates) =
            match &mut parent_processing_gate.processing_type {
                ProcessingSizeOfGate::Large {
                    outstanding_children,
                    gate,
                    multiple_valid_input_gates,
                } => {
                    //Subtract the current gate from the number of outstanding children.
                    *outstanding_children -= 1;
                    *outstanding_children += num_new_children;

                    multiple_valid_input_gates.append(&mut passed_multiple_valid_input_gates);

                    (outstanding_children, gate, multiple_valid_input_gates)
                }
                ProcessingSizeOfGate::Small => panic!("A parent gate should always be a large gate")
            };

        if *parent_outstanding_children == 0 {
            //This was the last gate that needed to be run. There are no children to push into the
            // queue.

            let parent_gate = parent_gate.clone();

            if multiple_valid_input_gates.is_empty() { //No invalid gates.
                thread_pool_lists.processing_set.remove(
                    parent_id
                ).expect("The parent gate was found above and now it is not found.");

                thread_pool_lists.waiting_to_be_processed_set.insert(
                    parent_id.clone(),
                    WaitingSizeOfGate::Large {
                        outstanding_children: 0
                    },
                );

                thread_pool_lists.gates.push_back(
                    ParentIdAndGate {
                        parent_id: parent_id.clone(),
                        gate: parent_gate,
                    }
                )
            } else { //Invalid gates were found.
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

    fn add_to_queue(
        &mut self,
        mut thread_pool_lists: MutexGuard<ThreadPoolLists>,
        outstanding_children: usize,
        gate: SharedMutex<dyn LogicGate>,
        parent_id: UniqueID,
        gate_id: UniqueID,
    ) {
        // let mut thread_pool_lists = self.thread_pool_lists.lock().unwrap();
        let num_children_gate = gate.lock().unwrap().num_children_gates();

        let inserted = thread_pool_lists.waiting_to_be_processed_set.insert(
            gate_id.clone(),
            if num_children_gate < NUM_CHILDREN_GATES_FOR_LARGE_GATE {
                WaitingSizeOfGate::Small
            } else {
                WaitingSizeOfGate::Large { outstanding_children }
            },
        );

        if let None = inserted {
            //If the gate is already in the queue, but not being processed, no need to do anything.
            return;
        }

        let mut processing = thread_pool_lists.processing_set.get_mut(&gate_id);

        if let Some(processing_gate) = processing.take() {
            if num_children_gate < NUM_CHILDREN_GATES_FOR_LARGE_GATE { //small gate
                processing_gate.status = ProcessingGateStatus::Cancel;
            } else {
                processing_gate.status = ProcessingGateStatus::Redo;
            }
        } else { //Gate is not in queue.
            let parental_set = thread_pool_lists
                .parental_tree
                .get_mut(&parent_id)
                .expect("Parental tree should always exist when calling a child tree.");

            parental_set.insert(gate_id.clone());

            //TODO: take care of the case where parent_id is 0

            //waiting_to_be_processed_set was inserted to above.

            thread_pool_lists.gates.push_back(
                ParentIdAndGate {
                    parent_id,
                    gate,
                }
            );

            self.condvar_wrapper.cond.notify_one();

            return;
        }

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
            gate_id.clone(),
        );

        //Do not want the current gate removed because it is going to redo. But want it to be empty.
        thread_pool_lists
            .parental_tree
            .insert(
                gate_id,
                HashSet::new(),
            );

        self.condvar_wrapper.cond.notify_one();

        //No need to add the gate, it is currently processing. This means that it will 'Redo'
        // itself.


        //TODO: Remember that a child gate will need to check and see if its parent was canceled.
        // This is an interesting point, if I get a parent gate that is 'processing' and it has
        //  outstanding children, I need to cancel all of the outstanding children right?
        // This can be done in two ways.
        //  When the parent gate is received, it can iterate through
        //  and set each child to canceled. This requires storing the processing and waiting to be
        //  processed gates that relate back to the parent gate.
        //  OR
        //  The child gate can check the status of its parent gate when it completes and cancel
        //  itself. It will need to keep a hierarchy of all of its parents so that if any of them
        //  are canceled, it will need to automatically stop. I think that when the gate is canceled
        //  it will still need a way to go through and remove any children gates from the waiting_to_be_processed
        //  and gates queue.

        //TODO: There is another problem. Say I get a large gate and I queue up all its input gates,
        // what is the flow like here? When does it go back into the waiting_to_be_processed queue?
        // I guess when the outstanding_number hits zero, but where is it stored until then?

        //Small gates
        // Actually run the gate.
        // When the gate completes, fetch will be called for each gate added to the queue, the
        //  parent number will need to be incremented.
        // When the run_circuit completes, the new gates will need to be added using add_to_queue().
        // No need to cancel a small gate (they should be fast), but when fetch_output_signals is
        //  called, if ANY parent gate of this is cancelled or set to retry, it will need to abandon
        //  adding any new gates.

        //Large gates
        // This will not actually run, It will need to increment its own outstanding_number by the
        //  number of input gates, then add each input gate to the 'gates' queue with the parent id.
        // If a retry comes for this gate

        //TODO: In order for the outstanding_number to have meaning, it will have to be incremented
        // as it is stored (so when the parent gate is pushed OR when fetch is returned from a
        // sibling gate).

        //TODO: The gates array will need the direct parent unique_id or zero if it has no parent.
        // This will allow for the outstanding_number to be incremented and decremented at the
        // start and end.

        //TODO: Lets say I have Grandparent Gate A -> Parent Gate B -> Child Gate C. Assume B and C
        // are large gates.
        // If I cancel Grandparent Gate A, the parent gate B would still run to completion, this
        //  means child gate C would run to completion. Instead, maybe I can just query the map over
        //  and over because each higher level gate should have child gates. No, I can't do this,
        //  it requires knowing the children gates when the parent gate is passed to this method.
        //  I need to somehow build an object that shows relationships to the children and parents.
        //  Then cancel all children and parents that are outstanding when this function is called
        //  on that child.

        //TODO: Need a structure that can keep track of all children relationships. For now lets
        // say a map of vectors. Then each Id can store the child Ids that are outstanding. But
        // I can't use this to go 'up' the tree.

        //TODO: Need to go 'down' the tree when the canceled gate is pushed on to it. This will
        // avoid going 'up' the tree each time any gate is called to see if any parent was canceled.

        //TODO: So need a map of sets, the key is the parent id and the value is the child id.
        // Whenever a gate is called that is 'processing', it will need to iterate through all of
        // its outstanding children (and grandchildren etc...) and cancel them using the map.
        // Whenever a gate is called at all, it will need to add the next gates that are returned
        // from fetch to the vector for the parent ID AND it will need to remove itself.


        //TODO: Steps
        // Need to check if this gate should run in a single-threaded or multi-threaded way.
        // If single-threaded, run it (should be cancellable by checking the status of its parent)
        // If multi-threaded, store it in a separate Vec and run its children gates
        //  This needs to include a status (for CANCELLED or ACTIVE)
        //  When its outstanding children gates hit 0, it will need to be reloaded and be
        //   waiting_to_be_processed (maybe something different for ACTIVE and CANCELLED)

        // let _guard = self.mutex.lock().expect("Mutex failed to lock.");
        //
        //TODO: If contained in currently_processing_set
        // Interrupt the running thread that is doing it and have it restart
        // Need to take into account that the interrupt could happen as the thread running it is
        //  completing.
        // TODO: Maybe interrupting the smaller gates isn't a big deal, but should probably
        //  somehow interrupt the bigger gates This would mean that they need to somehow
        //   1) Add the larger gate to a queue (This needs to be done anyway in order to tell which
        //    gates are currently being processed).
        //   2) Pop them from the processed queue and if anything that is working on a small gate
        //    doesn't find them, don't complete.
        //  So I am thinking that there will be a distinction between simple and complex gates.
        //   Simple gates will just be added to a queue and directly worked by the threads. Complex
        //   Gates will be saved somewhere else. Then they can be either CANCELED or COMPLETED and
        //   the threads running the internal simple gates for the complex gate will react
        //   accordingly.
        //
        //
        // let inserted = self.waiting_to_be_processed_set.insert(
        //     gate_id
        // );
        //
        // if inserted {
        //     //If the gate is already in the queue, but not being processed, no need to do anything.
        //     return;
        // }
        //
        // self.gates.push(gate);
        //
        // //TODO: Will need to do something to activate the thread pool. If I use channels, I will
        // // have a 'gap' in between when the instance is retrieved from the thread pool and when the
        // // unique_id is moved from waiting to processing. So I probably need to use the same Mutex
        // // to protect the thread pool as I am using here and use the 'standard' thread pool design.
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

            let gate_output = gate.fetch_output_signals();

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
                        let next_gate = Arc::clone(&next_gate_info.gate);
                        let mut mutable_next_gate = next_gate.lock().unwrap();

                        let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                            mutable_next_gate.update_input_signal(next_gate_info.throughput.clone());
                        let gate_id = mutable_next_gate.get_unique_id();

                        let contains_id = next_gates_set.contains(&gate_id);

                        if print_output {
                            println!("checking gate {} tag {} signal {:?}", mutable_next_gate.get_gate_type(), mutable_next_gate.get_tag(), next_gate_info.throughput.signal.clone());
                            // println!("input_signal_updated: {} contains_key(): {:#?} changed_count_this_tick: {:?}", input_signal_updated, next_gates.contains_key(&gate_id), changed_count_this_tick);
                            println!("input_signal_updated: {input_signal_updated} propagate_signal_through_circuit: {propagate_signal_through_circuit} changed_count_this_tick {changed_count_this_tick} contains_id {contains_id}");
                        }

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
                        if (input_signal_updated || (propagate_signal_through_circuit && changed_count_this_tick == 1)) && !contains_id {
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

            let gate_output = gate.fetch_output_signals();

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
                        let mut mutable_next_gate = next_gate.lock().unwrap();

                        unique_gates.insert(mutable_next_gate.get_unique_id());

                        let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                            mutable_next_gate.update_input_signal(next_gate_info.throughput.clone());
                        let gate_id = mutable_next_gate.get_unique_id();

                        let contains_id = next_gates_set.contains(&gate_id);

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
                        if (input_signal_updated || changed_count_this_tick == 1) && !contains_id {
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

pub fn convert_binary_to_inputs_for_load(binary_strings: Vec<&str>, num_ram_cells: usize) -> Vec<SharedMutex<AutomaticInput>> {
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

pub fn collect_signals_from_logic_gate(gate: SharedMutex<dyn LogicGate>) -> Vec<Signal> {
    let cpu_output = gate.lock().unwrap().fetch_output_signals().unwrap();
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
    use crate::logic::basic_gates::{Not, Or};
    use crate::logic::foundations::Signal::{HIGH, LOW_};
    use crate::logic::input_gates::AutomaticInput;
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
                let output_signals = value.fetch_output_signals().unwrap();

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
}
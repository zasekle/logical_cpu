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
use crate::shared_mutex::{new_used_mutex, SharedMutex, UsedMutex};
use crate::test_stuff::extract_output_tags_sorted_by_index;

//TODO: set this to a higher value
//TODO: assert somewhere that this value is greater than 0
static NUM_CHILDREN_GATES_FOR_LARGE_GATE: usize = 7;

pub struct CondvarWrapper {
    cond: Condvar,
    mutex: UsedMutex<()>,
}

impl CondvarWrapper {
    fn new() -> Self {
        CondvarWrapper {
            cond: Condvar::new(),
            mutex: new_used_mutex(-1, ()),
        }
    }

    fn wait(&self) {
        let mut guard = self.mutex.lock().unwrap();
        let _unused_guard = self.cond.wait(guard.take_guard()).unwrap();
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
        held_by_thread: bool,
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
                    held_by_thread,
                    multiple_valid_input_gates,
                    ..
                } => {
                    //Note that gates cannot be locked inside Debug implementations or deadlock can
                    // occur. This means that gate information is not displayed.
                    format!("'ProcessingSizeOfGate::Large'\ngate: cannot be displayed\noutstanding_children: {}\nheld_by_thread: {}\nmultiple_valid_input_gates.len(): {}", outstanding_children, held_by_thread, multiple_valid_input_gates.len())
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
    fn convert_from_processing(size: &ProcessingSizeOfGate, initially_added: bool) -> Self {
        match size {
            ProcessingSizeOfGate::Large { outstanding_children, .. } => Self::Large {
                outstanding_children: *outstanding_children,
                initially_added,
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
    thread_pool_lists: Arc<UsedMutex<ThreadPoolLists>>,

    threads: Vec<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    propagate_signal: Arc<AtomicBool>,
    condvar_wrapper: Arc<CondvarWrapper>,
    num_threads_running: Arc<AtomicI32>,
    processing_completed: Arc<UsedMutex<bool>>,
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
                new_used_mutex(
                    -2,
                    ThreadPoolLists {
                        parental_tree: HashMap::from([(UniqueID::zero_id(), HashSet::new())]),
                        processing_set: HashMap::new(),
                        waiting_to_be_processed_set: HashMap::new(),
                        gates: VecDeque::new(),
                        input_gate_output_states: Vec::new(),
                    },
                )
            ),
            threads: Vec::new(),
            shutdown: Arc::new(AtomicBool::from(false)),
            propagate_signal: Arc::new(AtomicBool::from(false)),
            condvar_wrapper: Arc::new(CondvarWrapper::new()),
            num_threads_running: Arc::new(AtomicI32::new(0)),
            processing_completed: Arc::new(new_used_mutex(-2, false)),
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
            // //This must be set before the thread starts. That way it can be used below in this function
            // // to check when all threads are initialized.
            // num_threads_running_clone.fetch_add(1, Ordering::Acquire);

            thread_pool.threads.push(
                thread::spawn(move || {
                        let panic_result = std::panic::catch_unwind (move || {
                            println!("Thread {i} started, id {:?}", thread::current().id());

                            loop {
                                println!("\n");
                                if shutdown_clone.load(Ordering::Acquire) {
                                    println!("Thread {i} shutting down");
                                    break;
                                }

                                let mut increment_thread = true;
                                let mut large_gate_completed = false;
                                let popped_element;
                                loop {
                                    //The lock will be held as long as the MutexGuard is alive. So I
                                    // need to create a scope to make sure the lock is not held for the
                                    // duration of the task being run.
                                    let mut thread_pool_lists = thread_pool_lists_clone.lock().unwrap();

                                    if increment_thread {
                                        num_threads_running_clone.fetch_add(1, Ordering::Release);
                                        increment_thread = false;
                                    }

                                    //todo delete
                                    println!("Thread {i}\nparental_tree {:#?}", thread_pool_lists.parental_tree);

                                    let front_gate = thread_pool_lists.gates.pop_front();

                                    if let Some(gate) = &front_gate {
                                        let gate_id = gate.gate.lock().unwrap().get_unique_id().clone();
                                        let gate_num_children = gate.gate.lock().unwrap().num_children_gates();

                                        println!("{i} popped_element gate_id {} parent_id {}", gate_id.id(), gate.parent_id.id());

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
                                                    held_by_thread: true,
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
                                    println!("thread {i} running task ThreadId({:?})", thread::current().id());

                                    let element_num_children = running_gate.gate.lock().unwrap().num_children_gates();

                                    println!("thread {i} extracted children ThreadId({:?})", thread::current().id());

                                    let mut clock_tick_inputs = Vec::new();
                                    let mut next_gates = Vec::new();
                                    let mut next_gates_set = HashSet::new();
                                    let mut multiple_valid_signals = Vec::new();
                                    let mut number_gates_that_ran = 0;
                                    let mut parent_id = running_gate.parent_id;
                                    if element_num_children < NUM_CHILDREN_GATES_FOR_LARGE_GATE
                                        || large_gate_completed {
                                        let fetched_signals =
                                            if large_gate_completed {
                                                running_gate.gate.lock().unwrap().fetch_output_signals_no_calculate()
                                            } else {
                                                running_gate.gate.lock().unwrap().fetch_output_signals_calculate()
                                            };

                                        let is_input_gate = running_gate.gate.lock().unwrap().is_input_gate();
                                        let gate_tag = running_gate.gate.lock().unwrap().get_tag();

                                        match fetched_signals {
                                            Ok(output_states) => {
                                                if is_input_gate {
                                                    clock_tick_inputs.push(
                                                        (gate_tag.clone(), output_states.clone())
                                                    );
                                                }
                                                //TODO: Why isn't UniqueID 4 removed all the time?

                                                //TODO: Right now there is potential for deadlock here. Although
                                                // I am not sure where the other spot is at. But something will
                                                // stop running here and it will get stuck when the gate tries
                                                // to lock itself.
                                                // It COULD also be that both threads are stopped and a third one
                                                // would prevent this.
                                                // It happened when gate 10 was popped, then gate 4 was popped afterwards.
                                                // So I somewhere when gate 10 is called it must lock gate 4. It looks like
                                                // Thread 0 was what got stuck at this point and Thread 1 had already finished
                                                // processing gate 10. Thread 1 has not made it to the point that it printed
                                                // out the popped_element yet.
                                                //TODO: In order for deadlock to occur, two locks need to be locked, what is
                                                // the second lock though?
                                                //TODO: It pops siblings that are children of ID 0, it looks like the second
                                                // input gate and the memory cell gate.
                                                //TODO: So it deadlocks when the large gate is reached below at LARGE GATE,
                                                // then the same gate is printed here

                                                //TODO: Uncommenting this doesn't make the problem stop, it just makes it less likely.
                                                // 1) The Mutex is locking on the same thread when it is already locked.
                                                // 2) The Mutex is locking in the wrong order relative to another lock.
                                                // 3) Maybe inside of where it is locking, another lock occurs.
                                                println!("{i} output_states {:#?}", output_states);

                                                for gate_output_state in output_states {
                                                    match gate_output_state {
                                                        GateOutputState::NotConnected(signal) => {

                                                            if gate_tag == END_OUTPUT_GATE_TAG
                                                                && signal == HIGH {
                                                                println!("End of program reached on clock-tick {}. Stopping execution. ThreadId({:?})", get_clock_tick_number(), thread::current().id());
                                                                Self::internal_shutdown(
                                                                    &mut shutdown_clone,
                                                                    &mut signal_clone,
                                                                    &mut thread_pool_lists_clone,
                                                                );
                                                            }
                                                        }
                                                        GateOutputState::Connected(next_gate_info) => {
                                                            let next_gate = next_gate_info.gate.clone();
                                                            //TODO: remove prints

                                                            //TODO: There seems to be a problem here, it can get stuck here on the very first
                                                            // move where Thread 0 holds the first input gate at this point and then Thread 1
                                                            // holds the second input gate above at output_states. I don't see two different
                                                            // gates being locked at all in this situation. Or at least not out of order.
                                                            println!("{i} Connected about to lock ThreadId({:?})", thread::current().id());
                                                            // let mut mutable_next_gate = next_gate.lock().unwrap();
                                                            println!("{i} Connected locked ThreadId({:?})", thread::current().id());
                                                            // println!("{i} Connected locked type {} ThreadId({:?})", mutable_next_gate.get_gate_type(), thread::current().id());

                                                            //TODO: Locking occurs inside update_input_signal, need to look at it.
                                                            //TODO: So not sure if this is THE problem, but I do see a problem,
                                                            // 1) I extract the input gates through get_input_gates.
                                                            // 2) The parent gate has functions that directly work on and lock the input gates.
                                                            //  This means there is a lock inside a lock and b/c no order is guaranteed, it can deadlock.
                                                            // I suppose in an ideal world, the mutable data inside the gates would be the thing protected by
                                                            //  the locks. This I could skip locking the gate in general. But I think this would require each
                                                            //  and every child gate to have a lock around it instead of a single lock on the parent gate.
                                                            // Right now, gate 4 will run get_input_gates and return its input gates. Then when the next gate
                                                            //  calls update_input_signal, it will lock gate 4 and then lock the input gate. Then the input_gate
                                                            //  can lock, followed by a lock of its parent?
                                                            // Maybe there is a way for the input gates to share the same Mutex as the parent, then I can use a
                                                            //  re-entrant Mutex?
                                                            // So, is the only problem the order of locks? Can I somehow fix this so that I always access the
                                                            //  parent lock internally?
                                                            let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                                                                next_gate.lock().unwrap().update_input_signal(next_gate_info.throughput.clone());

                                                            //Faulted at gate 2 (a child of gate 4)

                                                            println!("{i} Connected gate_id ThreadId({:?})", thread::current().id());
                                                            let gate_id = next_gate.lock().unwrap().get_unique_id();

                                                            println!("{i} Connected contains_id ThreadId({:?})", thread::current().id());
                                                            let contains_id = next_gates_set.contains(&gate_id);

                                                            println!("{i} Connected check_if_next_gate_should_be_stored ThreadId({:?})", thread::current().id());
                                                            let should_update_gate = check_if_next_gate_should_be_stored(
                                                                input_signal_updated,
                                                                changed_count_this_tick,
                                                                contains_id,
                                                                propagate_signal_clone.load(Ordering::Relaxed),
                                                            );

                                                            if should_update_gate {
                                                                println!("{i} Connected number_children_in_gate ThreadId({:?})", thread::current().id());
                                                                let number_children_in_gate = next_gate.lock().unwrap().num_children_gates();
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

                                                            println!("{i} Connected unlocked ThreadId({:?})", thread::current().id());
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
                                        println!("{i} LARGE GATE reached ThreadId({:?})", thread::current().id());
                                        // let mutable_running_gate = running_gate.gate.lock().unwrap();
                                        println!("{i} locked ThreadId({:?})", thread::current().id());
                                        //When the gates are added below, the parent id will be the current gate for a large gate.
                                        parent_id = running_gate.gate.lock().unwrap().get_unique_id();
                                        println!("{i} parent_id ThreadId({:?})", thread::current().id());
                                        let input_gates = running_gate.gate.lock().unwrap().get_input_gates();
                                        println!("{i} input_gates ThreadId({:?})", thread::current().id());

                                        println!("LARGE GATE {} ThreadId({:?})", running_gate.gate.lock().unwrap().get_unique_id().id(), thread::current().id());
                                        println!("LARGE GATE input_gates_size {} ThreadId({:?})", input_gates.len(), thread::current().id());

                                        // drop(mutable_running_gate);

                                        for input_gate in input_gates.into_iter() {
                                            // let mutable_input_gate = input_gate.lock().unwrap();

                                            let gate_id = input_gate.lock().unwrap().get_unique_id();

                                            let contains_id = next_gates_set.contains(&gate_id);

                                            if !contains_id {
                                                let number_children_in_gate = input_gate.lock().unwrap().num_children_gates();
                                                // drop(mutable_input_gate);
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

                                    println!("next_gates.len() {} ThreadId({:?})", next_gates.len(), thread::current().id());

                                    let mut thread_pool_lists_guard = thread_pool_lists_clone.lock().unwrap();

                                    let gate_id = running_gate.gate.lock().unwrap().get_unique_id();

                                    let mut thread_pool_lists: &mut ThreadPoolLists = &mut thread_pool_lists_guard;

                                    let processing_element = thread_pool_lists.processing_set.get(
                                        &gate_id
                                    );
                                    //.expect(format!("A gate was removed from the processing_set while it was running. gate_id {}", gate_id.id()).as_str());

                                    thread_pool_lists.input_gate_output_states.append(
                                        &mut clock_tick_inputs
                                    );

                                    if let Some(processing_element) = processing_element {
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

                                                        let processing_element = thread_pool_lists.processing_set.get_mut(
                                                            &gate_id
                                                        ).expect(
                                                            "A gate was not found in the processing set immediately after it completed."
                                                        );

                                                        match processing_element.processing_type {
                                                            ProcessingSizeOfGate::Large { ref mut held_by_thread, .. } => {
                                                                *held_by_thread = false;
                                                            }
                                                            ProcessingSizeOfGate::Small => {}
                                                        }

                                                        //Insert gate as a parent.
                                                        thread_pool_lists.parental_tree.insert(
                                                            gate_id.clone(), HashSet::new(),
                                                        );

                                                        next_gates.len()
                                                    };

                                                println!("num_new_children {} number_gates_that_ran {} ThreadId({:?})", next_gates.len(), number_gates_that_ran, thread::current().id());
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
                                                //TODO: delete
                                                println!("REDO REACHED for gate id {}", gate_id.id());

                                                thread_pool_lists.waiting_to_be_processed_set.insert(
                                                    gate_id,
                                                    WaitingSizeOfGate::convert_from_processing(
                                                        &processing_element.processing_type,
                                                        true,
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
                                                //TODO: delete
                                                println!("CANCEL REACHED for gate id {}", gate_id.id());

                                                thread_pool_lists.processing_set.remove(
                                                    &gate_id
                                                );

                                                //If this gate was canceled, it was already removed from
                                                // parental_tree.
                                            }
                                        }
                                    }

                                    println!("waiting_to_be_processed_set.is_empty() {} num_threads_running {}", thread_pool_lists.waiting_to_be_processed_set.is_empty(), num_threads_running_clone.load(Ordering::Relaxed));
                                    println!("waiting_to_be_processed_set {:#?}", thread_pool_lists.waiting_to_be_processed_set);
                                    println!("gates.len() {:#?}", thread_pool_lists.gates.len());
                                    //TODO: So what happened here is that thread 0 got here and found that there were two threads running. Then
                                    // it went back to the top of the loop and got stuck at the mutex while this one did the same thing. Can
                                    // I move where 'num_threads_running' is changed?
                                    //This is the only thread running and there are no more elements to be
                                    // processed.
                                    if thread_pool_lists.waiting_to_be_processed_set.is_empty()
                                        && num_threads_running_clone.load(Ordering::Relaxed) <= 1
                                    {
                                        println!("complete_queue() called");
                                        Self::complete_queue(
                                            &mut processing_completed_clone,
                                            &mut wait_for_completion_clone,
                                        );
                                    }

                                    //This must be decremented while the lock is still held. This is so
                                    // that the threads will properly reach '1' when only one is running.
                                    num_threads_running_clone.fetch_add(-1, Ordering::Acquire);
                                } else {
                                    println!("Thread {i} sleeping");

                                    //NOTE: If thread_pool_lists is locked here, then it can end up in a situation
                                    // where the thread pool does not properly end. If I want to lock it, do it
                                    // after the fetch_add.

                                    num_threads_running_clone.fetch_add(-1, Ordering::Acquire);

                                    //todo: delete
                                    let processing_set_len = thread_pool_lists_clone.lock().unwrap().processing_set.len();
                                    let waiting_to_pro_set_len = thread_pool_lists_clone.lock().unwrap().waiting_to_be_processed_set.len();

                                    println!("processing_set_len {processing_set_len} waiting_to_pro_set_len {waiting_to_pro_set_len} parental_tree {:#?}", thread_pool_lists_clone.lock().unwrap().parental_tree);

                                    signal_clone.wait();
                                }
                            }
                        });

                    match panic_result {
                        Ok(_) => {
                            println!("Thread {:?} completed successfully", thread::current().id());
                        }
                        Err(err) => {
                            println!("Thread {:?} panic! with error {:?}", thread::current().id(), err);
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
                ..
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
        thread_pool_lists: &mut Arc<UsedMutex<ThreadPoolLists>>,
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
        processing_completed: &mut Arc<UsedMutex<bool>>,
        wait_for_completion: &mut Arc<Condvar>,
    ) {
        let mut completed = processing_completed.lock().unwrap();
        *completed = true;

        wait_for_completion.notify_all();
    }

    pub fn join(&mut self) -> bool {
        //Pause until the thread pool is completed.
        let mut guard = self.processing_completed.lock().unwrap().take_guard();
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
            let mut redo = false;

            if let Some(processing_gate) = processing.take() {
                println!("add_to_queue() Inside processing_set");
                if gate_element.number_children_in_gate < NUM_CHILDREN_GATES_FOR_LARGE_GATE { //small gate
                    processing_gate.status = ProcessingGateStatus::Cancel;
                } else { // large gate
                    if let ProcessingSizeOfGate::Large { held_by_thread, .. } = processing_gate.processing_type {
                        //TODO: handling held_by_thread
                        // So what I need below is to check held_by_thread for children gates that are
                        //  large gates, then I need to switch it to canceled if held_by_thread and make sure
                        //  that cancel will handle all the other cases. If !held_by_thread, then can just remove the gate from
                        //  the relevant sets.
                        redo = held_by_thread;
                        processing_gate.status = ProcessingGateStatus::Redo;
                    } else {
                        panic!("Large gate should always be a processing type of large")
                    }
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

            //TODO problems
            // Why aren't the other gates in the large gate being added to the parental tree at any point?
            // The large gate was not removed from parental_tree, but its child was.
            //  It was never removed from the waiting_to_be_processed_set
            //  What I need to do is list out the steps a large gate goes through and record it somewhere
            //   1) Large gate is added to the waiting_to_be_processed_set with initially_added==true and outstanding_children==0
            //   2) Mutex is locked by thread X and Large gate is popped from the waiting_to_be_processed_set and added to the processing_set.
            //   3) Mutex is unlocked by thread X and input gates are extracted from large gate.
            //   4) Mutex is locked by thread X and large gate is given a set in parental tree while input gates are added to the waiting_to_be_processed_set.
            //   5) Threads process child gates, each new child is added and each old child is removed.
            //   6) When outstanding_children hits 0, the gate is removed from the processing_set and added to the waiting_to_be_processed_set.
            //   7) Mutex is locked by thread Y and Large gate is popped from the waiting_to_be_processed_set and added to the processing_set.
            //   8) Mutex is unlocked by thread Y and fetch_output() is run on Large gate to retrieve the output gates.
            //   9) Mutex is locked by thread Y and large gate is removed from parental tree. Output gates are also added to the waiting_to_be_processed_set.
            // Even though the final output gate was processed, the system did not end. There is something to do
            //  with how the threads are counted as 'running'. They are running until they are asleep, so they
            //  can be running while waiting at the mutex, then bypass the condition to shut down. However,
            //  I believe that there was some reason I could not put the end condition under sleep.

            //TODO so redo is not reached for gate 4, but cancel is reached for the input gate 1,
            // I assume it is because I was assuming originally that a 'processing' gate was held
            //  by a thread and this would force the gate to be attempted again, but it COULD be
            //  held by the thread, however, there is no guarantee that it is. So what happens instead
            //  is that when it is not held by the thread, it will fizzle because no thread is directly
            //  in charge of restarting it.
            // I could make it so that it is held outstanding by a thread, this could lead to deadlock if too many large gates exist, it essentially wastes a thread.
            // I could make it so that there is a way to identify in the processing_set if it is held by a thread,
            //  If it IS held by a thread, then do what I am doing.
            //  If it is NOT held by a thread, then I will need to switch it up so that it is put back in the waiting to process queue (all its children will still need to be removed from
            //   the parental_tree and it may need to be removed itself).

            //TODO: It still isn't running the full OneBitMemory gate.

            fn remove_children(
                thread_pool_lists: &mut ThreadPoolLists,
                gate_id: UniqueID,
            ) {
                //It is possible for this to fail. This is because when a large gate is processing,
                // there is a time where it will not exist inside the tree. This is between the time
                // that it is popped from the gates array and before it has completed processing.
                println!("removing id {}", gate_id.id());
                let children_option = thread_pool_lists
                    .parental_tree
                    .remove(&gate_id);

                if let Some(children) = children_option {
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
            }

            remove_children(
                &mut thread_pool_lists,
                gate_element.gate_id.clone(),
            );

            //TODO: Failure happened when redo is true is reached, it then hit REDO REACHED and

            if redo {
                println!("redo is true");
                //Do not want the current gate removed because it is going to redo. But want it to be empty.
                thread_pool_lists
                    .parental_tree
                    .insert(
                        gate_element.gate_id,
                        HashSet::new(),
                    );
            } else {
                println!("redo is false");
                //The current gate was not held by a thread and so it must be removed complete and added
                // to the waiting queue.
                thread_pool_lists
                    .processing_set
                    .remove(
                        &gate_element.gate_id
                    )
                    .expect(
                        "The processing element was not found immediately after it was found above."
                    );

                let waiting_element = thread_pool_lists
                    .waiting_to_be_processed_set
                    .get_mut(
                        &gate_element.gate_id
                    );
                // .expect("The waiting to be processed element was added about and should exist here.");

                if let Some(WaitingSizeOfGate::Large { ref mut initially_added, .. }) = waiting_element {
                    *initially_added = true;
                }

                thread_pool_lists.gates.push_back(
                    ParentIdAndGate {
                        parent_id: gate_element.parent_id,
                        gate: gate_element.gate,
                    }
                );
            }

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
                        // let mut mutable_next_gate = next_gate.lock().unwrap();

                        let InputSignalReturn { changed_count_this_tick, input_signal_updated } =
                            next_gate.lock().unwrap().update_input_signal(next_gate_info.throughput.clone());
                        let gate_id = next_gate.lock().unwrap().get_unique_id();

                        let contains_id = next_gates_set.contains(&gate_id);

                        let should_update_gate = check_if_next_gate_should_be_stored(
                            input_signal_updated,
                            changed_count_this_tick,
                            contains_id,
                            propagate_signal_through_circuit,
                        );

                        if print_output {
                            println!("checking gate {} tag {} signal {:?}", next_gate.lock().unwrap().get_gate_type(), next_gate.lock().unwrap().get_tag(), next_gate_info.throughput.signal.clone());
                            // println!("input_signal_updated: {} contains_key(): {:#?} changed_count_this_tick: {:?}", input_signal_updated, next_gates.contains_key(&gate_id), changed_count_this_tick);
                            // println!("input_signal_updated: {input_signal_updated} propagate_signal_through_circuit: {propagate_signal_through_circuit} changed_count_this_tick {changed_count_this_tick} contains_id {contains_id}");
                        }

                        if should_update_gate {
                            if print_output {
                                println!("Pushing gate {} tag {}", next_gate.lock().unwrap().get_gate_type(), next_gate.lock().unwrap().get_tag());
                            }
                            // drop(mutable_next_gate);
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
    use std::ops::Deref;
    use std::time::Duration;
    use crate::logic::basic_gates::{And, Not, Or};
    use crate::logic::foundations::Signal::{HIGH, LOW_};
    use crate::logic::input_gates::AutomaticInput;
    use crate::logic::memory_gates::{OneBitMemoryCell, VariableBitMemoryCell};
    use crate::logic::output_gates::SimpleOutput;
    use crate::run_circuit::run_circuit;
    use crate::shared_mutex::new_shared_mutex;
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

    fn create_large_gate() -> (usize, SharedMutex<VariableBitMemoryCell>) {
        let mut number_bits = 1;
        let mut variable_bit_mem_cell = VariableBitMemoryCell::new(number_bits);
        let mut number_children = variable_bit_mem_cell.lock().unwrap().num_children_gates();
        while number_children < NUM_CHILDREN_GATES_FOR_LARGE_GATE {
            number_bits += 1;
            variable_bit_mem_cell = VariableBitMemoryCell::new(number_bits);
            number_children = variable_bit_mem_cell.lock().unwrap().num_children_gates();
        }
        (number_bits, variable_bit_mem_cell)
    }

    #[test]
    fn single_large_gate_multi_thread() {
        for _ in 0..100 {
            //TODO: uncomment
            // let (number_bits, variable_bit_mem_cell) = create_large_gate();
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
            println!("set_input_gate {}", set_input_gate.lock().unwrap().get_unique_id().id());
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
            println!("enable_input_gate {}", enable_input_gate.lock().unwrap().get_unique_id().id());
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
            println!("output_gate {}", output_gate.lock().unwrap().get_unique_id().id());
            output_gates.push(output_gate.clone());

            connect_gates(
                one_bit_memory_cell.clone(),
                0,
                output_gate.clone(),
                0,
            );

            println!("one_bit_memory_cell id {}", one_bit_memory_cell.lock().unwrap().get_unique_id().id());
            println!("one_bit_memory_cell num_children_gates {}", one_bit_memory_cell.lock().unwrap().num_children_gates());

            let mut thread_pool = RunCircuitThreadPool::new(2); //todo num_cpus::get() - 1);

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

                        match gate_output_state {
                            GateOutputState::NotConnected(signal) => {
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
    }

    //TODO: delete this test
    #[test]
    fn force_deadlock() {
        let first = 1;
        let second = 2;

        let first_mutex = new_shared_mutex(1, first);
        let second_mutex = new_shared_mutex(2, second);

        let clone_one_one = first_mutex.clone();
        let clone_two_one = second_mutex.clone();

        let clone_one_two = first_mutex.clone();
        let clone_two_two = second_mutex.clone();

        struct HigherLevelObject {
            lower_level: SharedMutex<i32>,
        }

        let third = 3;
        let high_object = HigherLevelObject {
            lower_level: new_shared_mutex(3, third),
        };

        let testing_object = new_shared_mutex(4, high_object);
        let testing_object_one = testing_object.clone();
        let testing_object_two = testing_object.clone();

        println!("First testing_object access.");
        let object = testing_object.lock().unwrap();

        let low_object = object.lower_level.lock().unwrap();

        println!("First low_object {}.", *low_object);

        drop(low_object);
        drop(object);
        println!("First testing_object finished.");

        let first_thread = thread::spawn(move || {
            println!("1 testing_object access.");
            let object = testing_object_one.lock().unwrap();

            let low_object = object.lower_level.lock().unwrap();

            println!("1 low_object {}.", *low_object);

            drop(low_object);
            drop(object);
            println!("1 testing_object finished.");

            let mutex_guard_first = clone_one_one.lock();

            thread::sleep(Duration::from_millis(200));

            let mutex_guard_second = clone_two_one.lock();

            println!("First Thread both locked!");
        });

        let second_thread = thread::spawn(move || {
            println!("2 testing_object access.");
            let object = testing_object_two.lock().unwrap();

            let low_object = object.lower_level.lock().unwrap();

            println!("2 low_object {}.", *low_object);
            drop(low_object);
            drop(object);
            println!("2 testing_object finished.");

            let mutex_guard_first = clone_two_two.lock();

            thread::sleep(Duration::from_millis(200));

            let mutex_guard_second = clone_one_two.lock();

            println!("Second Thread both locked!");
        });

        first_thread.join().expect("First thread error");

        second_thread.join().expect("Second thread error");
    }


    //TODO: tests
    // simple loop
    // first tick propagates
    // multiple ticks
    // GateLogicError::NoMoreAutomaticInputsRemaining works
    // END_OUTPUT_GATE_TAG works

    //TODO: remember to clean out all the println statements.
}
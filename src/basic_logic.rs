use std::rc::{Rc, Weak};
use crate::basic_logic::Signal::{HIGH, LOW};

#[derive(PartialEq, Debug, Clone)]
pub enum Signal {
    LOW,
    HIGH,
}

#[derive(PartialEq, Debug, Clone)]
pub enum LogicError {
    InvalidInputSize,
}

impl Signal {
    fn swap(s: &Signal) -> Signal {
        return if *s == LOW {
            HIGH
        } else {
            LOW
        };
    }
}

//TODO: All input needs to be passed with the current clock_tick OR some kind of shared resource.
struct Node {
    connections: Vec<Rc<Node>>,
    parent: Weak<LogicNodes>,
    signal: Signal,
    current_clock_tick: usize,
    oscillations: usize,
}

impl Node {
    fn new(parent: Weak<LogicNodes>) -> Self {
        Node {
            connections: Vec::new(),
            parent,
            signal: LOW,
            current_clock_tick: 0,
            oscillations: 0,
        }
    }

    //TODO: Need to crash if oscillations hits a certain point.
}

pub struct LogicNodes {
    inputs: Vec<Box<Node>>,
    outputs: Vec<Box<Node>>,
    changed_count: usize,
    input_size: usize,
    output_size: usize,
}

impl LogicNodes {
    fn new(input_size: usize, output_size: usize) -> Rc<Self> {
        let inputs = Vec::with_capacity(input_size);
        let outputs = Vec::with_capacity(output_size);

        let mut self_reference = Rc::new(
            LogicNodes {
                inputs,
                outputs,
                changed_count: 0,
                input_size,
                output_size,
            }
        );

        let weak_self_reference = Rc::downgrade(&self_reference);
        let mut mut_self_reference = Rc::get_mut(&mut self_reference).unwrap();

        mut_self_reference.inputs.resize_with(
            input_size,
            || Box::new(Node::new(Weak::clone(&weak_self_reference))),
        );

        mut_self_reference.outputs.resize_with(
            output_size,
            || Box::new(Node::new(Weak::clone(&weak_self_reference))),
        );

        self_reference
    }
}

//So lets say I start somewhere. Then I go through

// 1) So first and foremost I want to go through all nodes of the clock. These will be the output
//  nodes.
// 2) I will go through and collect each gate that is connected to the output nodes (I can get them
//  through the input nodes, skip the ones that don't change at all I suppose).
// 3) Once all output nodes have been iterated through, I will go through the collected gates and
//  iterate through each output node again (the loop starts over).
//TODO: Does this approach really go through a single logic object at a time? Or is that even what
// I want? So should it go through the ALU first?


//TODO: Extract this to a separate struct and trait.
//TODO: Maybe wrap this in something to eliminate the need to make a reference count every time. Or
// maybe this is the wrapper for LogicNodes.
pub struct And {
    logic_nodes: Rc<LogicNodes>,
}

impl And {
    //TODO: Need to store current state.
    pub fn new(input_size: usize, output_size: usize) -> Self {
        let mut logic_nodes = LogicNodes::new(input_size, output_size);

        And {
            logic_nodes,
        }
    }

    //TODO: I would like for a single 'unit' to complete before moving on to another unit. So what
    // does that even mean in practice? Is it even possible/practical to do? It seems that if I
    // start at my clock, then iterate through each wire coming off it, then iterate through each
    // 'next' one etc... would it be different doing breadth vs depth?
    // breadth seems more reasonable, plus I may be able to multi-thread it eventually.

    //TODO: What is the flow like for this system of doing it?
    // 1) Signal goes in to a Node.
    // 2) The Node calls the parent and runs whatever operation it needs.
    // 3) Somehow the output is called and the next node is accessed.

    //TODO: Will need to think about de-allocations. This is because the And stores a reference to
    // Nodes and Node stores a reference to And.
}

trait LogicGate {
    fn connect_output(&mut self, output_index: usize, next_gate: Rc<dyn LogicGate>, next_gate_input_index: usize);

    //Returns true if something changed.
    fn change_input(&mut self, input_index: usize, signal: Signal) -> bool;

    fn collect_output(&self) -> Vec<OutputNode>;
}

#[derive(Clone)]
struct OutputNode {
    output_signal: Signal,
    input_index: usize,
    gate: Rc<dyn LogicGate>,
}

//Let me re-think this a little bit, I don't really see the point of the Nodes themselves. As long
// as I just make a Vector of traits as output along with the pin number I think it should be fine.
pub struct Or {
    inputs: Vec<Signal>,
    outputs: Vec<OutputNode>, //TODO: may need to be Rc
}

impl Or {
    pub fn new(input_num: usize, output_num: usize) -> Self {
        Or {
            inputs: Vec::with_capacity(input_num),
            outputs: Vec::with_capacity(output_num),
        }
    }
}

impl LogicGate for Or {
    fn connect_output(&mut self, output_index: usize, next_gate: Rc<dyn LogicGate>, next_gate_input_index: usize) {
        self.outputs[output_index] = OutputNode {
            output_signal: LOW,
            input_index: next_gate_input_index,
            gate: next_gate,
        };
    }

    fn change_input(&mut self, input_index: usize, signal: Signal) -> bool {
        if self.inputs[input_index] == signal {
            false
        } else {
            self.inputs[input_index] == signal;
            true
        }
    }

    fn collect_output(&self) -> Vec<OutputNode> {
        let mut output_signal = LOW;
        for s in self.inputs.iter() {
            if *s == HIGH {
                output_signal = HIGH;
                break;
            }
        }

        let mut output_clone = self.outputs.clone();
        output_clone.iter_mut().for_each(|mut f| f.output_signal = output_signal.clone());

        output_clone
    }
}

// pub trait LogicUnit {
//     fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError>;
// }
//
// pub struct And;
//
// impl LogicUnit for And {
//     fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
//         if input.len() < 2 {
//             return Err(LogicError::InvalidInputSize);
//         }
//
//         let result = if input.iter().all(|i| *i == HIGH) {
//             HIGH
//         } else {
//             LOW
//         };
//
//         Ok(vec![result])
//     }
// }
//
// pub struct Or;
//
// impl LogicUnit for Or {
//     fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
//         if input.len() < 2 {
//             return Err(LogicError::InvalidInputSize);
//         }
//
//         let result = if input.iter().any(|i| *i == HIGH) {
//             HIGH
//         } else {
//             LOW
//         };
//
//         Ok(vec![result])
//     }
// }
//
// pub struct Nand;
//
// impl LogicUnit for Nand {
//     fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
//         let and_gate = And{};
//         not_output(
//             &and_gate
//                 .input(input)
//                 .expect("Nand input was invalid")
//         )
//     }
// }
//
// pub struct Nor;
//
// impl LogicUnit for Nor {
//     fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
//         let or_gate = Or{};
//         not_output(
//             &or_gate
//                 .input(input)
//                 .expect("Nor input was invalid")
//         )
//     }
// }
//
// fn not_output(input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
//     if input.is_empty() {
//         return Err(LogicError::InvalidInputSize);
//     }
//
//     Ok(
//         input
//             .iter()
//             .map(|s| Signal::swap(s) )
//             .collect()
//     )
// }

// use std::rc::{Rc, Weak};
//
// struct Parent {
//     children: Vec<Node>,
// }
//
// struct Node {
//     parent: Weak<Parent>,
// }
//
// impl Parent {
//     fn new() -> Rc<Self> {
//         Rc::new(Parent { children: vec![] })
//     }
//
//     fn add_child(&mut self, parent_weak: Weak<Self>) {
//         let child = Node {
//             parent: Weak::clone(&parent_weak),
//         };
//         self.children.push(child);
//     }
// }
//
// fn main() {
//     let mut parent = Parent::new();
//     let parent_weak = Rc::downgrade(&parent);
//     let mut parent_ref = Rc::get_mut(&mut parent).unwrap();
//
//     parent_ref.add_child(parent_weak);
// }

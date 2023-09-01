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
    current_clock_tick: usize,
    oscillations: usize,
}

impl Node {
    fn new(parent: Weak<LogicNodes>) -> Self {
        Node {
            connections: Vec::new(),
            parent,
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

mod basic_logic;

use crate::basic_logic::{Signal::{HIGH, LOW}, And, Or, Nor, Nand, LogicUnit};

fn main() {
    let and_gate = And{};
    let or_gate = Or{};
    let nor_gate = Nor{};
    let nand_gate = Nand{};

    let input = [HIGH, HIGH, HIGH, HIGH];
    println!("and_gate: {:#?}", and_gate.input(&input).unwrap());

    let input = [HIGH, HIGH];
    println!("or_gate: {:#?}", or_gate.input(&input).unwrap());

    let input = [HIGH, HIGH, LOW];
    println!("nor_gate: {:#?}", nor_gate.input(&input).unwrap());

    let input = [HIGH, HIGH];
    println!("nand_gate: {:#?}", nand_gate.input(&input).unwrap());

    //There is a bit of a problem with how I am thinking about things. That is the idea that state
    // itself can be considered here. For example, with the very basic circuit that is used in memory
    // the Active S R latches, they actually change based on the previous state. So maybe these need
    // to store the current input, then I can simply 'check' what their output is.

    //Also with this model how would I simulate a gate attaching back to itself? Maybe Instead of
    // Signal inputs, I actually give them a copy of an object that represents input or output?


    //I could do a few things.
    // 1) Keep track of state.
    //   - So the way this would need to work is that I would have an input and an output for each
    //     gate. Then the start would need to be the clock. Then I suppose there would be something
    //     I could modify so that I could have human input. But something else would need to keep
    //     track of the order of logic gates and such.
    // 2) Maybe add in connections or nodes or something.
    //   - I could have a 'connection' somehow. Say instead of just the logic gate, it also stores
    //     the units it connects to.


    //TODO: How would I eliminate loops? Maybe when I pass the connection through I assign each
    // logic unit a clock tick number. Then if the clock has already ticked ignore it. This could
    // cause other problems I assume. For example, if I propagate something out depth-wise maybe
    // when I increase the breadth, it will change these? Or maybe I should propagate based on the
    // unit, then go depth after that. This way


    //So my current idea is that, lets leave it the way it is. All solutions will have problems.
    // I will 'wire' it up manually inside larger and larger units. And I will add a clock tick
    // number to each unit.

    //TODO: How do I give it manual inputs? Maybe the clock works on a separate thread to the input
    // and I just feed it commands from the GUI?

    loop { //clock

        let and_gate = And{};



    }
}

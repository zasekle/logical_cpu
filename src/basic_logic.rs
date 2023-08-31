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
        }
    }
}

pub trait LogicUnit {
    fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError>;
}

pub struct And;

impl LogicUnit for And {
    fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
        if input.len() < 2 {
            return Err(LogicError::InvalidInputSize);
        }

        let result = if input.iter().all(|i| *i == HIGH) {
            HIGH
        } else {
            LOW
        };

        Ok(vec![result])
    }
}

pub struct Or;

impl LogicUnit for Or {
    fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
        if input.len() < 2 {
            return Err(LogicError::InvalidInputSize);
        }

        let result = if input.iter().any(|i| *i == HIGH) {
            HIGH
        } else {
            LOW
        };

        Ok(vec![result])
    }
}

pub struct Nand;

impl LogicUnit for Nand {
    fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
        let and_gate = And{};
        not_output(
            &and_gate
                .input(input)
                .expect("Nand input was invalid")
        )
    }
}

pub struct Nor;

impl LogicUnit for Nor {
    fn input(&self, input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
        let or_gate = Or{};
        not_output(
            &or_gate
                .input(input)
                .expect("Nor input was invalid")
        )
    }
}

fn not_output(input: &[Signal]) -> Result<Vec<Signal>, LogicError> {
    if input.is_empty() {
        return Err(LogicError::InvalidInputSize);
    }

    Ok(
        input
            .iter()
            .map(|s| Signal::swap(s) )
            .collect()
    )
}

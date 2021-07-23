use crate::stack;
use crate::stack::parsing;
use std::rc::{ Rc };

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Num(f64),
    Str(String),
    Bool(bool),
    Code(Rc<Vec<parsing::Ops>>)
}

impl Cell {
    pub fn add(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a + b)),
            (Cell::Str(a), Cell::Str(b)) => Ok(Cell::Str(format!("{}{}", a, b))),
            _ => Err(stack::Error::InvalidType(String::from("+ is only defined for num and str"))),
        }
    }

    pub fn sub(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a - b)),
            _ => Err(stack::Error::InvalidType(String::from("- is only defined for num"))),
        }
    }

    pub fn mul(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a * b)),
            _ => Err(stack::Error::InvalidType(String::from("* is only defined for num"))),
        }
    }

    pub fn div(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a / b)),
            _ => Err(stack::Error::InvalidType(String::from("/ is only defined for num "))),
        }
    }

    pub fn equals(self, other: Cell) -> Result<Cell, stack::Error> {
        Ok(Cell::Bool(self == other))
    }

    pub fn not(self) -> Result<Cell, stack::Error> {
        match self {
            Cell::Bool(b) => Ok(Cell::Bool(!b)),
            _ => Err(stack::Error::InvalidType(String::from("not is only defined for bool"))),
        }
    }

    pub fn and(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Bool(a), Cell::Bool(b)) => Ok(Cell::Bool(a && b)),
            _ => Err(stack::Error::InvalidType(String::from("and is only defined for bool"))),
        }
    }

    pub fn or(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Bool(a), Cell::Bool(b)) => Ok(Cell::Bool(a || b)),
            _ => Err(stack::Error::InvalidType(String::from("or is only defined for bool"))),
        }
    }
}
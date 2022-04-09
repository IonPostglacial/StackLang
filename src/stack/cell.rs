use crate::stack;
use crate::stack::parsing;

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Num(f64),
    Str(String),
    Bool(bool),
    Code(Vec<parsing::Ops>),
}

impl Cell {
    pub fn add(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a + b)),
            (Cell::Str(a), Cell::Str(b)) => Ok(Cell::Str(format!("{}{}", a, b))),
            _ => Err(stack::Error::InvalidType(
                "+ is only defined for num and str".into(),
            )),
        }
    }

    pub fn sub(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a - b)),
            _ => Err(stack::Error::InvalidType(
                "- is only defined for num".into(),
            )),
        }
    }

    pub fn mul(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a * b)),
            _ => Err(stack::Error::InvalidType(
                "* is only defined for num".into(),
            )),
        }
    }

    pub fn div(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a / b)),
            _ => Err(stack::Error::InvalidType(
                "/ is only defined for num ".into(),
            )),
        }
    }

    pub fn equals(self, other: Cell) -> Result<Cell, stack::Error> {
        Ok(Cell::Bool(self == other))
    }

    pub fn not(self) -> Result<Cell, stack::Error> {
        match self {
            Cell::Bool(b) => Ok(Cell::Bool(!b)),
            _ => Err(stack::Error::InvalidType(
                "not is only defined for bool".into(),
            )),
        }
    }

    pub fn and(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Bool(a), Cell::Bool(b)) => Ok(Cell::Bool(a && b)),
            _ => Err(stack::Error::InvalidType(
                "and is only defined for bool".into(),
            )),
        }
    }

    pub fn or(self, other: Cell) -> Result<Cell, stack::Error> {
        match (self, other) {
            (Cell::Bool(a), Cell::Bool(b)) => Ok(Cell::Bool(a || b)),
            _ => Err(stack::Error::InvalidType(
                "or is only defined for bool".into(),
            )),
        }
    }
}

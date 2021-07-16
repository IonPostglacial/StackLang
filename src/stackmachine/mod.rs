mod lex;
mod parsing;

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    Num(f64),
    Str(String),
    Bool(bool),
    Code(Vec<parsing::Ops>)
}

impl Cell {
    fn add(self, other: Cell) -> Result<Cell, StackError> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a + b)),
            (Cell::Str(a), Cell::Str(b)) => Ok(Cell::Str(format!("{}{}", a, b))),
            _ => Err(StackError::InvalidType),
        }
    }

    fn sub(self, other: Cell) -> Result<Cell, StackError> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a - b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn mul(self, other: Cell) -> Result<Cell, StackError> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a * b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn div(self, other: Cell) -> Result<Cell, StackError> {
        match (self, other) {
            (Cell::Num(a), Cell::Num(b)) => Ok(Cell::Num(a / b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn equals(self, other: Cell) -> Result<Cell, StackError> {
        Ok(Cell::Bool(self == other))
    }

    fn not(self) -> Result<Cell, StackError> {
        match self {
            Cell::Bool(b) => Ok(Cell::Bool(!b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn and(self, other: Cell) -> Result<Cell, StackError> {
        match (self, other) {
            (Cell::Bool(a), Cell::Bool(b)) => Ok(Cell::Bool(a && b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn or(self, other: Cell) -> Result<Cell, StackError> {
        match (self, other) {
            (Cell::Bool(a), Cell::Bool(b)) => Ok(Cell::Bool(a || b)),
            _ => Err(StackError::InvalidType),
        }
    }
}

#[derive(Debug)]
pub enum StackError {
    Unimplemented,
    InvalidType,
    StackUnderflow,
}

pub struct StackMachine {
    stack: Vec<Cell>,
}

pub fn new() -> StackMachine {
    StackMachine { stack: vec![] }
}

impl StackMachine {
    pub fn push(&mut self, cell: Cell) {
        self.stack.push(cell);
    }

    pub fn pop(&mut self) -> Result<Cell, StackError> {
        match self.stack.pop() {
            Some(cell) => Ok(cell),
            None => Err(StackError::StackUnderflow),
        }
    }

    fn exec_symbol(&mut self, sym: lex::Symbol) -> Result<(), StackError> {
        match sym {
            lex::Symbol::Add => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.add(op1)?))
            },
            lex::Symbol::Sub => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.sub(op1)?))
            },
            lex::Symbol::Mul => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.mul(op1)?))
            },
            lex::Symbol::Div => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.div(op1)?))
            },
            lex::Symbol::Pop => {
                self.pop()?;
                Ok(())
            },
            lex::Symbol::Dup => {
                let mut new_top: Option<Cell> = None;
                {
                    if let Some(cell) = self.stack.last() {
                        new_top = Some(cell.clone())
                    }
                };
                if let Some(cell) = new_top {
                    Ok(self.push(cell))
                } else {
                    Err(StackError::StackUnderflow)
                }
            },
            lex::Symbol::True => Ok(self.push(Cell::Bool(true))),
            lex::Symbol::False => Ok(self.push(Cell::Bool(false))),
            lex::Symbol::Eq => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.equals(op1)?))
            },
            lex::Symbol::Not => {
                let op = self.pop()?;
                Ok(self.push(op.not()?))
            },
            lex::Symbol::And => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.and(op1)?))
            },
            lex::Symbol::Or => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.or(op1)?))
            },
            lex::Symbol::If => {
                let if_false = self.pop()?;
                let if_true = self.pop()?;
                let cond_cell = self.pop()?;
                match (cond_cell, if_true, if_false) {
                    (Cell::Bool(cond), Cell::Code(ops_true), Cell::Code(ops_false)) => {
                        self.exec_ops(if cond { ops_true } else { ops_false })
                    }
                    _ => Err(StackError::InvalidType)
                }
            },
            lex::Symbol::Custom(_) => Err(StackError::Unimplemented),
        }
    }

    fn exec_ops(&mut self, ops: Vec<parsing::Ops>) -> Result<(), StackError> {
        for op in ops {
            if let Err(err) = match op {
                parsing::Ops::Push(cell) => Ok(self.push(cell)),
                parsing::Ops::Call(sym) => self.exec_symbol(sym),
                parsing::Ops::Err(_) => Err(StackError::Unimplemented),
            } {
                return Err(err);
            }
        }
        Ok(())
    }

    pub fn eval(&mut self, source: &str) -> Result<(), StackError> {
        let tokens = lex::lex_source(source);
        if let Some(ops) = parsing::parse_tokens(tokens) {
            self.exec_ops(ops)?;
        }
        Ok(())
    }

    pub fn print_stack(self) {
        println!("{:?}", self.stack)
    }
}
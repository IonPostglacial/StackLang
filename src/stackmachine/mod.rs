mod parsing;

#[derive(Debug, Clone, PartialEq)]
pub enum StackCell {
    Num(f64),
    Str(String),
    Bool(bool),
}

impl StackCell {
    fn add(self, other: StackCell) -> Result<StackCell, StackError> {
        match (self, other) {
            (StackCell::Num(a), StackCell::Num(b)) => Ok(StackCell::Num(a + b)),
            (StackCell::Str(a), StackCell::Str(b)) => Ok(StackCell::Str(format!("{}{}", a, b))),
            _ => Err(StackError::InvalidType),
        }
    }

    fn sub(self, other: StackCell) -> Result<StackCell, StackError> {
        match (self, other) {
            (StackCell::Num(a), StackCell::Num(b)) => Ok(StackCell::Num(a - b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn mul(self, other: StackCell) -> Result<StackCell, StackError> {
        match (self, other) {
            (StackCell::Num(a), StackCell::Num(b)) => Ok(StackCell::Num(a * b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn div(self, other: StackCell) -> Result<StackCell, StackError> {
        match (self, other) {
            (StackCell::Num(a), StackCell::Num(b)) => Ok(StackCell::Num(a / b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn inc(self) -> Result<StackCell, StackError> {
        match self {
            StackCell::Num(n) => Ok(StackCell::Num(n + 1.0)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn dec(self) -> Result<StackCell, StackError> {
        match self {
            StackCell::Num(n) => Ok(StackCell::Num(n - 1.0)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn equals(self, other: StackCell) -> Result<StackCell, StackError> {
        Ok(StackCell::Bool(self == other))
    }

    fn not(self) -> Result<StackCell, StackError> {
        match self {
            StackCell::Bool(b) => Ok(StackCell::Bool(!b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn and(self, other: StackCell) -> Result<StackCell, StackError> {
        match (self, other) {
            (StackCell::Bool(a), StackCell::Bool(b)) => Ok(StackCell::Bool(a && b)),
            _ => Err(StackError::InvalidType),
        }
    }

    fn or(self, other: StackCell) -> Result<StackCell, StackError> {
        match (self, other) {
            (StackCell::Bool(a), StackCell::Bool(b)) => Ok(StackCell::Bool(a || b)),
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
    stack: Vec<StackCell>,
}

pub fn new() -> StackMachine {
    StackMachine { stack: vec![] }
}

impl StackMachine {
    pub fn push(&mut self, cell: StackCell) {
        self.stack.push(cell);
    }

    pub fn pop(&mut self) -> Result<StackCell, StackError> {
        match self.stack.pop() {
            Some(cell) => Ok(cell),
            None => Err(StackError::StackUnderflow),
        }
    }

    fn exec_symbol(&mut self, sym: parsing::Symbol) -> Result<(), StackError> {
        match sym {
            parsing::Symbol::Add => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.add(op1)?))
            },
            parsing::Symbol::Sub => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.sub(op1)?))
            },
            parsing::Symbol::Mul => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.mul(op1)?))
            },
            parsing::Symbol::Div => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.div(op1)?))
            },
            parsing::Symbol::Inc => {
                let op = self.pop()?;
                Ok(self.push(op.inc()?))
            },
            parsing::Symbol::Dec => {
                let op = self.pop()?;
                Ok(self.push(op.dec()?))
            },
            parsing::Symbol::Pop => {
                self.pop()?;
                Ok(())
            },
            parsing::Symbol::Dup => {
                let mut new_top: Option<StackCell> = None;
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
            parsing::Symbol::True => Ok(self.push(StackCell::Bool(true))),
            parsing::Symbol::False => Ok(self.push(StackCell::Bool(false))),
            parsing::Symbol::Eq => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.equals(op1)?))
            },
            parsing::Symbol::Not => {
                let op = self.pop()?;
                Ok(self.push(op.not()?))
            },
            parsing::Symbol::And => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.and(op1)?))
            },
            parsing::Symbol::Or => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.or(op1)?))
            },
            parsing::Symbol::Custom(_) => Err(StackError::Unimplemented),
        }
    }

    pub fn eval(&mut self, source: &str) -> Result<(), StackError> {
        for token in parsing::parse_source(source) {
            if let Err(err) = match token {
                parsing::Token::Num(n) => Ok(self.push(StackCell::Num(n))),
                parsing::Token::Str(s) => Ok(self.push(StackCell::Str(String::from(s)))),
                parsing::Token::Sym(s) => self.exec_symbol(s),
                parsing::Token::Err(_) => Err(StackError::Unimplemented),
            } {
                return Err(err);
            }
        }
        Ok(())
    }

    pub fn print_stack(self) {
        println!("{:?}", self.stack)
    }
}
use internment::Intern;

use super::{Cell, Code, Op, Symbol, Ops, parse_string};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Error {
    Unimplemented(String),
    InvalidType(String),
    StackUnderflow,
    CallStackUnderflow,
    ParsingError,
}

pub struct Machine {
    stack: Vec<Cell>,
    call_stack: Vec<usize>,
    instruction_counter: usize,
    definitions: HashMap<Intern<Symbol>, Code>,
}

impl Machine {
    pub fn new() -> Machine {
        let mut definitions: HashMap<Intern<Symbol>, Code> = HashMap::new();
        definitions.insert(Symbol::new_global("print"), Code::BuiltIn(|this| {
            if let Some(cell) = this.stack.last() {
                match cell {
                    Cell::Int(n) => print!("{n}"),
                    Cell::Sym(s) => print!(":{}", s.name),
                    Cell::Str(s) => print!("{s}"),
                    Cell::Code(c) => print!("{:?}", c),
                    Cell::False => print!("false"),
                    Cell::True => print!("true"),
                }
            }
            Ok(())
        }));
        definitions.insert(Symbol::new_global("linebreak"), Code::BuiltIn(|_| {
            println!();
            Ok(())
        }));
        Machine {
            stack: Vec::new(),
            call_stack: Vec::new(),
            instruction_counter: 0,
            definitions,
        }
    }

    pub fn push(&mut self, cell: Cell) {
        self.stack.push(cell);
    }

    pub fn pop(&mut self) -> Result<Cell, Error> {
        match self.stack.pop() {
            Some(cell) => Ok(cell),
            None => Err(Error::StackUnderflow),
        }
    }

    fn exec_code(&mut self, code: &Code) -> Result<(), Error> {
        match code {
            Code::Custom(gosub) => {
                self.call_stack.push(self.instruction_counter);
                self.instruction_counter = *gosub;
            }
            Code::BuiltIn(f) => {
                f(self)?;
                self.instruction_counter += 1;
            }
        }
        Ok(())
    }

    fn exec_symbol(&mut self, sym: Intern<Symbol>) -> Result<(), Error> {
        let code = self.definitions.get(&sym).ok_or(Error::Unimplemented(sym.name.to_string()))?;
        self.exec_code(&code.clone())?;
        Ok(())
    }

    fn binary_int_op(&mut self, f: fn (n: i64, m: i64) -> i64) -> Result<(), Error> {
        self.instruction_counter += 1;
        let op1 = self.pop()?;
        let op2 = self.pop()?;
        match (op1, op2) {
            (Cell::Int(n), Cell::Int(m)) => {
                Ok(self.push(Cell::Int(f(n, m))))
            }
            _ => Err(Error::InvalidType("op takes 2 integers".to_string()))
        }
    }

    fn comparison_op(&mut self, f: fn (n: i64, m: i64) -> bool) -> Result<(), Error> {
        self.instruction_counter += 1;
        let op1 = self.pop()?;
        let op2 = self.pop()?;
        match (op1, op2) {
            (Cell::Int(n), Cell::Int(m)) => {
                Ok(self.push(if f(n, m) { Cell::True } else { Cell::False }))
            }
            _ => Err(Error::InvalidType("op takes 2 integers".to_string()))
        }
    }

    fn swap(&mut self) -> Result<(), Error> {
        let len = self.stack.len();
        if len < 2 {
            return Err(Error::StackUnderflow)
        }
        self.stack.swap(len - 1, len - 2);
        Ok(())
    }

    pub fn eval(&mut self, src: &str) -> Result<Vec<Cell>, Error> {
        let ops = parse_string(src).ok_or(Error::ParsingError)?;
        self.eval_ops(&ops)
    }

    pub fn eval_ops(&mut self, code: &Ops) -> Result<Vec<Cell>, Error> {
        self.instruction_counter = code.start;
        while self.instruction_counter < code.ops.len() {
            match &code.ops[self.instruction_counter] {
                Op::Push(cell) => {
                    self.push(cell.clone());
                    self.instruction_counter += 1;
                }
                Op::Call(sym) => self.exec_symbol(*sym)?,
                Op::Return => {
                    match self.call_stack.pop() {
                        Some(i) => self.instruction_counter = i + 1,
                        None => {
                            return Err(Error::CallStackUnderflow);
                        }
                    }
                }
                Op::Def => {
                    let def_sym = self.pop()?;
                    let def_body = self.pop()?;
                    match (def_sym, def_body) {
                        (Cell::Sym(sym), Cell::Code(ops)) => {
                            self.definitions.insert(sym, ops);
                        }
                        _ => Err(Error::InvalidType("'def' takes a code block and a symbol".to_string()))?,
                    }
                    self.instruction_counter += 1;
                }
                Op::Add => self.binary_int_op(|n, m| m + n)?,
                Op::Mul => self.binary_int_op(|n, m| m * n)?,
                Op::Sub => self.binary_int_op(|n, m| m - n)?,
                Op::Div => self.binary_int_op(|n, m| m / n)?,
                Op::Lt => self.comparison_op(|n, m| m < n)?,
                Op::Lte => self.comparison_op(|n, m| m <= n)?,
                Op::Gt => self.comparison_op(|n, m| m > n)?,
                Op::Gte => self.comparison_op(|n, m| m >= n)?,
                Op::Eq => {
                    let op1 = self.pop()?;
                    let op2 = self.pop()?;
                    self.push(if op1 == op2 { Cell::True } else { Cell::False });
                    self.instruction_counter += 1;
                }
                Op::Drop => {
                    self.pop()?; 
                    self.instruction_counter += 1;
                }
                Op::Dup => {
                    let mut new_top: Option<Cell> = None;
                    {
                        if let Some(cell) = self.stack.last() {
                            new_top = Some(cell.clone())
                        }
                    };
                    if let Some(cell) = new_top {
                        self.push(cell);
                    } else {
                        return Err(Error::StackUnderflow)
                    }
                    self.instruction_counter += 1;
                }
                Op::Swap => {
                    self.swap()?;
                    self.instruction_counter += 1;
                }
                Op::Rot => {
                    let len = self.stack.len();
                    if len < 3 {
                        return Err(Error::StackUnderflow)
                    }
                    self.stack.swap(len - 3, len - 2);
                    self.stack.swap(len - 2, len - 1);
                    self.instruction_counter += 1;
                }
                Op::UnRot => {
                    let len = self.stack.len();
                    if len < 3 {
                        return Err(Error::StackUnderflow)
                    }
                    self.stack.swap(len - 2, len - 1);
                    self.stack.swap(len - 3, len - 2);
                    self.instruction_counter += 1;
                }
                Op::Not => {
                    let cond = self.pop()?;
                    if cond == Cell::False {
                        self.push(Cell::True)
                    } else {
                        self.push(Cell::False)
                    }
                    self.instruction_counter += 1;
                }
                Op::Exec => {
                    let code = self.pop()?;
                    match code {
                        Cell::Code(code) => {
                            self.exec_code(&code)?;
                        }
                        _ => Err(Error::InvalidType("Code expected".to_string()))?,
                    }
                }
                Op::CondPop => {
                    let len = self.stack.len();
                    if len < 3 {
                        return Err(Error::StackUnderflow)
                    }
                    let cond = &self.stack[len - 3];
                    if cond == &Cell::False {
                        self.stack.swap(len - 1, len - 3); 
                    } else {
                        self.stack.swap(len - 2, len - 3); 
                    }
                    self.stack.truncate(len - 2);
                    self.instruction_counter += 1;
                }
                Op::While => {
                    let body = self.pop()?;
                    let cond = self.pop()?;
                    match body {
                        Cell::Code(code) => {
                            if cond == Cell::False {
                                self.instruction_counter += 1;
                            } else {
                                self.instruction_counter -= 2;
                                self.exec_code(&code)?;
                            }
                        }
                        _ => Err(Error::InvalidType("While expects a code block".to_string()))?,
                    }
                }
            }
        }
        Ok(self.stack.clone())
    }
}
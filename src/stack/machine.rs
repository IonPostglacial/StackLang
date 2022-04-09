use crate::stack;
use crate::stack::lex;
use crate::stack::parsing::{parse_tokens, Ops};
use std::borrow::Borrow;
use std::collections::HashMap;

pub struct Machine {
    stack: Vec<stack::Cell>,
    definitions: HashMap<String, Vec<Ops>>,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            stack: vec![],
            definitions: HashMap::new(),
        }
    }

    pub fn push(&mut self, cell: stack::Cell) {
        self.stack.push(cell);
    }

    pub fn pop(&mut self) -> Result<stack::Cell, stack::Error> {
        match self.stack.pop() {
            Some(cell) => Ok(cell),
            None => Err(stack::Error::StackUnderflow),
        }
    }

    fn exec_symbol(&mut self, sym: lex::Symbol) -> Result<(), stack::Error> {
        match sym {
            lex::Symbol::Add => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.add(op1)?))
            }
            lex::Symbol::Sub => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.sub(op1)?))
            }
            lex::Symbol::Mul => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.mul(op1)?))
            }
            lex::Symbol::Div => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.div(op1)?))
            }
            lex::Symbol::Pop => {
                self.pop()?;
                Ok(())
            }
            lex::Symbol::Dup => {
                let mut new_top: Option<stack::Cell> = None;
                {
                    if let Some(cell) = self.stack.last() {
                        new_top = Some(cell.clone())
                    }
                };
                if let Some(cell) = new_top {
                    Ok(self.push(cell))
                } else {
                    Err(stack::Error::StackUnderflow)
                }
            }
            lex::Symbol::Swap => {
                let op2 = self.pop()?;
                let op1 = self.pop()?;
                self.push(op2);
                self.push(op1);
                Ok(())
            }
            lex::Symbol::Rot => {
                let op3 = self.pop()?;
                let op2 = self.pop()?;
                let op1 = self.pop()?;
                self.push(op2);
                self.push(op3);
                self.push(op1);
                Ok(())
            }
            lex::Symbol::True => Ok(self.push(stack::Cell::Bool(true))),
            lex::Symbol::False => Ok(self.push(stack::Cell::Bool(false))),
            lex::Symbol::Eq => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.equals(op1)?))
            }
            lex::Symbol::Not => {
                let op = self.pop()?;
                Ok(self.push(op.not()?))
            }
            lex::Symbol::And => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.and(op1)?))
            }
            lex::Symbol::Or => {
                let op1 = self.pop()?;
                let op2 = self.pop()?;
                Ok(self.push(op2.or(op1)?))
            }
            lex::Symbol::If => {
                let if_false = self.pop()?;
                let if_true = self.pop()?;
                let cond_cell = self.pop()?;
                match (cond_cell, if_true, if_false) {
                    (
                        stack::Cell::Bool(cond),
                        stack::Cell::Code(ops_true),
                        stack::Cell::Code(ops_false),
                    ) => self.exec_ops(if cond {
                        ops_true.borrow()
                    } else {
                        ops_false.borrow()
                    }),
                    _ => Err(stack::Error::InvalidType("'if' takes a boolean and 2 code blocks".into())),
                }
            }
            lex::Symbol::While => {
                let loop_body = self.pop()?;
                match loop_body {
                    stack::Cell::Code(loop_ops) => {
                        loop {
                            self.exec_ops(&loop_ops)?;
                            if self.stack.len() == 0
                                || self.stack.pop() != Some(stack::Cell::Bool(true))
                            {
                                break;
                            }
                        }
                        Ok(())
                    }
                    _ => Err(stack::Error::InvalidType("'while' takes 1 code block".into())),
                }
            }
            lex::Symbol::Def => {
                let def_body = self.pop()?;
                let def_sym = self.pop()?;
                match (def_sym, def_body) {
                    (stack::Cell::Str(sym), stack::Cell::Code(ops)) => {
                        self.definitions.insert(sym, ops);
                        Ok(())
                    }
                    _ => Err(stack::Error::InvalidType("'def' takes a string and a code block".into())),
                }
            }
            lex::Symbol::Exec => match self.pop()? {
                stack::Cell::Code(ops) => self.exec_ops(ops.borrow()),
                _ => Err(stack::Error::InvalidType("'exec' takes 1 code block".into())),
            },
            lex::Symbol::Print => {
                if let Some(cell) = self.stack.last() {
                    println!("{:?}", cell);
                }
                Ok(())
            }
            lex::Symbol::Custom(sym) => {
                let mut maybe_ops: Option<Vec<Ops>> = None;
                {
                    match self.definitions.get(&sym) {
                        Some(ops) => maybe_ops = Some(ops.clone()),
                        None => {}
                    }
                }
                if let Some(ops) = maybe_ops {
                    self.exec_ops(ops.borrow())
                } else {
                    Err(stack::Error::Unimplemented(format!(
                        "'{}' is not implemented",
                        sym
                    )))
                }
            }
        }
    }

    fn exec_ops(&mut self, ops: &Vec<Ops>) -> Result<(), stack::Error> {
        for op in ops {
            if let Err(err) = match op {
                Ops::Push(cell) => Ok(self.push(cell.clone())),
                Ops::Call(sym) => self.exec_symbol(sym.clone()),
                Ops::Err(err) => Err(stack::Error::Parsing(err.clone())),
            } {
                return Err(err);
            }
        }
        Ok(())
    }

    pub fn eval(&mut self, source: &str) -> Result<(), stack::Error> {
        let tokens = lex::lex_source(source);
        if let Some(ops) = parse_tokens(tokens) {
            self.exec_ops(&ops)?;
        }
        Ok(())
    }

    pub fn print_stack(self) {
        println!("{:?}", self.stack)
    }
}

use internment::Intern;
use super::{Error, Machine};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub namespace: String,
    pub name: String,
}

impl Symbol {
    pub fn new_global(name: &str) -> Intern<Symbol> {
        Intern::new(Symbol { namespace: "".to_string(), name: name.to_string() })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // Word handling
    Push(Cell),
    Call(Intern<Symbol>),
    Return,
    Def,
    // Arithmetic
    Add,
    Mul,
    Sub,
    Div,
    // Comparison
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    // Stack
    Drop,
    Dup,
    Swap,
    Rot,
    UnRot,
    // Logic
    Not,
    // Control Flow
    Exec,
    CondPop,
    While,
}

#[derive(Debug)]
pub struct Ops {
    pub ops: Vec<Op>,
    pub start: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Code {
    Custom(usize),
    BuiltIn(fn (machine: &mut Machine) -> Result<(), Error>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    False,
    True,
    Int(i64),
    Sym(Intern<Symbol>),
    Str(Intern<String>),
    Code(Code),
}
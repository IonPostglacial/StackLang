mod cell;
mod lex;
mod machine;
mod parsing;

#[derive(Debug)]
pub enum Error {
    Unimplemented(String),
    InvalidType(String),
    Parsing(String),
    StackUnderflow,
}

pub use cell::Cell;
pub use machine::Machine;

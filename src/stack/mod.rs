mod lex;
mod parsing;
mod machine;
mod cell;

#[derive(Debug)]
pub enum Error {
    Unimplemented(String),
    InvalidType(String),
    Parsing(String),
    StackUnderflow,
}

pub use cell::Cell;
pub use machine::Machine;
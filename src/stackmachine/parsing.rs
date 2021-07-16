use crate::stackmachine;
use crate::stackmachine::lex;
use std::rc::{ Rc };

#[derive(Debug, Clone, PartialEq)]
pub enum Ops {
    Push(stackmachine::Cell),
    Call(lex::Symbol),
    Err(String),
}

pub fn parse_tokens<'a>(tokens: lex::TokenStream) -> Option<Vec<Ops>> {
    let mut ops_blocks: Vec<Vec<Ops>> = vec![vec![]];
    for token in tokens {
        match token {
            lex::Token::Num(n) => ops_blocks.last_mut()?.push(Ops::Push(stackmachine::Cell::Num(n))),
            lex::Token::Str(s) => ops_blocks.last_mut()?.push(Ops::Push(stackmachine::Cell::Str(String::from(s)))),
            lex::Token::Sym(s) => ops_blocks.last_mut()?.push(Ops::Call(s)),
            lex::Token::Err(err) => ops_blocks.last_mut()?.push(Ops::Err(err)),
            lex::Token::OpenBlock => {
                let new_ops_block = vec![];
                ops_blocks.push(new_ops_block);
            }
            lex::Token::CloseBlock => {
                if let Some(completed_ops_block) = ops_blocks.pop() {
                    ops_blocks.last_mut()?.push(Ops::Push(stackmachine::Cell::Code(Rc::new(completed_ops_block))));
                }
            }
        }
    }
    match ops_blocks.last() {
        Some(ops) => Some(ops.clone()),
        None => None,
    }
}
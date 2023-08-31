use std::{str::Chars, iter::Peekable};
use internment::Intern;

use super::{Cell, Code, Op, Ops, Symbol};

#[derive(Debug, Clone)]
enum TokenKind {
    String, Number, Symbol, Word
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}


#[derive(Debug)]
struct Lexer {
    pos: usize,
}

fn is_space(ch: char) -> bool {
    match ch {
        ' ' | '\t' | '\n' | '\r' => true,
        _ => false
    }
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer { pos: 0 }
    }

    fn consume_char(&mut self, text: &mut Peekable<Chars<'_>>) {
        text.next();
        self.pos += 1;
    }

    fn consume_while(&mut self, text: &mut Peekable<Chars<'_>>, p: fn (char) -> bool) -> Option<usize> {
        loop {
            match text.peek() {
                Some(&ch) if p(ch) => {
                    self.consume_char(text);
                }
                None | Some(_) => { return Some(self.pos); }
            }
        }
    }

    fn consume_space(&mut self, text: &mut Peekable<Chars<'_>>) {
        self.consume_while(text, is_space);
    }

    fn consume_com(&mut self, text: &mut Peekable<Chars<'_>>) {
        self.consume_while(text, |ch| ch != '\n');
        
    }

    fn consume_str(&mut self, text: &mut Peekable<Chars<'_>>) -> Option<Token> {
        let start = self.pos + 1;
        let mut end;
        loop {
            self.consume_char(text);
            end = self.consume_while(text, |ch| ch != '"')?;
            self.consume_char(text);
            match text.peek() {
                Some('"') => continue,
                None | Some(_) => break,
            }
        }
        Some(Token { kind: TokenKind::String, start, end })
    }

    fn consume_sym(&mut self, text: &mut Peekable<Chars<'_>>) -> Option<Token> {
        let start = self.pos + 1;
        let end = self.consume_while(text, |ch| !is_space(ch))?;
        Some(Token { kind: TokenKind::Symbol, start, end })
    }

    fn consume_num(&mut self, text: &mut Peekable<Chars<'_>>) -> Option<Token> {
        let start = self.pos;
        let end = self.consume_while(text, |ch| !is_space(ch))?;
        Some(Token { kind: TokenKind::Number, start, end })
    }

    fn consume_word(&mut self, text: &mut Peekable<Chars<'_>>) -> Option<Token> {
        let start = self.pos;
        let end = self.consume_while(text, |ch| !is_space(ch))?;
        Some(Token { kind: TokenKind::Word, start, end })
    }

    pub fn next(&mut self, src: &str) -> Option<Token> {
        let mut text = src[self.pos..].chars().peekable();
        match text.peek() {
            Some('#') => {
                self.consume_com(&mut text);
                self.next(src)
            }
            Some('"') => self.consume_str(&mut text),
            Some(':') => self.consume_sym(&mut text),
            Some('0'..='9') => self.consume_num(&mut text),
            Some(&ch) if is_space(ch) => {
                self.consume_space(&mut text);
                self.next(src)
            }
            Some(_) => self.consume_word(&mut text),
            None => None,
        }
    }
}

pub fn parse_string<'a>(src: &str) -> Option<Ops> {
    let mut ops: Vec<Op> = Vec::new();
    let mut ops_blocks: Vec<Vec<Op>> = vec![Vec::new()];
    let mut lx = Lexer::new();
    while let Some(token) = lx.next(src) {
        let word = &src[token.start..token.end];
        match token.kind {
            TokenKind::Number => ops_blocks.last_mut()?.push(Op::Push(Cell::Int(word.parse::<i64>().expect("invalid number")))),
            TokenKind::String => ops_blocks.last_mut()?.push(Op::Push(Cell::Str(Intern::new(word.replace("\"\"", "\""))))),
            TokenKind::Symbol => ops_blocks.last_mut()?.push(Op::Push(Cell::Sym(Symbol::new_global(word)))),
            TokenKind::Word  => {
                match word {
                    "{" => {
                        let new_ops_block = vec![];
                        ops_blocks.push(new_ops_block);
                    }
                    "}" => {
                        if let Some(ref mut completed_ops_block) = ops_blocks.pop() {
                            let sub_start = ops.len();
                            ops.append(completed_ops_block);
                            ops.push(Op::Return);
                            ops_blocks
                                .last_mut()?
                                .push(Op::Push(Cell::Code(Code::Custom(sub_start))));
                        }
                    }
                    "false" => ops_blocks.last_mut()?.push(Op::Push(Cell::False)),
                    "true" => ops_blocks.last_mut()?.push(Op::Push(Cell::True)),
                    "def" => ops_blocks.last_mut()?.push(Op::Def),
                    "+" => ops_blocks.last_mut()?.push(Op::Add),
                    "*" => ops_blocks.last_mut()?.push(Op::Mul),
                    "-" => ops_blocks.last_mut()?.push(Op::Sub),
                    "/" => ops_blocks.last_mut()?.push(Op::Div),
                    ">" => ops_blocks.last_mut()?.push(Op::Gt),
                    ">=" => ops_blocks.last_mut()?.push(Op::Gte),
                    "<" => ops_blocks.last_mut()?.push(Op::Lt),
                    "<=" => ops_blocks.last_mut()?.push(Op::Lte),
                    "=" => ops_blocks.last_mut()?.push(Op::Eq),
                    "." => ops_blocks.last_mut()?.push(Op::Drop),
                    "dup" => ops_blocks.last_mut()?.push(Op::Dup),
                    "swap" => ops_blocks.last_mut()?.push(Op::Swap),
                    "rot" => ops_blocks.last_mut()?.push(Op::Rot),
                    "-rot" => ops_blocks.last_mut()?.push(Op::UnRot),
                    "not" => ops_blocks.last_mut()?.push(Op::Not),
                    "exec" => ops_blocks.last_mut()?.push(Op::Exec),
                    "if" => {
                        ops_blocks.last_mut()?.push(Op::CondPop);
                        ops_blocks.last_mut()?.push(Op::Exec)
                    }
                    "while" => {
                        let last = ops_blocks.last_mut()?;
                        last.push(Op::Push(Cell::True));
                        let len = last.len();
                        last.swap(len - 1, len - 2);
                        ops_blocks.last_mut()?.push(Op::While)
                    }
                    word => {
                        ops_blocks.last_mut()?.push(Op::Call(Symbol::new_global(word)))
                    }
                }
            }
        }
    }
    match ops_blocks.pop() {
        Some(ref mut top_level) => {
            let start = ops.len();
            ops.append(top_level);
            Some(Ops { ops, start: start })
        }
        None => None,
    }
}
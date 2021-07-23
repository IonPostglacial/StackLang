#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    Add,
    Sub,
    Mul,
    Div,
    Pop,
    Dup,
    Swap,
    Rot,
    True,
    False,
    Eq,
    Not,
    And,
    Or,
    If,
    While,
    Def,
    Exec,
    Print,
    Custom(String),
}

#[derive(Debug)]
pub enum Token<'a> {
    Num(f64),
    Str(&'a str),
    Sym(Symbol),
    Err(String),
    OpenBlock,
    CloseBlock,
}

pub struct TokenStream<'a> {
    src: &'a str,
    pos: usize,
}

fn produce_token(tok_str: &str, in_num: bool) -> Option<Token> {
    if in_num {
        match tok_str.parse::<f64>() {
            Ok(num) => Some(Token::Num(num)),
            Err(_) => Some(Token::Err(format!("Invalid number: '{}'", tok_str))),
        }
    } else if tok_str == "{" {
        Some(Token::OpenBlock)
    } else if tok_str == "}" {
        Some(Token::CloseBlock)
    } else {
        Some(Token::Sym(match tok_str {
            "+" => Symbol::Add,
            "-" => Symbol::Sub,
            "*" => Symbol::Mul,
            "/" => Symbol::Div,
            "." => Symbol::Pop,
            "dup" => Symbol::Dup,
            "swap" => Symbol::Swap,
            "rot" => Symbol::Rot,
            "true" => Symbol::True,
            "false" => Symbol::False,
            "=" => Symbol::Eq,
            "not" => Symbol::Not,
            "and" => Symbol::And,
            "or" => Symbol::Or,
            "if" => Symbol::If,
            "while" => Symbol::While,
            "def" => Symbol::Def,
            "exec" => Symbol::Exec,
            "print" => Symbol::Print,
            _ => Symbol::Custom(String::from(tok_str))
        }))
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        if self.pos >= self.src.len() { return None }

        let starting_pos = self.pos;
        let mut in_tok = false;
        let mut in_str = false;
        let mut in_num = false;

        for (i, c) in self.src[starting_pos..].chars().enumerate() {
            let current_index = starting_pos + i;

            if c == '"' {
                in_str = !in_str;
                if !in_str {
                    let tok = Token::Str(&self.src[self.pos..current_index]);
                    self.pos = current_index + 1;
                    return Some(tok)
                } else {
                    self.pos = current_index + 1;
                    continue
                }
            }
            if !in_str {
                match c {
                    ' ' | '\t' => {
                        let was_in_tok = in_tok;
                        let was_in_num = in_num;
                        in_num = false;
                        in_tok = false;
                        if was_in_tok {
                            let tok_str = &self.src[self.pos..current_index];
                            self.pos = current_index + 1;
                            return produce_token(tok_str, was_in_num)
                        } else {
                            self.pos = current_index + 1
                        }
                    },
                    '0' ..= '9' => {
                        if !in_tok {
                            in_num = true;
                            in_tok = true;
                        }
                    }
                    _ => {
                        in_tok = true;
                    }
                }
            }
        }
        if in_str {
            Some(Token::Err(String::from("Invalid token")))
        } else {
            let last_tok = &self.src[self.pos..];
            self.pos = self.src.len();
            produce_token(last_tok, in_num)
        }
    }
}

pub fn lex_source(source: &str) -> TokenStream {
    TokenStream { src: source, pos: 0 }
}
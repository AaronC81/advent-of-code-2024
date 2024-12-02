use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    LiteralInteger(isize),
    Action(String),
    Binding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Atom(Atom),
    LBrace,
    RBrace,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    input
        .split_ascii_whitespace()
        .map(|part|
            if let Ok(num) = part.parse() {
                Ok(Token::Atom(Atom::LiteralInteger(num)))
            } else if part.chars().all(|c| is_valid_identifier_char(c)) {
                Ok(Token::Atom(Atom::Action(part.to_owned())))
            } else if part.starts_with('$') && part.chars().skip(1).all(|c| is_valid_identifier_char(c)) {
                Ok(Token::Atom(Atom::Binding(part.to_owned())))
            } else if part == "{" {
                Ok(Token::LBrace)
            } else if part == "}" {
                Ok(Token::RBrace)
            } else {
                Err(format!("unknown token `{part}`").into())
            }
        )
        .collect()
}

fn is_valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || ['_', '+', '-', '*', '/', '=', '^', ':', '.', '?', '[', ']', '#', '@', '<', '>', '&', '|', '!'].contains(&c)
}

use std::error::Error;

use crate::loc::Loc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Atom {
    LiteralInteger(isize),
    LiteralChar(char),
    Action(String),
    Binding(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Atom(Atom),
    LBrace,
    RBrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub loc: Loc,
}

impl Token {
    pub fn new(kind: TokenKind, loc: Loc) -> Self {
        Self { kind, loc }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    input
        .split_ascii_whitespace() // TODO: need a variant of this which bundles `Loc`s
        .map(tokenize_one)
        .map(|kind|
            kind.map(|kind| Token::new(kind, Loc::stub())))
        .collect()
}

/// Convert a single token as a string to a [TokenKind].
fn tokenize_one(token: &str) -> Result<TokenKind, Box<dyn Error>> {
    if let Ok(num) = token.parse() {
        Ok(TokenKind::Atom(Atom::LiteralInteger(num)))
    } else if token.chars().all(|c| is_valid_identifier_char(c)) {
        Ok(TokenKind::Atom(Atom::Action(token.to_owned())))
    } else if token.starts_with('$') && token.chars().skip(1).all(|c| is_valid_identifier_char(c)) {
        Ok(TokenKind::Atom(Atom::Binding(token.to_owned())))
    } else if token.starts_with('\'') && token.ends_with('\'') {
        let chars = token.chars().collect::<Vec<_>>();
        if chars.len() != 3 { // ' x '
            return Err(format!("invalid character literal: {token}").into());
        }

        let c = chars[1];
        Ok(TokenKind::Atom(Atom::LiteralChar(c)))
    } else if token == "{" {
        Ok(TokenKind::LBrace)
    } else if token == "}" {
        Ok(TokenKind::RBrace)
    } else {
        Err(format!("unknown token `{token}`").into())
    }
}

fn is_valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || ['_', '+', '-', '*', '/', '=', '^', ':', '.', ',', '?', '[', ']', '#', '@', '<', '>', '&', '|', '!'].contains(&c)
}

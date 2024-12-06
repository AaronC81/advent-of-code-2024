use std::{error::Error, rc::Rc};

use crate::loc::{Loc, LocSource};

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

pub fn tokenize(source: &LocSource) -> Result<Vec<Token>, Box<dyn Error>> {    
    split_whitespace_with_loc(source)
        .map(|(token, loc)| (tokenize_one(&token), loc))
        .map(|(kind, loc)|
            kind.map(|kind| Token::new(kind, loc)))
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

/// Like `split_whitespace` but includes a [Loc] with each item.
fn split_whitespace_with_loc(source: &LocSource) -> impl Iterator<Item = (String, Loc)> {
    let chars = source.contents
        .chars()
        .chain([' '].into_iter()) // Force a final buffer flush by adding some whitespace on the end
        .enumerate();

    let mut buffer: Option<(String, usize)> = None;
    let mut items = vec![];
    for (i, char) in chars {
        if char.is_whitespace() {
            // If there is a buffer, 'finalize' it into the list of items
            // (Otherwise, we can harmlessly skip the consecutive whitespace)
            if let Some((contents, start)) = buffer {
                let loc = Loc::new(source.clone(), start, contents.len());
                items.push((contents, loc));
            }
            buffer = None;
        } else {
            buffer = match buffer {
                // If there's no buffer, start filling one up
                None => Some((char.to_string(), i)),

                // If there's already one, add to it
                Some((contents, start)) => Some((contents + &char.to_string(), start))
            }
        }
    }

    items.into_iter()
}

fn is_valid_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || ['_', '+', '-', '*', '/', '=', '^', ':', '.', ',', '?', '[', ']', '#', '@', '<', '>', '&', '|', '!'].contains(&c)
}

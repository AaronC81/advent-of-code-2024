use std::error::Error;

use crate::token::{Atom, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Atom(Atom),
    Sequence(Vec<Node>),
    Block(Box<Node>),
}

pub fn parse(mut tokens: Vec<Token>) -> Result<Node, Box<dyn Error>> {
    // Reverse tokens so we get a stack which we can pop from
    tokens.reverse();

    let node = parse_sequence(&mut tokens, false)?;

    if !tokens.is_empty() {
        return Err(format!("unable to parse from: {:?}", tokens.pop().unwrap()).into())
    }

    Ok(node)
}

fn parse_sequence(tokens: &mut Vec<Token>, in_block: bool) -> Result<Node, Box<dyn Error>> {
    let mut items = vec![];

    while let Some(token) = tokens.pop() {
        match token {
            Token::Atom(atom) => items.push(Node::Atom(atom)),

            Token::LBrace => {
                let body = parse_sequence(tokens, true)?;
                items.push(Node::Block(Box::new(body)))
            }

            Token::RBrace => {
                if in_block {
                    return Ok(Node::Sequence(items))
                } else {
                    return Err("unexpected end of block while not inside a block".into())
                }
            }
        }
    }

    if in_block {
        return Err("ran out of tokens while inside block".into())
    }

    return Ok(Node::Sequence(items)) 
}

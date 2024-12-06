use std::error::Error;

use crate::{loc::Loc, token::{Atom, Token, TokenKind}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Atom(Atom),
    Sequence(Vec<Node>),
    Block(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: NodeKind,
    pub loc: Loc,
}

impl Node {
    pub fn new(kind: NodeKind, loc: Loc) -> Self {
        Self { kind, loc }
    }
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

    while let Some(Token { kind, loc }) = tokens.pop() {
        match kind {
            TokenKind::Atom(atom) => items.push(Node::new(NodeKind::Atom(atom), loc)),

            TokenKind::LBrace => {
                let body = parse_sequence(tokens, true)?;
                items.push(Node::new(NodeKind::Block(Box::new(body)), loc))
            }

            TokenKind::RBrace => {
                if in_block {
                    // `items` will be empty for an empty block - if so, point at the brace
                    let span_loc =
                        if items.is_empty() {
                            loc
                        } else {
                            loc_spanning(&items)
                        };
                    return Ok(Node::new(NodeKind::Sequence(items), span_loc))
                } else {
                    return Err("unexpected end of block while not inside a block".into())
                }
            }
        }
    }

    if in_block {
        return Err("ran out of tokens while inside block".into())
    }

    // I don't think it's possible for `items` to be empty here
    let loc = loc_spanning(&items);
    return Ok(Node::new(NodeKind::Sequence(items), loc)) 
}

fn loc_spanning(nodes: &[Node]) -> Loc {
    nodes.iter().fold(
        nodes.first().unwrap().loc.clone(),
        |acc, el| Loc::new_spanning(&acc, &el.loc)
    )
}

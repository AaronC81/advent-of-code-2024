use std::{env::args, error::Error, fs::read_to_string, path::Path, process::abort};

use eval::{Interpreter, Value};
use parser::{parse, Node};
use preprocess::preprocess;
use token::tokenize;

mod preprocess;
mod token;
mod parser;
mod eval;

pub fn code_to_node(code: &str) -> Result<Node, Box<dyn Error>> {
    let preprocessed = preprocess(&code);
    let tokens = tokenize(&preprocessed)?;
    let root = parse(tokens)?;

    Ok(root)
}

fn main() -> Result<(), Box<dyn Error>> {
    let code_path = args().nth(1).expect("no code path passed");
    let input_path = args().nth(2);

    // Load standard library
    let stdlib = code_to_node(include_str!("../lib/stdlib.stk"))?;

    // Load input code
    let code = read_to_string(code_path)?;
    let root = code_to_node(&code)?;

    let input = input_path.map(|p| read_to_string(p)).transpose()?;

    let mut interpreter = Interpreter::new();
    if let Some(input) = input {
        interpreter.set_top_level_binding("$input", Value::String(input));
    }
    interpreter.execute(&stdlib)?; // Load stdlib
    interpreter.execute(&root)?;

    Ok(())
}


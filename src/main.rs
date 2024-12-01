use std::{env::args, error::Error, fs::read_to_string, process::abort};

use eval::{Interpreter, Value};
use parser::parse;
use preprocess::preprocess;
use token::tokenize;

mod preprocess;
mod token;
mod parser;
mod eval;

fn main() -> Result<(), Box<dyn Error>> {
    let code_path = args().nth(1).expect("no code path passed");
    let input_path = args().nth(2);

    let code = read_to_string(code_path)?;
    let preprocessed = preprocess(&code);
    let tokens = tokenize(&preprocessed)?;
    let root = parse(tokens)?;

    let input = input_path.map(|p| read_to_string(p)).transpose()?;

    let mut interpreter = Interpreter::new();
    if let Some(input) = input {
        interpreter.set_top_level_binding("$input", Value::String(input));
    }
    interpreter.execute(&root)?;

    Ok(())
}


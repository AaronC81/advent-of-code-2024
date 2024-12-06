#![feature(let_chains)]

use std::{env::args, error::Error, fs::read_to_string, io::{stdin, stdout, Write}, path::Path, process::abort, rc::Rc};

use eval::{Interpreter, Value};
use loc::LocSource;
use parser::{parse, Node};
use token::tokenize;

mod token;
mod parser;
mod eval;
mod loc;

pub fn code_to_node(code: &str, name: &str) -> Result<Node, Box<dyn Error>> {
    let source = LocSource::new(name.to_owned(), Rc::new(code.to_owned()));
    let tokens = tokenize(&source)?;
    let root = parse(tokens)?;

    Ok(root)
}

fn main() -> Result<(), Box<dyn Error>> {
    // If no (additional) args passed, start a repl
    if args().len() == 1 {
        return repl();
    }

    let code_path = args().nth(1).expect("no code path passed");
    let input_path = args().nth(2);

    // Load input code
    let code = read_to_string(&code_path)?;
    let root = code_to_node(&code, &code_path)?;

    let input = input_path.map(|p| read_to_string(p)).transpose()?;

    let mut interpreter = Interpreter::new();
    if let Some(input) = input {
        interpreter.set_top_level_binding("$input", Value::from_string(&input));
    }
    interpreter.execute(&load_stdlib()?)?; // Load stdlib
    interpreter.execute(&root)?;

    Ok(())
}

fn repl() -> Result<(), Box<dyn Error>> {
    let mut interpreter = Interpreter::new();
    interpreter.execute(&load_stdlib()?)?;

    loop {
        print!("> ");
        stdout().flush()?;

        let mut line = String::new();
        stdin().read_line(&mut line)?;

        let node;
        match code_to_node(&line, "(repl)") {
            Ok(n) => node = n,
            Err(e) => {
                println!("Parse error: {e}");
                continue;
            }
        }

        match interpreter.execute(&node) {
            Ok(_) => {
                interpreter.print_stack_debug();
                println!("");
            },
            Err(e) => {
                println!("Execution error: {e}");
                continue;
            }
        }
    }
}

fn load_stdlib() -> Result<Node, Box<dyn Error>> {
    code_to_node(include_str!("../lib/stdlib.stk"), "(stdlib)")
}

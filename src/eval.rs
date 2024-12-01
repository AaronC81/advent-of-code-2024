use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{parser::Node, token::Atom};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Integer(isize),
    Boolean(bool),
    Array(Vec<Value>),

    Unbound(String),
    Block(Node),
}

impl Value {
    pub fn into_string(self) -> Result<String, Box<dyn Error>> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(format!("expected string, got `{self:?}`").into())
        }
    }

    pub fn into_integer(self) -> Result<isize, Box<dyn Error>> {
        match self {
            Value::Integer(i) => Ok(i),
            _ => Err(format!("expected integer, got `{self:?}`").into())
        }
    }

    pub fn into_array(self) -> Result<Vec<Value>, Box<dyn Error>> {
        match self {
            Value::Array(v) => Ok(v),
            _ => Err(format!("expected array, got `{self:?}`").into())
        }
    }

    pub fn into_boolean(self) -> Result<bool, Box<dyn Error>> {
        match self {
            Value::Boolean(b) => Ok(b),
            _ => Err(format!("expected bool, got `{self:?}`").into())
        }
    }

    pub fn into_integer_array(self) -> Result<Vec<isize>, Box<dyn Error>> {
        self.into_array()?
            .into_iter()
            .map(|item|
                match item {
                    Value::Integer(i) => Ok(i),
                    _ => Err("all items in array must be numeric".into()),
                }
            )
            .collect::<Result<Vec<_>, Box<dyn Error>>>()
    }

    pub fn into_block(self) -> Result<Node, Box<dyn Error>> {
        match self {
            Value::Block(n) => Ok(n),
            _ => Err(format!("expected block, got `{self:?}`").into())
        }
    }
}

// Representation when printed
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Boolean(b) => write!(f, "{b}"),

            Value::Array(vec) => {
                write!(f, "[")?;
                let mut is_first = true;
                for item in vec {
                    if !is_first {
                        write!(f, ", ")?;
                    }
                    is_first = false;

                    write!(f, "{}", item)?;
                }
                write!(f, "]")?;

                Ok(())
            },

            Value::Unbound(b) => write!(f, "(unbound binding: {b})"),
            Value::Block(_) => write!(f, "(block)"),
        }
    }
}

struct BindingFrame {
    bindings: HashMap<String, Value>,
}

impl BindingFrame {
    pub fn new() -> Self {
        BindingFrame {
            bindings: HashMap::new(),
        }
    }
}

pub struct Interpreter {
    binding_frames: Vec<BindingFrame>,
    stack: Vec<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            binding_frames: vec![BindingFrame::new()],
            stack: vec![],
        }
    }

    pub fn set_top_level_binding(&mut self, name: &str, value: Value) {
        self.binding_frames.first_mut().unwrap().bindings.insert(name.to_owned(), value);
    }

    pub fn execute(&mut self, node: &Node) -> Result<(), Box<dyn Error>> {
        match node {
            Node::Atom(atom) => match atom {
                Atom::LiteralInteger(i) => self.push(Value::Integer(*i)),
                Atom::Action(a) => self.execute_action(a)?,
                Atom::Binding(b) => self.push_binding(b),
            }

            Node::Sequence(ns) => {
                for n in ns {
                    self.execute(n)?;
                }
            }

            Node::Block(node) => {
                self.stack.push(Value::Block(*node.clone()));
            },
        }

        Ok(())
    }

    fn execute_block(&mut self, node: &Node) -> Result<(), Box<dyn Error>> {
        self.binding_frames.push(BindingFrame::new());
        self.execute(node)?;
        self.binding_frames.pop();

        Ok(())
    }

    fn execute_action(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        match name {
            // Core machinery
            ":" => {
                let target = self.pop()?;
                let Value::Unbound(name) = target else {
                    return Err(format!("bind target `{target}` is not a binding; has it already been assigned?").into())
                };

                let value = self.pop()?;
                self.binding_frames.last_mut().unwrap().bindings.insert(name, value);
            },
            "#" => {
                let block = self.pop()?.into_block()?;
                self.execute_block(&block)?;
            },
            "true" => self.push(Value::Boolean(true)),
            "false" => self.push(Value::Boolean(false)),
            "=" => {
                let b = self.pop()?;
                let a = self.pop()?;

                self.push(Value::Boolean(a == b));
            },
            "?" => {
                let if_truthy = self.pop()?;
                let if_falsey = self.pop()?;
                let cond = self.pop()?.into_boolean()?;

                if cond {
                    self.push(if_truthy);
                } else {
                    self.push(if_falsey);
                }
            },

            // Basic arithmetic
            "+" => {
                let b = self.pop()?.into_integer()?;
                let a = self.pop()?.into_integer()?;
                self.push(Value::Integer(a + b))
            },
            "-" => {
                let b = self.pop()?.into_integer()?;
                let a = self.pop()?.into_integer()?;
                self.push(Value::Integer(a - b))
            },
            "*" => {
                let b = self.pop()?.into_integer()?;
                let a = self.pop()?.into_integer()?;
                self.push(Value::Integer(a * b))
            },
            "/" => {
                let b = self.pop()?.into_integer()?;
                let a = self.pop()?.into_integer()?;
                self.push(Value::Integer(a / b))
            },

            // Unary arithmetic
            "neg" => {
                let i = self.pop()?.into_integer()?;
                self.push(Value::Integer(-i))
            },
            "abs" => {
                let i = self.pop()?.into_integer()?;
                self.push(Value::Integer(i.abs()))
            },

            // Basic stack
            "dup" => {
                let x = self.pop()?;
                self.push(x.clone());
                self.push(x);
            },
            "swap" => {
                let x = self.pop()?;
                let y = self.pop()?;
                self.push(x);
                self.push(y);
            },
            "drop" => { self.pop()?; },

            // Stack unpack
            "." | ".." | "..." | "...." | "....." | "......" => {
                let a = self.pop()?.into_array()?;
                let expected_count = name.len();

                if expected_count != a.len() {
                    return Err(format!("unpack action `{name}` expected {expected_count} items but got {}", a.len()).into())
                }

                for item in a {
                    self.push(item);
                }
            },

            // Array operations
            "[]" => {
                self.push(Value::Array(vec![]))
            },
            "@" => {
                let index = self.pop()?.into_integer()?;
                let mut arr = self.pop()?.into_array()?;

                if index < 0 || index >= arr.len() as isize {
                    return Err(format!("index out of range `{index}`").into())
                }

                self.push(arr.remove(index as usize));
            },
            "length" => {
                let arr = self.pop()?.into_array()?;
                self.push(Value::Integer(arr.len() as isize));
            }
            "append" => {
                let v = self.pop()?;
                let mut arr = self.pop()?.into_array()?;
                arr.push(v);

                self.push(Value::Array(arr));
            },
            "range" => {
                let end = self.pop()?.into_integer()?;
                let start = self.pop()?.into_integer()?;

                self.push(Value::Array(
                    (start..=end).map(Value::Integer).collect()
                ));
            },
            "map" => {
                let op = self.pop()?.into_block()?;
                let arr = self.pop()?.into_array()?;

                let mut new_arr = vec![];
                for item in arr {
                    self.push(item);
                    self.execute_block(&op)?;
                    new_arr.push(self.pop()?);
                }

                self.push(Value::Array(new_arr));
            },
            "++" => {
                let b = self.pop()?.into_array()?;
                let a = self.pop()?.into_array()?;

                self.push(Value::Array([a, b].concat()))
            },
            "fold" => {
                let mut acc = self.pop()?;
                let op = self.pop()?.into_block()?; // called with array item on top, then acc
                let arr = self.pop()?.into_array()?;

                for item in arr {
                    self.push(acc);
                    self.push(item);
                    self.execute_block(&op)?;
                    acc = self.pop()?;
                }

                self.push(acc);
            },
            "sort" => {
                let mut arr = self.pop()?.into_integer_array()?;
                arr.sort();

                self.push(Value::Array(
                    arr.into_iter().map(|i| Value::Integer(i)).collect()
                ))
            },
            "sum" => {
                let sum = self.pop()?.into_integer_array()?.iter().sum();
                self.push(Value::Integer(sum));
            }

            // String operations
            "lines" => {
                let s = self.pop()?.into_string()?;
                
                self.push(Value::Array(
                    s.split("\n")
                        .map(|line| Value::String(line.to_owned()))
                        .collect()
                ));
            },
            "wsplit" => {
                let s = self.pop()?.into_string()?;
                
                self.push(Value::Array(
                    s.split_ascii_whitespace()
                        .map(|line| Value::String(line.to_owned()))
                        .collect()
                ));
            },
            "int" => {
                let s = self.pop()?.into_string()?;
                match s.parse() {
                    Ok(i) => self.push(Value::Integer(i)),
                    Err(_) => return Err(format!("not convertible to integer: `{s}`").into()),
                }
            },

            // I/O
            "print" => print!("{}", self.pop()?),
            "println" => println!("{}", self.pop()?),

            // Oh no!
            _ => return Err(format!("unknown action `{name}`").into()),
        }

        Ok(())
    }

    fn push_binding(&mut self, name: &str) {
        // If the binding is already bound, retrieve its value and push it onto the stack
        // Otherwise push an unbound binding to make assignment work
        for frame in self.binding_frames.iter().rev() {
            if let Some(value) = frame.bindings.get(name) {
                self.push(value.clone());
                return;
            }
        }

        self.push(Value::Unbound(name.to_owned()))
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn pop(&mut self) -> Result<Value, Box<dyn Error>> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => Err("attempted to pop from empty stack".into())
        }
    }
}

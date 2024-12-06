use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{loc::Loc, parser::{Node, NodeKind}, token::Atom};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Char(char),
    Integer(isize),
    Boolean(bool),
    Array(Vec<Value>),

    Unbound(String),
    Block(Node),
}

impl Value {
    pub fn into_char(self) -> Result<char, ExecutionError> {
        match self {
            Value::Char(c) => Ok(c),
            _ => Err(ExecutionError::new(format!("expected character, got `{self:?}`")))
        }
    }

    pub fn into_string(self) -> Result<String, ExecutionError> {
        self.into_array()?
            .into_iter()
            .map(|item|
                match item {
                    Value::Char(c) => Ok(c),
                    _ => Err(ExecutionError::new("all items in array must be characters")),
                }
            )
            .collect()
    }

    pub fn from_string(s: &str) -> Value {
        Value::Array(s.chars().map(Value::Char).collect())
    }

    pub fn into_integer(self) -> Result<isize, ExecutionError> {
        match self {
            Value::Integer(i) => Ok(i),
            _ => Err(ExecutionError::new(format!("expected integer, got `{self:?}`")))
        }
    }

    pub fn into_array(self) -> Result<Vec<Value>, ExecutionError> {
        match self {
            Value::Array(v) => Ok(v),
            _ => Err(ExecutionError::new(format!("expected array, got `{self:?}`")))
        }
    }

    pub fn into_boolean(self) -> Result<bool, ExecutionError> {
        match self {
            Value::Boolean(b) => Ok(b),
            _ => Err(ExecutionError::new(format!("expected bool, got `{self:?}`")))
        }
    }

    pub fn into_integer_array(self) -> Result<Vec<isize>, ExecutionError> {
        self.into_array()?
            .into_iter()
            .map(|item|
                match item {
                    Value::Integer(i) => Ok(i),
                    _ => Err(ExecutionError::new("all items in array must be numeric")),
                }
            )
            .collect::<Result<Vec<_>, ExecutionError>>()
    }

    pub fn into_block(self) -> Result<Node, ExecutionError> {
        match self {
            Value::Block(n) => Ok(n),
            _ => Err(ExecutionError::new(format!("expected block, got `{self:?}`")))
        }
    }
}

// Representation when printed
impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Special case: print strings as strings
        if let Ok(s) = self.clone().into_string() && s.len() > 0 {
            return write!(f, "{s}");
        }

        match self {
            Value::Char(c) => write!(f, "{c}"),
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
    user_actions: HashMap<String, Node>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            binding_frames: vec![BindingFrame::new()],
            stack: vec![],
            user_actions: HashMap::new(),
        }
    }

    pub fn set_top_level_binding(&mut self, name: &str, value: Value) {
        self.binding_frames.first_mut().unwrap().bindings.insert(name.to_owned(), value);
    }

    pub fn execute(&mut self, node: &Node) -> Result<(), ExecutionError> {
        match &node.kind {
            NodeKind::Atom(atom) => match atom {
                Atom::LiteralInteger(i) => self.push(Value::Integer(*i)),
                Atom::LiteralChar(c) => self.push(Value::Char(*c)),
                Atom::Action(a) => self.execute_action(a).map_err(|e| e.add_loc(&node.loc))?,
                Atom::Binding(b) => self.push_binding(b),
            }

            NodeKind::Sequence(ns) => {
                for n in ns {
                    self.execute(&n)?;
                }
            }

            NodeKind::Block(node) => {
                self.stack.push(Value::Block(*node.clone()));
            },
        }

        Ok(())
    }

    fn execute_block(&mut self, node: &Node) -> Result<(), ExecutionError> {
        self.binding_frames.push(BindingFrame::new());
        self.execute(node)?;
        self.binding_frames.pop();

        Ok(())
    }

    fn execute_action(&mut self, name: &str) -> Result<(), ExecutionError> {
        match name {
            // Core machinery
            ":" => {
                let target = self.pop()?;
                let Value::Unbound(name) = target else {
                    return Err(ExecutionError::new(format!("bind target `{target}` is not a binding; has it already been assigned?")))
                };

                let value = self.pop()?;
                self.binding_frames.last_mut().unwrap().bindings.insert(name, value);
            },
            "::" => {
                let target = self.pop()?;
                let Value::Unbound(name) = target else {
                    return Err(ExecutionError::new(format!("bind target `{target}` is not a binding; has it already been assigned?")))
                };

                // Drop $ off binding name
                let name = name.strip_prefix('$').unwrap();

                let block = self.pop()?.into_block()?;

                if self.user_actions.contains_key(name) {
                    return Err(ExecutionError::new(format!("already defined an action named `{name}`")))
                }

                self.user_actions.insert(name.to_owned(), block);
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
            "|" => {
                let b = self.pop()?.into_boolean()?;
                let a = self.pop()?.into_boolean()?;

                self.push(Value::Boolean(a || b));
            },
            "&" => {
                let b = self.pop()?.into_boolean()?;
                let a = self.pop()?.into_boolean()?;

                self.push(Value::Boolean(a && b));
            },
            "!" => {
                let x = self.pop()?.into_boolean()?;
                self.push(Value::Boolean(!x));
            },
            "while" => {
                let cond = self.pop()?.into_block()?;
                let action = self.pop()?.into_block()?;

                loop {
                    self.execute_block(&cond)?;
                    let b = self.pop()?.into_boolean()?;

                    if b {
                        self.execute_block(&action)?;
                    } else {
                        break
                    }
                }
            }

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

            // Numeric comparison
            ">" => {
                let b = self.pop()?.into_integer()?;
                let a = self.pop()?.into_integer()?;
                self.push(Value::Boolean(a > b))
            },
            "<" => {
                let b = self.pop()?.into_integer()?;
                let a = self.pop()?.into_integer()?;
                self.push(Value::Boolean(a < b))
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

            // Stack unpack
            "." | ".." | "..." | "...." | "....." | "......" => {
                let a = self.pop()?.into_array()?;
                let expected_count = name.len();

                if expected_count != a.len() {
                    return Err(ExecutionError::new(format!("unpack action `{name}` expected {expected_count} items but got {}", a.len())))
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
                    return Err(ExecutionError::new(format!("index out of range `{index}`")))
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
            "shift" => {
                let mut arr = self.pop()?.into_array()?;
                let first = arr.remove(0);

                self.push(Value::Array(arr));
                self.push(first);
            },
            "break" => {
                let pred = self.pop()?.into_block()?;
                let arr = self.pop()?.into_array()?;

                // Create a new array whenever an item matches the predicate
                // but KEEP the item which satisfied the predicate
                // (That's why we're called `break` and not `split`, though I don't think it's a
                //  great name...)
                let mut result = vec![vec![]];
                for item in arr {
                    // Invoke predicate
                    self.push(item.clone());
                    self.execute_block(&pred)?;
                    let is_delimiter = self.pop()?.into_boolean()?;

                    if is_delimiter {
                        // Delimiter: add new array containing just this
                        // (Wrapped in an array so you can `map` over the broken array and treat
                        //  all items in the same way)
                        result.push(vec![item]);

                        // ...then start new list for non-delimiters
                        result.push(vec![]);
                    } else {
                        // Non-delimiter: just keep adding onto the last bit
                        result.last_mut().unwrap().push(item);
                    }
                }

                // Push result
                self.push(
                    Value::Array(result.into_iter().map(Value::Array).collect())
                );
            },
            "reverse" => {
                let mut arr = self.pop()?.into_array()?;
                arr.reverse();
                self.push(Value::Array(arr));
            }

            // String operations
            // TODO: can be implemented as more general array operations now
            "lines" => {
                let s = self.pop()?.into_string()?;
                
                self.push(Value::Array(
                    s.split("\n")
                        .map(Value::from_string)
                        .collect()
                ));
            },
            "wsplit" => {
                let s = self.pop()?.into_string()?;
                
                self.push(Value::Array(
                    s.split_ascii_whitespace()
                        .map(Value::from_string)
                        .collect()
                ));
            },
            "int" => {
                let s = self.pop()?.into_string()?;
                match s.parse() {
                    Ok(i) => self.push(Value::Integer(i)),
                    Err(_) => return Err(ExecutionError::new(format!("not convertible to integer: `{s}`"))),
                }
            },
            
            // Character operations
            "digit?" => {
                let c = self.pop()?.into_char()?;
                self.push(Value::Boolean(c.is_digit(10)));
            }

            // I/O
            "print" => print!("{}", self.pop()?),
            "println" => println!("{}", self.pop()?),
            "debug" => self.print_stack_debug(),

            // User actions
            _ if self.user_actions.contains_key(name) => {
                let body = self.user_actions.get(name).unwrap().clone();
                self.execute_block(&body)?;
            }
            
            // Oh no!
            _ => return Err(ExecutionError::new(format!("unknown action `{name}`"))),
        }

        Ok(())
    }

    pub fn print_stack_debug(&self) {
        println!("\n=== TOP ===");
        for item in self.stack.iter().rev() {
            println!("{item}");
        }
        println!("===========");
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

    fn pop(&mut self) -> Result<Value, ExecutionError> {
        match self.stack.pop() {
            Some(v) => Ok(v),
            None => Err(ExecutionError::new("attempted to pop from empty stack")),
        }
    }
}

/// Error encountered during action evaluation.
/// These start without any associated [Loc], but it is added while being passed up the chain.
/// This saves you from passing the node down unnecessarily.
#[derive(Debug, Clone)]
pub struct ExecutionError {
    message: String,
    loc: Option<Loc>,
}

impl ExecutionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), loc: None }
    }

    pub fn add_loc(self, loc: &Loc) -> Self {
        if self.loc.is_some() {
            self // Don't replace an existing loc
        } else {
            ExecutionError { loc: Some(loc.clone()), ..self }
        }
    }
}
impl Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at ")?;

        match &self.loc {
            Some(loc) => write!(f, "`{}` (position {:?} in {})", loc.contents(), loc.pos, loc.source.name)?,
            None => write!(f, "unknown position")?,
        }

        write!(f, ": {}", self.message)?;

        Ok(())
    }
}
impl Error for ExecutionError {}

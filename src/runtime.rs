use std::io::{self, Write};
use crate::tokenizer::Tokenizer;
use crate::parser::Parser;
use crate::generator::{BytecodeGenerator, OpCode, Value};
use std::collections::HashMap;
use crate::analyzer::{Analyzer, Type};

pub struct Runtime {
    tokenizer: Tokenizer,
    variables: HashMap<String, Value>,
    variable_types: HashMap<String, String>,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            tokenizer: Tokenizer::new(""),
            variables: HashMap::new(),
            variable_types: HashMap::new(),
        }
    }

    pub fn run_repl(&mut self) -> Result<(), String> {
        println!("Welcome to nair8 v0.1.0");
        println!("Type '.exit' to quit, '.load' to load a file, or enter code directly.");

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");

            let input = input.trim();

            match input {
                ".exit" => {
                    println!("Goodbye!");
                    break;
                }
                ".load" => {
                    println!("Enter file path:");
                    let mut file_path = String::new();
                    io::stdin().read_line(&mut file_path).expect("Failed to read line");
                    let file_path = file_path.trim();
                    
                    self.run_file(file_path)?;
                }
                _ => {
                    if !input.is_empty() {
                        self.process_input(input)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn run_file(&mut self, file_path: &str) -> Result<(), String> {
        match std::fs::read_to_string(file_path) {
            Ok(content) => {
                println!("Running file: {}", file_path);
                self.process_input(&content)
            }
            Err(e) => Err(format!("Error reading file '{}': {}", file_path, e)),
        }
    }

    fn process_input(&mut self, input: &str) -> Result<(), String> {
        self.tokenizer = Tokenizer::new(input);
        let tokens = self.tokenizer.tokenize()?;
        
        // Create and run parser
        let mut parser = Parser::new(tokens.clone());
        let ast = parser.parse()?;
        
        // Run type checker with existing variables
        let mut analyzer = Analyzer::new();
        
        // Only copy variables that have explicit types
        for (name, _value) in &self.variables {
            let var_type = if let Some(declared_type) = self.variable_types.get(name) {
                match declared_type.as_str() {
                    "Whole" => Type::Whole,
                    "Decimal" => Type::Decimal,
                    "Text" => Type::Text,
                    "Truth" => Type::Truth,
                    "Void" => Type::Void,
                    _ => Type::Any,
                }
            } else {
                Type::Any  // If no declared type, it's Any
            };
            analyzer.variables.insert(name.clone(), var_type);
        }
        
        analyzer.analyze(&ast)?;
        
        // Generate and run bytecode
        let mut generator = BytecodeGenerator::new();
        let bytecode = generator.generate(ast.clone())?;
        
        // Debug output
        println!("Tokens:");
        for token in tokens {
            println!("  {}", token);
        }
        
        println!("\nAST:");
        for node in &ast {
            println!("  {:?}", node);
        }
        
        println!("\nBytecode:");
        for op in &bytecode {
            println!("  {:?}", op);
        }

        self.execute_bytecode(bytecode)
    }

    fn execute_bytecode(&mut self, bytecode: Vec<OpCode>) -> Result<(), String> {
        let mut stack: Vec<Value> = Vec::new();
        let mut ip = 0;

        while ip < bytecode.len() {
            match &bytecode[ip] {
                OpCode::StoreVar(name) => {
                    let value = stack.pop().ok_or("Stack underflow")?;
                    
                    // Check if this variable has a declared type
                    if let Some(declared_type) = self.variable_types.get(name) {
                        // Skip type checking if we're storing null during declaration
                        if !matches!(value, Value::Null) {
                            let value_type = match &value {
                                Value::Number(n) => {
                                    if n.fract() == 0.0 { "Whole" } else { "Decimal" }
                                },
                                Value::String(_) => "Text",
                                Value::Boolean(_) => "Truth",
                                Value::Null => "Void",
                                Value::Object(ref class_name) => class_name,
                            };
                            
                            if declared_type != value_type {
                                return Err(format!("Type mismatch: cannot assign {} to variable of type {}", 
                                              value_type, declared_type));
                            }
                        }
                    }
                    
                    self.variables.insert(name.clone(), value);
                    Ok(())
                },
                OpCode::LoadVar(name) => {
                    // Only try to load if the variable exists
                    if let Some(value) = self.variables.get(name) {
                        stack.push(value.clone());
                        Ok(())
                    } else {
                        Err(format!("Undefined variable: {}", name))
                    }
                },
                OpCode::Push(value) => {
                    stack.push(value.clone());
                    Ok(())
                },
                OpCode::Pop => {
                    stack.pop();
                    Ok(())
                },
                OpCode::Duplicate => {
                    if let Some(value) = stack.last() {
                        stack.push(value.clone());
                    }
                    Ok(())
                },
                OpCode::Add => {
                    let b = stack.pop().ok_or("Stack underflow")?;
                    let a = stack.pop().ok_or("Stack underflow")?;
                    stack.push(self.binary_op(a, b, |x, y| x + y)?);
                    Ok(())
                },
                OpCode::Subtract => {
                    let b = stack.pop().ok_or("Stack underflow")?;
                    let a = stack.pop().ok_or("Stack underflow")?;
                    stack.push(self.binary_op(a, b, |x, y| x - y)?);
                    Ok(())
                },
                OpCode::Multiply => {
                    let b = stack.pop().ok_or("Stack underflow")?;
                    let a = stack.pop().ok_or("Stack underflow")?;
                    stack.push(self.binary_op(a, b, |x, y| x * y)?);
                    Ok(())
                },
                OpCode::Divide => {
                    let b = stack.pop().ok_or("Stack underflow")?;
                    let a = stack.pop().ok_or("Stack underflow")?;
                    stack.push(self.binary_op(a, b, |x, y| x / y)?);
                    Ok(())
                },
                OpCode::Jump(target) => {
                    ip = *target;
                    Ok(())
                },
                OpCode::JumpIfFalse(target) => {
                    if let Some(Value::Boolean(false)) = stack.last() {
                        ip = *target;
                        Ok(())
                    } else {
                        Ok(())
                    }
                },
                OpCode::Call(name, arg_count) => {
                    let mut args = Vec::new();
                    // Pop arguments in reverse order
                    for _ in 0..*arg_count {
                        if let Some(arg) = stack.pop() {
                            args.insert(0, arg);
                        }
                    }

                    match name.as_str() {
                        "show" => {
                            // Built-in show function
                            if let Some(value) = args.get(0) {
                                println!("{}", value);
                            }
                            stack.push(Value::Null); // show returns null
                        },
                        _ => {
                            return Err(format!("Unknown function: {}", name));
                        }
                    }
                    Ok(())
                },
                OpCode::Return => {
                    // TODO: Implement return
                    break;
                },
                OpCode::NewObject(_class_name) => {
                    // TODO: Implement object creation
                    return Err("Object creation not implemented yet".to_string());
                },
                OpCode::GetProperty(_name) => {
                    // TODO: Implement property access
                    return Err("Property access not implemented yet".to_string());
                },
                OpCode::SetProperty(_name) => {
                    // TODO: Implement property setting
                    return Err("Property setting not implemented yet".to_string());
                },
                OpCode::CheckType(type_name) => {
                    if let Some(var_name) = self.get_next_var_name(&bytecode[ip+1..]) {
                        self.variable_types.insert(var_name.clone(), type_name.clone());
                    }
                    Ok(())
                },
                OpCode::Cast(type_name) => {
                    if let Some(value) = stack.pop() {
                        let new_value = match (value.clone(), type_name.as_str()) {
                            (Value::Number(n), "Whole") => {
                                Value::Number(n.floor())
                            },
                            (Value::Number(n), "Decimal") => {
                                Value::Number(n)
                            },
                            (Value::String(s), "Text") => {
                                Value::String(s)
                            },
                            (Value::Boolean(b), "Truth") => {
                                Value::Boolean(b)
                            },
                            _ => return Err(format!("Cannot cast {:?} to {}", value, type_name)),
                        };
                        stack.push(new_value);
                    }
                    Ok(())
                },
                OpCode::Concat => {
                    let b = stack.pop().ok_or("Stack underflow")?;
                    let a = stack.pop().ok_or("Stack underflow")?;
                    stack.push(self.concat_values(a, b)?);
                    Ok(())
                },
                OpCode::Interpolate(part_count) => {
                    let mut result = String::new();
                    for _ in 0..*part_count {
                        if let Some(Value::String(part)) = stack.pop() {
                            result = part + &result;
                        }
                    }
                    stack.push(Value::String(result));
                    Ok(())
                },
                OpCode::CheckAssignmentType => {
                    let _var_value = stack.pop().ok_or("Stack underflow")?;
                    let new_value = stack.last().ok_or("Stack underflow")?;
                    
                    if let Some(var_name) = self.get_next_var_name(&bytecode[ip+1..]) {
                        // Only check type if the variable has an explicit type declaration
                        if let Some(declared_type) = self.variable_types.get(&var_name) {
                            let new_type = match new_value {
                                Value::Number(n) => {
                                    if n.fract() == 0.0 { "Whole" } else { "Decimal" }
                                },
                                Value::String(_) => "Text",
                                Value::Boolean(_) => "Truth",
                                Value::Null => "Void",
                                Value::Object(ref class_name) => class_name,
                            };

                            if declared_type != new_type {
                                return Err(format!("Type mismatch: cannot assign {} to variable of type {}", 
                                              new_type, declared_type));
                            }
                        }
                        // If variable doesn't have a declared type, allow any assignment
                    }
                    Ok(())
                },
            }?;
            ip += 1;
        }
        Ok(())
    }

    fn get_next_var_name(&self, upcoming_ops: &[OpCode]) -> Option<String> {
        for op in upcoming_ops {
            if let OpCode::StoreVar(name) = op {
                return Some(name.clone());
            }
        }
        None
    }

    // Helper methods for the Runtime impl
    fn binary_op<F>(&self, a: Value, b: Value, op: F) -> Result<Value, String>
    where
        F: Fn(f64, f64) -> f64,
    {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(op(x, y))),
            _ => Err("Invalid operands for arithmetic operation".to_string()),
        }
    }

    fn concat_values(&self, a: Value, b: Value) -> Result<Value, String> {
        match (a, b) {
            (Value::String(s1), Value::String(s2)) => Ok(Value::String(s1 + &s2)),
            _ => Err("Can only concatenate strings".to_string()),
        }
    }
}


fn main() -> Result<(), String> {
    let mut runtime = Runtime::new();
    runtime.run_repl()
}

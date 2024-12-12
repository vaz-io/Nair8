use std::io::{self, Write};
use crate::tokenizer::Tokenizer;
use crate::parser::Parser;
use crate::generator::{BytecodeGenerator, OpCode, Value};
use std::collections::HashMap;

pub struct Runtime {
    tokenizer: Tokenizer,
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            tokenizer: Tokenizer::new(""),
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
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
                    
                    match std::fs::read_to_string(file_path) {
                        Ok(content) => {
                            println!("Loading file: {}", file_path);
                            self.process_input(&content)?;
                        }
                        Err(e) => println!("Error loading file: {}", e),
                    }
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

    fn process_input(&mut self, input: &str) -> Result<(), String> {
        self.tokenizer = Tokenizer::new(input);
        let tokens = self.tokenizer.tokenize()?;
        
        // Create and run parser
        let mut parser = Parser::new(tokens.clone());
        let ast = parser.parse()?;
        
        // Generate bytecode
        let mut generator = BytecodeGenerator::new();
        let bytecode = generator.generate(ast.clone())?;
        
        // Debug output
        println!("Tokens:");
        for token in tokens {
            println!("  {}", token);
        }
        
        println!("\nAST:");
        for node in ast {
            println!("  {:?}", node);
        }

        println!("\nBytecode:");
        for instruction in &bytecode {
            println!("  {:?}", instruction);
        }

        // Execute bytecode
        self.execute_bytecode(bytecode)?;

        Ok(())
    }

    fn execute_bytecode(&mut self, bytecode: Vec<OpCode>) -> Result<(), String> {
        let mut stack: Vec<Value> = Vec::new();
        let mut variables: HashMap<String, Value> = HashMap::new();
        let mut ip = 0;  // instruction pointer

        while ip < bytecode.len() {
            match &bytecode[ip] {
                OpCode::Push(value) => {
                    stack.push(value.clone());
                },
                OpCode::Pop => {
                    stack.pop();
                },
                OpCode::LoadVar(name) => {
                    if let Some(value) = variables.get(name) {
                        stack.push(value.clone());
                    } else {
                        return Err(format!("Undefined variable: {}", name));
                    }
                },
                OpCode::StoreVar(name) => {
                    if let Some(value) = stack.pop() {
                        variables.insert(name.clone(), value);
                    }
                },
                // Add more opcodes as needed...
            }
            ip += 1;
        }

        Ok(())
    }
}


fn main() -> Result<(), String> {
    let mut runtime = Runtime::new();
    runtime.run()
}

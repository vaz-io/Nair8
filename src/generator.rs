use crate::parser::Node;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum OpCode {
    // Stack Operations
    Push(Value),
    Pop,
    Duplicate,
    
    // Variables
    LoadVar(String),
    StoreVar(String),
    
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    
    // Control Flow
    Jump(usize),
    JumpIfFalse(usize),
    Call(String, usize),  // function name, arg count
    Return,
    
    // Objects
    NewObject(String),    // class name
    GetProperty(String),  // property name
    SetProperty(String),  // property name
    
    // Types
    CheckType(String),    // type name
    Cast(String),        // type name
    
    // String Operations
    Concat,
    Interpolate(usize),  // number of parts
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Object(String),      // class name
}

pub struct BytecodeGenerator {
    instructions: Vec<OpCode>,
    constants: Vec<Value>,
    variables: HashMap<String, usize>,
    current_scope: usize,
    loop_starts: Vec<usize>,
    loop_ends: Vec<usize>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        BytecodeGenerator {
            instructions: Vec::new(),
            constants: Vec::new(),
            variables: HashMap::new(),
            current_scope: 0,
            loop_starts: Vec::new(),
            loop_ends: Vec::new(),
        }
    }

    pub fn generate(&mut self, nodes: Vec<Node>) -> Result<Vec<OpCode>, String> {
        for node in nodes {
            self.generate_node(node)?;
        }
        Ok(self.instructions.clone())
    }

    fn generate_node(&mut self, node: Node) -> Result<(), String> {
        match node {
            Node::VariableDecl { name, type_annotation, initializer } => {
                // Generate type check if there's a type annotation
                if let Some(type_node) = type_annotation {
                    self.generate_type_annotation(*type_node)?;
                }

                // Generate initializer if present
                if let Some(init) = initializer {
                    self.generate_node(*init)?;
                } else {
                    self.instructions.push(OpCode::Push(Value::Null));
                }

                self.variables.insert(name.clone(), self.current_scope);
                self.instructions.push(OpCode::StoreVar(name));
            },

            Node::StringInterpolation { parts } => {
                for part in parts {
                    self.generate_node(part)?;
                }
                self.instructions.push(OpCode::Interpolate(parts.len()));
            },

            Node::Literal(token_type) => {
                let value = match token_type {
                    crate::tokenizer::TokenType::Number(n) => Value::Number(n),
                    crate::tokenizer::TokenType::String(s) => Value::String(s),
                    crate::tokenizer::TokenType::Boolean(b) => Value::Boolean(b),
                    crate::tokenizer::TokenType::Null => Value::Null,
                    _ => return Err("Unsupported literal type".to_string()),
                };
                self.instructions.push(OpCode::Push(value));
            },

            Node::Variable(name) => {
                self.instructions.push(OpCode::LoadVar(name));
            },

            // Add more node types as needed...
            _ => return Err(format!("Unsupported node type: {:?}", node)),
        }
        Ok(())
    }

    fn generate_type_annotation(&mut self, type_node: Node) -> Result<(), String> {
        match type_node {
            Node::TypeAnnotation(type_name) => {
                self.instructions.push(OpCode::CheckType(type_name));
                Ok(())
            },
            _ => Err("Expected type annotation".to_string()),
        }
    }
}

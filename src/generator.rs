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
    CheckAssignmentType,
    ConvertToString,
    Show,
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Object(String),      // class name
}

// Add Display implementation for Value
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Object(name) => write!(f, "[object {}]", name),
        }
    }
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
            self.generate_node(&node)?;
        }
        Ok(self.instructions.clone())
    }

    fn generate_node(&mut self, node: &Node) -> Result<(), String> {
        match node {
            Node::VariableDecl { name, type_annotation, initializer } => {
                if let Some(init) = initializer {
                    // Generate code for initializer
                    self.generate_node(init)?;
                } else {
                    // No initializer, push null
                    self.emit(OpCode::Push(Value::Null));
                }

                // If there's a type annotation, check it
                if let Some(type_node) = type_annotation {
                    if let Node::TypeAnnotation(type_name) = &**type_node {
                        self.emit(OpCode::CheckType(type_name.clone()));
                    }
                }

                // Store the variable
                self.emit(OpCode::StoreVar(name.clone()));
                Ok(())
            },

            Node::Assignment { name, value } => {
                // Generate code for the value first
                self.generate_node(value)?;

                // Only generate LoadVar and CheckAssignmentType if the variable exists
                if self.variables.contains_key(name) {
                    self.emit(OpCode::LoadVar(name.to_string()));
                    self.emit(OpCode::CheckAssignmentType);
                }
                
                // Store the variable
                self.emit(OpCode::StoreVar(name.to_string()));
                
                // Track the variable if it's new
                if !self.variables.contains_key(name) {
                    self.variables.insert(name.clone(), self.variables.len());
                }
                
                Ok(())
            },

            Node::Binary { left, operator, right } => {
                self.generate_node(left)?;
                self.generate_node(right)?;
                
                let opcode = match operator {
                    crate::tokenizer::TokenType::Plus => OpCode::Add,
                    crate::tokenizer::TokenType::Minus => OpCode::Subtract,
                    crate::tokenizer::TokenType::Multiply => OpCode::Multiply,
                    crate::tokenizer::TokenType::Divide => OpCode::Divide,
                    _ => return Err("Unsupported binary operator".to_string()),
                };
                self.instructions.push(opcode);
                Ok(())
            },

            Node::Call { callee, args } => {
                // Generate code for arguments first
                for arg in args {
                    self.generate_node(arg)?;
                }
                
                // Generate code for the callee
                match **callee {
                    Node::Variable(ref name) => {
                        self.emit(OpCode::Call(name.clone(), args.len()));
                        Ok(())
                    },
                    _ => Err("Only direct function calls are supported".to_string()),
                }
            },

            Node::ShowStmt(expr) => {
                self.generate_node(expr)?;
                self.emit(OpCode::Show);
                Ok(())
            },

            Node::Block(statements) => {
                for stmt in statements {
                    self.generate_node(stmt)?;
                }
                Ok(())
            },

            Node::WhenStmt { condition, then_branch, else_branch } => {
                // Generate condition code
                self.generate_node(condition)?;
                
                // Add jump-if-false instruction (we'll patch the jump address later)
                let jump_if_false_pos = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0));
                
                // Generate then branch
                self.generate_node(then_branch)?;
                
                if let Some(else_branch) = else_branch {
                    // Add jump instruction to skip else branch (we'll patch the address later)
                    let jump_pos = self.instructions.len();
                    self.instructions.push(OpCode::Jump(0));
                    
                    // Patch the jump-if-false address
                    let else_start = self.instructions.len();
                    if let OpCode::JumpIfFalse(ref mut addr) = self.instructions[jump_if_false_pos] {
                        *addr = else_start;
                    }
                    
                    // Generate else branch
                    self.generate_node(else_branch)?;
                    
                    // Patch the jump address
                    let after_else = self.instructions.len();
                    if let OpCode::Jump(ref mut addr) = self.instructions[jump_pos] {
                        *addr = after_else;
                    }
                } else {
                    // Patch the jump-if-false address
                    let after_then = self.instructions.len();
                    if let OpCode::JumpIfFalse(ref mut addr) = self.instructions[jump_if_false_pos] {
                        *addr = after_then;
                    }
                }
                Ok(())
            },

            Node::LoopStmt { condition, body } => {
                let loop_start = self.instructions.len();
                
                // Generate condition
                self.generate_node(condition)?;
                
                // Add conditional jump to exit loop
                let jump_if_false_pos = self.instructions.len();
                self.instructions.push(OpCode::JumpIfFalse(0));
                
                // Generate loop body
                self.generate_node(body)?;
                
                // Add jump back to start
                self.instructions.push(OpCode::Jump(loop_start));
                
                // Patch the exit jump address
                let after_loop = self.instructions.len();
                if let OpCode::JumpIfFalse(ref mut addr) = self.instructions[jump_if_false_pos] {
                    *addr = after_loop;
                }
                Ok(())
            },

            Node::Get { object, name } => {
                self.generate_node(object)?;
                self.emit(OpCode::GetProperty(name.clone()));
                Ok(())
            },

            Node::New { class_name, args } => {
                for arg in args {
                    self.generate_node(arg)?;
                }
                self.emit(OpCode::NewObject(class_name.clone()));
                Ok(())
            },

            Node::StringInterpolation { parts } => {
                self.generate_string_interpolation(parts)?;
                self.emit(OpCode::Interpolate(parts.len()));
                Ok(())
            },

            Node::Literal(value) => {
                match value {
                    Value::Number(n) => self.emit(OpCode::Push(Value::Number(*n))),
                    Value::String(s) => self.emit(OpCode::Push(Value::String(s.clone()))),
                    Value::Boolean(b) => self.emit(OpCode::Push(Value::Boolean(*b))),
                    Value::Null => self.emit(OpCode::Push(Value::Null)),
                    Value::Object(name) => self.emit(OpCode::Push(Value::Object(name.clone()))),
                }
                Ok(())
            },

            Node::Variable(name) => {
                self.emit(OpCode::LoadVar(name.clone()));
                Ok(())
            },

            // Add more node types as needed...
            _ => Err(format!("Unsupported node type: {:?}", node)),
        }
    }

    fn generate_type_annotation(&mut self, type_node: Node) -> Result<(), String> {
        match type_node {
            Node::TypeAnnotation(type_name) => {
                // For variable declarations with no initializer, we'll push null first
                self.instructions.push(OpCode::Push(Value::Null));
                self.instructions.push(OpCode::CheckType(type_name));
                Ok(())
            },
            _ => Err("Expected type annotation".to_string()),
        }
    }

    fn emit(&mut self, opcode: OpCode) {
        self.instructions.push(opcode);
    }

    fn generate_assignment(&mut self, name: &str, value: &Node) -> Result<(), String> {
        // Generate code for the value first
        self.generate_node(value)?;

        // For assignments, we only need LoadVar if the variable exists
        if self.variables.contains_key(name) {
            self.emit(OpCode::LoadVar(name.to_string()));
            self.emit(OpCode::CheckAssignmentType);
        }
        
        // Store the result
        self.emit(OpCode::StoreVar(name.to_string()));
        Ok(())
    }

    fn generate_string_interpolation(&mut self, parts: &[Node]) -> Result<(), String> {
        for part in parts {
            match part {
                Node::Literal(Value::String(s)) => {
                    self.emit(OpCode::Push(Value::String(s.clone())));
                },
                Node::Variable(name) => {
                    self.emit(OpCode::LoadVar(name.clone()));
                    self.emit(OpCode::ConvertToString);
                },
                _ => self.generate_node(part)?,
            }
            
            if parts.len() > 1 {
                self.emit(OpCode::Concat);
            }
        }
        Ok(())
    }
}

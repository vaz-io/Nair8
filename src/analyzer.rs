use crate::parser::Node;
use std::collections::HashMap;
use crate::generator::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Whole,      // Integer type
    Decimal,    // Float type
    Text,       // String type
    Truth,      // Boolean type
    Void,       // Null type
    Error,      // Error type
    Any,        // Any type (used for variables without type annotation)
    Object,     // Object type
    List(Box<Type>),
    Map { key: Box<Type>, value: Box<Type> },
    Promise(Box<Type>),
}

pub struct Analyzer {
    pub variables: HashMap<String, Type>,
    current_scope: Vec<HashMap<String, Type>>,
    current_var_type: Option<Type>,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            variables: HashMap::new(),
            current_scope: vec![HashMap::new()],
            current_var_type: None,
        }
    }

    pub fn analyze(&mut self, nodes: &[Node]) -> Result<(), String> {
        for node in nodes {
            self.check_node(node)?;
        }
        Ok(())
    }

    fn check_node(&mut self, node: &Node) -> Result<Type, String> {
        match node {
            Node::VariableDecl { name, type_annotation, initializer } => {
                let declared_type = if let Some(type_node) = type_annotation {
                    let typ = self.type_from_annotation(type_node)?;
                    self.current_var_type = Some(typ.clone());
                    typ
                } else {
                    Type::Any
                };

                if let Some(init) = initializer {
                    let init_type = self.check_node(init)?;
                    self.check_type_compatibility(&declared_type, &init_type)?;
                }

                self.current_var_type = None;
                self.variables.insert(name.clone(), declared_type.clone());
                Ok(declared_type)
            },

            Node::Literal(value) => {
                Ok(match value {
                    Value::Number(_) => Type::Whole,
                    Value::String(_) => Type::Text,
                    Value::Boolean(_) => Type::Truth,
                    Value::Null => Type::Void,
                    Value::Object(_) => Type::Object,
                })
            },

            Node::Variable(name) => {
                self.variables.get(name)
                    .cloned()
                    .or(Some(Type::Any))
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            },

            Node::Binary { left, operator, right } => {
                let left_type = self.check_node(left)?;
                let right_type = self.check_node(right)?;
                
                use crate::tokenizer::TokenType;
                match operator {
                    TokenType::Plus | TokenType::Minus | 
                    TokenType::Multiply | TokenType::Divide => {
                        match (&left_type, &right_type) {
                            (Type::Whole, Type::Whole) => Ok(Type::Whole),
                            (Type::Decimal, _) | (_, Type::Decimal) => Ok(Type::Decimal),
                            (Type::Text, Type::Text) if matches!(operator, TokenType::Plus) => {
                                Ok(Type::Text)
                            },
                            _ => Err(format!("Invalid operand types for binary operation: {:?} and {:?}", 
                                           left_type, right_type))
                        }
                    },
                    _ => Err("Unsupported operator".to_string()),
                }
            },

            Node::ShowStmt(expr) => {
                self.check_node(expr)?;
                Ok(Type::Void)
            },

            Node::StringInterpolation { parts } => {
                for part in parts {
                    let part_type = self.check_node(part)?;
                    if !matches!(part_type, Type::Text) {
                        return Err("String interpolation parts must be convertible to text".to_string());
                    }
                }
                Ok(Type::Text)
            },

            Node::Assignment { name, value } => {
                let value_type = self.check_node(value)?;
                
                if let Some(var_type) = self.variables.get(name) {
                    if var_type != &Type::Any && var_type != &value_type {
                        return Err(format!("Type mismatch: cannot assign {:?} to variable of type {:?}", 
                                       value_type, var_type));
                    }
                } else {
                    self.variables.insert(name.clone(), Type::Any);
                }

                Ok(value_type)
            },

            Node::MappingLiteral { entries } => {
                if entries.is_empty() {
                    return Ok(Type::Map {
                        key: Box::new(Type::Text),
                        value: Box::new(Type::Any),
                    });
                }
                
                // Get the expected value type from the variable declaration
                let expected_value_type = if let Some(Type::Map { value, .. }) = &self.current_var_type {
                    Some(value.as_ref().clone())
                } else {
                    None
                };
                
                // Check all entries
                for (param_name, param_type, value) in entries {
                    let value_type = self.check_node(value)?;
                    
                    // If parameter has explicit type, check it
                    if let Some(type_node) = param_type {
                        let declared_type = self.check_node(&type_node)?;
                        self.check_type_compatibility(&declared_type, &value_type)?;
                    }
                    
                    // If mapping has declared value type, check against that
                    if let Some(expected) = &expected_value_type {
                        self.check_type_compatibility(expected, &value_type)?;
                    }
                }
                
                Ok(Type::Map {
                    key: Box::new(Type::Text),
                    value: Box::new(expected_value_type.unwrap_or(Type::Any)),
                })
            },

            _ => Ok(Type::Any), // Temporarily allow other nodes
        }
    }

    fn type_from_annotation(&self, node: &Node) -> Result<Type, String> {
        match node {
            Node::TypeAnnotation(type_name) => {
                match type_name.as_str() {
                    "Whole" => Ok(Type::Whole),
                    "Decimal" => Ok(Type::Decimal),
                    "Text" => Ok(Type::Text),
                    "Truth" => Ok(Type::Truth),
                    "Void" => Ok(Type::Void),
                    "Error" => Ok(Type::Error),
                    "Object" => Ok(Type::Object),
                    _ => Err(format!("Unknown type: {}", type_name)),
                }
            },
            _ => Err("Invalid type annotation".to_string()),
        }
    }

    fn check_type_compatibility(&self, expected: &Type, actual: &Type) -> Result<(), String> {
        if expected == actual || expected == &Type::Any {
            Ok(())
        } else {
            Err(format!("Type mismatch: expected {:?}, got {:?}", expected, actual))
        }
    }

    fn check_mapping(&mut self, entries: &[(String, Option<Node>, Node)]) -> Result<Type, String> {
        for (_param_name, param_type, value) in entries {
            // ... rest of the implementation
        }
        Ok(Type::Map {
            key: Box::new(Type::Text),
            value: Box::new(Type::Any),
        })
    }
}

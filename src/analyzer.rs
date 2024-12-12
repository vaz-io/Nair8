use crate::parser::Node;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Whole,    // Integer type
    Decimal,     // Float type
    Text,     // String type
    Truth,    // Boolean type
    Void,  // Null type
    Error,    // Error type
    Person,   // Person object type
    BaseEntity, // Base entity type
    Any,      // Any type (used for variables without type annotation)
    List(Box<Type>),
    Map { key: Box<Type>, value: Box<Type> },
    Promise(Box<Type>),
}

pub struct Analyzer {
    pub variables: HashMap<String, Type>,
    current_scope: Vec<HashMap<String, Type>>,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            variables: HashMap::new(),
            current_scope: vec![HashMap::new()],
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
                    self.type_from_annotation(type_node)?
                } else {
                    Type::Any
                };

                self.variables.insert(name.clone(), declared_type.clone());

                if let Some(init) = initializer {
                    let init_type = self.check_node(init)?;
                    self.check_type_compatibility(&declared_type, &init_type)?;
                }

                Ok(declared_type)
            },

            Node::Literal(token_type) => {
                use crate::tokenizer::TokenType;
                Ok(match token_type {
                    TokenType::Number(n) => {
                        if n.fract() == 0.0 { Type::Whole } else { Type::Decimal }
                    },
                    TokenType::String(_) => Type::Text,
                    TokenType::Boolean(_) => Type::Truth,
                    TokenType::Null => Type::Void,
                    _ => return Err("Invalid literal type".to_string()),
                })
            },

            Node::Variable(name) => {
                self.variables.get(name)
                    .cloned()
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
                    if !matches!(part_type, Type::Text | Type::Whole | Type::Decimal | 
                               Type::Truth | Type::Void) {
                        return Err(format!("Cannot interpolate type {:?} in string", part_type));
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
                    "Person" => Ok(Type::Person),
                    "BaseEntity" => Ok(Type::BaseEntity),
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
}

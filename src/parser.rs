use crate::{analyzer::Type, tokenizer::{Token, TokenType}};
use crate::generator::Value;

#[derive(Debug, Clone)]
pub enum Node {
    // Declarations
    VariableDecl {
        name: String,
        type_annotation: Option<Box<Node>>,
        initializer: Option<Box<Node>>,
    },
    TaskDecl {
        name: String,
        params: Vec<Node>,
        return_type: Option<Box<Node>>,
        body: Box<Node>,
    },
    ObjectDecl {
        name: String,
        base: Option<Box<Node>>,
        constructor: Option<Box<Node>>,
        methods: Vec<Node>,
    },

    // Statements
    Block(Vec<Node>),
    ExpressionStmt(Box<Node>),
    ReturnStmt(Box<Node>),
    WhenStmt {
        condition: Box<Node>,
        then_branch: Box<Node>,
        else_branch: Option<Box<Node>>,
    },
    LoopStmt {
        condition: Box<Node>,
        body: Box<Node>,
    },
    ShowStmt(Box<Node>),
    RaiseStmt {
        message: Box<Node>,
        error_type: Box<Node>,
    },

    // Expressions
    Binary {
        left: Box<Node>,
        operator: TokenType,
        right: Box<Node>,
    },
    Call {
        callee: Box<Node>,
        args: Vec<Node>,
    },
    Get {
        object: Box<Node>,
        name: String,
    },
    Literal(Value),
    Variable(String),
    Assignment {
        name: String,
        value: Box<Node>,
    },
    New {
        class_name: String,
        args: Vec<Node>,
    },

    // Types
    TypeAnnotation(String),
    ListType {
        element_type: Box<Node>,
    },
    MappingType {
        key_type: Box<Node>,
        value_type: Box<Node>,
    },
    StringInterpolation {
        parts: Vec<Node>,
    },
    PromiseType {
        value_type: Box<Node>,
    },
    ArrayLiteral {
        elements: Vec<Node>,
        type_annotation: Option<Box<Node>>,
    },
    ObjectLiteral {
        fields: Vec<(String, Node)>,
    },
    MethodCall {
        object: Box<Node>,
        method: String,
        args: Vec<Node>,
    },
    WithExpr {
        base: Box<Node>,
        args: Vec<Node>,
    },
    UsingExpr {
        base: Box<Node>,
        args: Vec<Node>,
    },
    MatchExpr {
        value: Box<Node>,
        cases: Vec<(Node, Node)>,
    },
    EmitStmt(Box<Node>),
    AwaitExpr {
        value: Box<Node>,
    },
    PropertyAccess {
        object: Box<Node>,
        property: String,
    },
    MappingLiteral {
        entries: Vec<(String, Option<Node>, Node)>, // (param_name, optional_type, value)
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Node, String> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();

            if self.match_token(&[TokenType::As]) {
                let type_node = self.type_annotation()?;
                
                if matches!(type_node, Node::MappingType { .. }) {
                    self.consume(&TokenType::Includes, "Expected 'includes' after mapping type")?;
                    
                    let initializer = Some(Box::new(self.mapping_initializer()?));
                    Ok(Node::VariableDecl {
                        name,
                        type_annotation: Some(Box::new(type_node)),
                        initializer,
                    })
                } else {
                    if self.match_token(&[TokenType::Is]) {
                        // Regular assignment without type annotation
                        Ok(Node::VariableDecl {
                            name,
                            type_annotation: None,
                            initializer: Some(Box::new(self.expression()?)),
                        })
                    } else {
                        Err("Expected 'as' or 'is' after identifier".to_string())
                    }
                }
            } else if self.match_token(&[TokenType::Is]) {
                // Regular assignment without type annotation
                Ok(Node::VariableDecl {
                    name,
                    type_annotation: None,
                    initializer: Some(Box::new(self.expression()?)),
                })
            } else {
                Err("Expected 'as' or 'is' after identifier".to_string())
            }
        } else {
            Err("Expected identifier".to_string())
        }
    }

    fn Task_declaration(&mut self) -> Result<Node, String> {
        let name = self.consume_identifier("Expected Task name")?;
        
        let mut params = Vec::new();
        if self.match_token(&[TokenType::Requires]) {
            params = self.parameter_list()?;
        }

        let return_type = if self.match_token(&[TokenType::Returns, TokenType::Returning]) {
            Some(Box::new(self.type_annotation()?))
        } else {
            None
        };

        self.consume(&TokenType::Colon, "Expected ':' after Task declaration")?;
        let body = Box::new(self.block()?);

        Ok(Node::TaskDecl {
            name,
            params,
            return_type,
            body,
        })
    }

    fn object_declaration(&mut self) -> Result<Node, String> {
        let name = self.consume_identifier("Expected object name")?;
        
        let base = if self.match_token(&[TokenType::Extends]) {
            Some(Box::new(Node::TypeAnnotation(self.consume_identifier("Expected base class name")?)))
        } else {
            None
        };

        self.consume(&TokenType::Colon, "Expected ':' after object declaration")?;

        let mut methods = Vec::new();
        let mut constructor = None;

        while !self.check(&TokenType::EOF) && !self.is_at_end() {
            if self.match_token(&[TokenType::Build]) {
                if constructor.is_some() {
                    return Err("Object can only have one constructor".to_string());
                }
                constructor = Some(Box::new(self.constructor_declaration()?));
            } else if self.match_token(&[TokenType::Task]) {
                methods.push(self.Task_declaration()?);
            } else {
                break;
            }
        }

        Ok(Node::ObjectDecl {
            name,
            base,
            constructor,
            methods,
        })
    }

    fn parameter_list(&mut self) -> Result<Vec<Node>, String> {
        let mut params = Vec::new();
        
        loop {
            let name = self.consume_identifier("Expected parameter name")?;
            let type_annotation = if self.match_token(&[TokenType::As]) {
                Some(Box::new(self.type_annotation()?))
            } else {
                None
            };
            
            params.push(Node::VariableDecl {
                name,
                type_annotation,
                initializer: None,
            });

            if !self.match_token(&[TokenType::Comma]) {
                break;
            }
        }

        Ok(params)
    }

    fn type_annotation(&mut self) -> Result<Node, String> {
        match &self.peek().token_type {
            TokenType::TypeMapping => {
                self.advance();
                
                // Check if there's an explicit type
                if self.match_token(&[TokenType::Of]) {
                    let value_type = Box::new(self.type_annotation()?);
                    Ok(Node::MappingType {
                        key_type: Box::new(Node::TypeAnnotation("Text".to_string())),
                        value_type,
                    })
                } else {
                    // Default to Any
                    Ok(Node::MappingType {
                        key_type: Box::new(Node::TypeAnnotation("Text".to_string())),
                        value_type: Box::new(Node::TypeAnnotation("Any".to_string())),
                    })
                }
            },
            TokenType::TypeText => {
                self.advance();
                Ok(Node::TypeAnnotation("Text".to_string()))
            },
            TokenType::TypeWhole => {
                self.advance();
                Ok(Node::TypeAnnotation("Whole".to_string()))
            },
            TokenType::TypeDecimal => {
                self.advance();
                Ok(Node::TypeAnnotation("Decimal".to_string()))
            },
            TokenType::TypeLogic => {
                self.advance();
                Ok(Node::TypeAnnotation("Logic".to_string()))
            },
            TokenType::TypeNothing => {
                self.advance();
                Ok(Node::TypeAnnotation("Nothing".to_string()))
            },
            TokenType::TypeList => {
                self.advance();
                if self.match_token(&[TokenType::OpenBracket]) {
                    let element_type = Box::new(self.type_annotation()?);
                    self.consume(&TokenType::CloseBracket, "Expected ']' after type parameter")?;
                    Ok(Node::ListType { element_type })
                } else {
                    Ok(Node::TypeAnnotation("List".to_string()))
                }
            },
            TokenType::TypePromise => {
                self.advance();
                if self.match_token(&[TokenType::OpenBracket]) {
                    let value_type = Box::new(self.type_annotation()?);
                    self.consume(&TokenType::CloseBracket, "Expected ']' after type parameter")?;
                    Ok(Node::PromiseType { value_type })
                } else {
                    Ok(Node::TypeAnnotation("Promise".to_string()))
                }
            },
            TokenType::TypeAny => {
                self.advance();
                Ok(Node::TypeAnnotation("Any".to_string()))
            },
            TokenType::TypeNumber => {
                self.advance();
                Ok(Node::TypeAnnotation("Number".to_string()))
            },
            TokenType::TypeError => {
                self.advance();
                Ok(Node::TypeAnnotation("Error".to_string()))
            },
            _ => Err("Expected type name".to_string()),
        }
    }

    fn block(&mut self) -> Result<Node, String> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() && !self.check(&TokenType::EOF) {
            statements.push(self.declaration()?);
        }
        
        Ok(Node::Block(statements))
    }

    fn constructor_declaration(&mut self) -> Result<Node, String> {
        self.consume(&TokenType::Defaults, "Expected 'defaults' after 'build'")?;
        let params = self.parameter_list()?;
        self.consume(&TokenType::Colon, "Expected ':' after constructor parameters")?;
        let body = Box::new(self.block()?);
        
        Ok(Node::TaskDecl {
            name: "build".to_string(),
            params,
            return_type: None,
            body,
        })
    }

    fn expression(&mut self) -> Result<Node, String> {
        match self.peek().token_type {
            TokenType::Identifier(_) => {
                let name = self.consume_identifier("Expected identifier")?;
                Ok(Node::Variable(name))
            },
            TokenType::String(_) => self.string_literal(),
            TokenType::Number(_) => {
                if let TokenType::Number(n) = self.peek().token_type {
                    self.advance();
                    Ok(Node::Literal(Value::Number(n)))
                } else {
                    Err("Expected number".to_string())
                }
            },
            TokenType::Boolean(_) => {
                if let TokenType::Boolean(b) = self.peek().token_type {
                    self.advance();
                    Ok(Node::Literal(Value::Boolean(b)))
                } else {
                    Err("Expected boolean".to_string())
                }
            },
            TokenType::Null => {
                self.advance();
                Ok(Node::Literal(Value::Null))
            },
            TokenType::New => {
                self.new_expression()
            },
            TokenType::Await => {
                Ok(Node::AwaitExpr {
                    value: Box::new(self.expression()?),
                })
            },
            TokenType::Quote => {
                let mut parts = Vec::new();
                while !self.check(&TokenType::Quote) && !self.is_at_end() {
                    if self.match_token(&[TokenType::LeftBrace]) {
                        let expr = self.expression()?;
                        self.consume(&TokenType::RightBrace, "Expected '}' after expression")?;
                        parts.push(expr);
                    } else {
                        let text = self.consume_string_part()?;
                        parts.push(Node::Literal(Value::String(text)));
                    }
                }
                self.consume(&TokenType::Quote, "Expected '\"' after string")?;
                Ok(Node::StringInterpolation { parts })
            },
            TokenType::TypeMapping => {
                let mut entries = Vec::new();
                loop {
                    let param_name = self.consume_identifier("Expected parameter name")?;
                    let (param_type, value) = if self.match_token(&[TokenType::As]) {
                        let param_type = self.type_annotation()?;
                        self.consume(&TokenType::Is, "Expected 'is' after type")?;
                        let value = self.expression()?;
                        (Some(param_type), value)
                    } else if self.match_token(&[TokenType::Is]) {
                        let value = self.expression()?;
                        (None, value)
                    } else {
                        return Err("Expected 'as' or 'is' after parameter name".to_string());
                    };
                    entries.push((param_name, param_type, value));
                    if !self.match_token(&[TokenType::Comma]) {
                        break;
                    }
                    while self.peek().token_type == TokenType::NewLine {
                        self.advance();
                    }
                }
                Ok(Node::MappingLiteral { entries })
            },
            TokenType::TypeList => {
                self.advance();
                let element_type = Box::new(self.type_annotation()?);
                self.consume(&TokenType::CloseBracket, "Expected ']' after type parameter")?;
                Ok(Node::ListType { element_type })
            },
            TokenType::TypePromise => {
                self.advance();
                let value_type = Box::new(self.type_annotation()?);
                self.consume(&TokenType::CloseBracket, "Expected ']' after type parameter")?;
                Ok(Node::PromiseType { value_type })
            },
            // TokenType::TypeAnnotation => {
            //     let type_name = self.consume_identifier("Expected type name")?;
            //     match type_name.as_str() {
            //         "Mapping" => Ok(Node::MappingType {
            //             key_type: Box::new(Node::TypeAnnotation("Text".to_string())),
            //             value_type: Box::new(Node::TypeAnnotation("Any".to_string())),
            //         }),
            //         "Whole" => Ok(Node::TypeAnnotation("Whole".to_string())),
            //         "Decimal" => Ok(Node::TypeAnnotation("Decimal".to_string())),
            //         "Text" => Ok(Node::TypeAnnotation("Text".to_string())),
            //         "Truth" => Ok(Node::TypeAnnotation("Logic".to_string())),
            //         "Nothing" => Ok(Node::TypeAnnotation("Nothing".to_string())),
            //         "Any" => Ok(Node::TypeAnnotation("Any".to_string())),
            //         "Number" => Ok(Node::TypeAnnotation("Number".to_string())),
            //         "Error" => Ok(Node::TypeAnnotation("Error".to_string())),
            //         _ => Err(format!("Unknown type: {}", type_name)),
            //     }
            // },
            _ => Err("Expected expression".to_string()),
        }
    }

    fn new_expression(&mut self) -> Result<Node, String> {
        let class_name = self.consume_identifier("Expected class name after 'new'")?;
        let mut args = Vec::new();

        if self.match_token(&[TokenType::With]) {
            args = self.argument_list()?;
        }

        Ok(Node::New {
            class_name,
            args,
        })
    }

    fn assignment(&mut self) -> Result<Node, String> {
        let name = match &self.tokens[self.current - 1] {
            Token { token_type: TokenType::Identifier(id), .. } => id.clone(),
            _ => return Err("Expected identifier".to_string()),
        };
        
        // Check if this is a new variable declaration with 'as' keyword
        if self.match_token(&[TokenType::As]) {
            let type_annotation = self.type_annotation()?;
            let initializer = if self.match_token(&[TokenType::Is]) {
                Some(Box::new(self.expression()?))
            } else {
                None
            };
            Ok(Node::VariableDecl {
                name,
                type_annotation: Some(Box::new(type_annotation)),
                initializer,
            })
        } else if self.match_token(&[TokenType::Is]) {
            // This is an assignment to an existing variable
            let value = Box::new(self.expression()?);
            Ok(Node::Assignment { name, value })
        } else {
            Err("Expected 'as' or 'is' after identifier".to_string())
        }
    }

    fn or(&mut self) -> Result<Node, String> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let operator = self.previous().token_type.clone();
            let right = Box::new(self.and()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator: operator.clone(),
                right,
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Node, String> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let operator = self.previous().token_type.clone();
            let right = Box::new(self.equality()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator: operator,
                right,
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Node, String> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::Is]) {
            let operator = self.previous().token_type.clone();
            let right = Box::new(self.comparison()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator: operator,
                right,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Node, String> {
        let mut expr = self.term()?;

        while self.match_token(&[TokenType::GreaterThan]) {
            let operator = self.previous().token_type.clone();
            let right = Box::new(self.term()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator: operator,
                right,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Node, String> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().token_type.clone();
            let right = Box::new(self.factor()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator: operator,
                right,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Node, String> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Multiply, TokenType::Divide]) {
            let operator = self.previous().token_type.clone();
            let right = Box::new(self.unary()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator: operator,
                right,
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Node, String> {
        if self.match_token(&[TokenType::Minus]) {
            let operator = self.previous_token_type();
            let right = Box::new(self.unary()?);
            Ok(Node::Binary {
                left: Box::new(Node::Literal(Value::Number(0.0))),
                operator,
                right,
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Node, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::OpenParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::Dot]) {
                let name = self.consume_identifier("Expected property name after '.'")?;
                expr = Node::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Node) -> Result<Node, String> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::CloseParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(&TokenType::CloseParen, "Expected ')' after arguments")?;

        Ok(Node::Call {
            callee: Box::new(callee),
            args: arguments,
        })
    }

    fn primary(&mut self) -> Result<Node, String> {
        let token = self.peek().clone();
        match token.token_type {
            TokenType::Identifier(name) => {
                self.advance();
                Ok(Node::Variable(name))
            },
            TokenType::String(value) => {
                self.advance();
                Ok(Node::Literal(Value::String(value)))
            },
            TokenType::LeftBrace => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&TokenType::RightBrace, "Expected '}' after expression")?;
                Ok(expr)
            },
            TokenType::Quote => {
                self.advance();
                let mut parts = Vec::new();
                
                while !self.check(&TokenType::Quote) && !self.is_at_end() {
                    if self.match_token(&[TokenType::LeftBrace]) {
                        let expr = self.expression()?;
                        self.consume(&TokenType::RightBrace, "Expected '}' after expression")?;
                        parts.push(expr);
                    } else {
                        let text = self.consume_string_part()?;
                        parts.push(Node::Literal(Value::String(text)));
                    }
                }
                
                self.consume(&TokenType::Quote, "Expected '\"' after string")?;
                Ok(Node::StringInterpolation { parts })
            },
            TokenType::Number(value) => {
                self.advance();
                Ok(Node::Literal(Value::Number(value)))
            },
            TokenType::Boolean(value) => {
                self.advance();
                Ok(Node::Literal(Value::Boolean(value)))
            },
            TokenType::Null => {
                self.advance();
                Ok(Node::Literal(Value::Null))
            },
            TokenType::TypeMapping => {
                self.advance();
                Ok(Node::MappingLiteral { entries: Vec::new() })
            },
            _ => Err("Expected expression".to_string()),
        }
    }

    fn consume_string_part(&mut self) -> Result<String, String> {
        if let TokenType::StringPart(text) = &self.peek().token_type {
            let text = text.clone();
            self.advance();
            Ok(text)
        } else {
            Err("Expected string part".to_string())
        }
    }

    fn when_statement(&mut self) -> Result<Node, String> {
        let condition = Box::new(self.expression()?);
        self.consume(&TokenType::Colon, "Expected ':' after when condition")?;
        let then_branch = Box::new(self.block()?);
        
        let else_branch = if self.match_token(&[TokenType::Or]) {
            self.consume(&TokenType::Colon, "Expected ':' after 'or'")?;
            Some(Box::new(self.block()?))
        } else {
            None
        };

        Ok(Node::WhenStmt {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn loop_statement(&mut self) -> Result<Node, String> {
        self.consume(&TokenType::While, "Expected 'while' after 'loop'")?;
        let condition = Box::new(self.expression()?);
        self.consume(&TokenType::Colon, "Expected ':' after loop condition")?;
        let body = Box::new(self.block()?);

        Ok(Node::LoopStmt { condition, body })
    }

    fn show_statement(&mut self) -> Result<Node, String> {
        self.advance(); // Consume 'show'
        let expr = self.expression()?;
        Ok(Node::ShowStmt(Box::new(expr)))
    }

    fn raise_statement(&mut self) -> Result<Node, String> {
        let message = Box::new(self.expression()?);
        self.consume(&TokenType::As, "Expected 'as' after raise message")?;
        let error_type = Box::new(self.type_annotation()?);
        
        Ok(Node::RaiseStmt {
            message,
            error_type,
        })
    }

    fn return_statement(&mut self) -> Result<Node, String> {
        let value = Box::new(self.expression()?);
        Ok(Node::ReturnStmt(value))
    }

    fn expression_statement(&mut self) -> Result<Node, String> {
        let expr = self.expression()?;
        Ok(Node::ExpressionStmt(Box::new(expr)))
    }

    fn string_literal(&mut self) -> Result<Node, String> {
        // Clone the string before advancing
        let string_content = if let TokenType::String(s) = &self.peek().token_type {
            s.clone()
        } else {
            return Err("Expected string literal".to_string());
        };
        
        // Now advance the parser
        self.advance();
        
        // Process the string content
        if string_content.contains('{') && string_content.contains('}') {
            let mut parts = Vec::new();
            let mut current_text = String::new();
            let mut chars = string_content.chars().peekable();
            
            while let Some(c) = chars.next() {
                if c == '{' {
                    // Add accumulated text if any
                    if !current_text.is_empty() {
                        parts.push(Node::Literal(Value::String(current_text.clone())));
                        current_text.clear();
                    }
                    
                    // Collect variable name
                    let mut var_name = String::new();
                    while let Some(&next_char) = chars.peek() {
                        if next_char == '}' {
                            chars.next(); // consume the '}'
                            break;
                        }
                        var_name.push(chars.next().unwrap());
                    }
                    
                    // Add variable reference
                    parts.push(Node::Variable(var_name));
                } else {
                    current_text.push(c);
                }
            }
            
            // Add any remaining text
            if !current_text.is_empty() {
                parts.push(Node::Literal(Value::String(current_text)));
            }
            
            Ok(Node::StringInterpolation { parts })
        } else {
            Ok(Node::Literal(Value::String(string_content)))
        }
    }

    fn argument_list(&mut self) -> Result<Vec<Node>, String> {
        let mut args = Vec::new();

        if !self.check(&TokenType::CloseParen) && !self.is_at_end() {
            loop {
                args.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        Ok(args)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::EOF)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().token_type == token_type
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(message.to_string())
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(message.to_string())
        }
    }

    fn statement(&mut self) -> Result<Node, String> {
        match self.peek().token_type {
            TokenType::Show => {
                self.advance(); // Consume 'show'
                match &self.peek().token_type {
                    TokenType::Identifier(_) => {
                        let name = self.consume_identifier("Expected variable name after 'show'")?;
                        Ok(Node::ShowStmt(Box::new(Node::Variable(name))))
                    },
                    TokenType::String(_) => {
                        let expr = self.string_literal()?;
                        Ok(Node::ShowStmt(Box::new(expr)))
                    },
                    TokenType::Number(_) => {
                        if let TokenType::Number(n) = self.advance().token_type {
                            Ok(Node::ShowStmt(Box::new(Node::Literal(Value::Number(n)))))
                        } else {
                            Err("Expected number".to_string())
                        }
                    },
                    TokenType::Boolean(_) => {
                        let expr = self.boolean_literal()?;
                        Ok(Node::ShowStmt(Box::new(expr)))
                    },
                    TokenType::Null | TokenType::TypeMapping => {
                        Ok(Node::ShowStmt(Box::new(Node::Literal(Value::Null))))
                    },
                    TokenType::TypePromise => {
                        let expr = self.promise_literal()?;
                        Ok(Node::ShowStmt(Box::new(expr)))
                    },
                    TokenType::TypeList => {
                        let expr = self.list_literal()?;
                        Ok(Node::ShowStmt(Box::new(expr)))
                    },
                    _ => Err("Expected variable name, string, or number after 'show'".to_string()),
                }
            },
            TokenType::Raise => {
                self.advance();
                self.raise_statement()
            },
            TokenType::Returns => {
                self.advance();
                self.return_statement()
            },
            TokenType::Requires => {
                self.advance(); // Consume 'requires'
                self.declaration()
            },
            TokenType::Returning => {
                self.advance(); // Consume 'returning'
                self.declaration()
            },
            TokenType::Emit => {
                self.advance(); // Consume 'emit'
                self.declaration()
            },
            TokenType::Using => {
                self.advance(); // Consume 'using'
                self.declaration()
            },
            TokenType::With => {
                self.advance(); // Consume 'with'
                self.declaration()
            },
            TokenType::As => {
                self.advance(); // Consume 'as'
                self.declaration()
            },
            TokenType::Is => {
                self.advance(); // Consume 'is'
                self.declaration()
            },
            TokenType::To => {
                self.advance(); // Consume 'to'
                self.declaration()
            },
            TokenType::Of => {
                self.advance(); // Consume 'of'
                self.declaration()
            },
            TokenType::At => {
                self.advance(); // Consume 'at'
                self.declaration()
            },
            TokenType::And => {
                self.advance(); // Consume 'and'
                self.declaration()
            },
            TokenType::Each => {
                self.advance(); // Consume 'each'
                self.declaration()
            },
            TokenType::Becomes => {
                self.advance(); // Consume 'becomes'
                self.declaration()
            },
            TokenType::My => {
                self.advance(); // Consume 'my'
                self.declaration()
            },
            TokenType::About => {
                self.advance(); // Consume 'about'
                self.declaration()
            },
            TokenType::Me => {
                self.advance(); // Consume 'me'
                self.declaration()
            },
            TokenType::Loop => {
                self.advance(); // Consume 'loop'
                self.loop_statement()
            },
            TokenType::While => {
                self.advance(); // Consume 'while'
                self.loop_statement()
            },
            TokenType::Emit => {
                self.advance(); // Consume 'Emit'
                self.declaration()
            },
            TokenType::Match => {
                self.advance(); // Consume 'match'
                self.declaration()
            },
            TokenType::Output => {
                self.advance(); // Consume 'output'
                self.declaration()
            },
            _ => self.expression_statement(),
        }
    }

    fn previous_token_type(&mut self) -> TokenType {
        self.previous().token_type.clone()
    }

    fn mapping_initializer(&mut self) -> Result<Node, String> {
        let mut entries = Vec::new();
        
        loop {
            // Parse parameter name
            let param_name = self.consume_identifier("Expected parameter name")?;
            
            // Handle both explicit and implicit type declarations
            let (param_type, value) = if self.match_token(&[TokenType::As]) {
                // Explicit type: param as Type is value
                let param_type = self.type_annotation()?;
                self.consume(&TokenType::Is, "Expected 'is' after type")?;
                let value = self.expression()?;
                (Some(param_type), value)
            } else if self.match_token(&[TokenType::Is]) {
                // Implicit type: param is value
                let value = self.expression()?;
                (None, value)
            } else {
                return Err("Expected 'as' or 'is' after parameter name".to_string());
            };
            
            entries.push((param_name, param_type, value));
            
            if !self.match_token(&[TokenType::Comma]) {
                break;
            }
            
            // Skip any newlines after comma
            while self.peek().token_type == TokenType::NewLine {
                self.advance();
            }
        }
        
        Ok(Node::MappingLiteral { entries })
    }

    fn type_from_annotation(&mut self, type_node: &Node) -> Result<Type, String> {
        match type_node {
            Node::MappingType { key_type, value_type } => {
                let key = self.type_from_annotation(key_type)?;
                let value = self.type_from_annotation(value_type)?;
                Ok(Type::Map {
                    key: Box::new(key),
                    value: Box::new(value),
                })
            },
            Node::TypeAnnotation(type_name) => {
                match type_name.as_str() {
                    "Whole" => Ok(Type::Whole),
                    "Decimal" => Ok(Type::Decimal),
                    "Text" => Ok(Type::Text),
                    "Truth" => Ok(Type::Truth),
                    "Nothing" => Ok(Type::Nothing),
                    "Any" => Ok(Type::Any),
                    "Promise" => Ok(Type::Promise(Box::new(Type::Any))),
                    "List" => Ok(Type::List(Box::new(Type::Any))),
                    "Mapping" => Ok(Type::Map { key: Box::new(Type::Text), value: Box::new(Type::Any) }),
                    _ => Err(format!("Unknown type: {}", type_name)),
                }
            },
            _ => Err("Invalid type annotation".to_string()),
        }
    }
}

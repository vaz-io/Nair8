use crate::tokenizer::{Token, TokenType};

#[derive(Debug)]
pub enum Node {
    // Declarations
    VariableDecl {
        name: String,
        type_annotation: Option<Box<Node>>,
        initializer: Option<Box<Node>>,
    },
    JobDecl {
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
    Literal(TokenType),
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
    YieldStmt(Box<Node>),
    AwaitExpr {
        value: Box<Node>,
    },
    PropertyAccess {
        object: Box<Node>,
        property: String,
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
        if self.match_token(&[TokenType::Job]) {
            self.job_declaration()
        } else if self.match_token(&[TokenType::Object]) {
            self.object_declaration()
        } else if let TokenType::Identifier(_) = self.peek().token_type {
            self.variable_declaration()
        } else {
            self.statement()
        }
    }

    fn job_declaration(&mut self) -> Result<Node, String> {
        let name = self.consume_identifier("Expected job name")?;
        
        let mut params = Vec::new();
        if self.match_token(&[TokenType::Requires]) {
            params = self.parameter_list()?;
        }

        let return_type = if self.match_token(&[TokenType::Returns, TokenType::Returning]) {
            Some(Box::new(self.type_annotation()?))
        } else {
            None
        };

        self.consume(&TokenType::Colon, "Expected ':' after job declaration")?;
        let body = Box::new(self.block()?);

        Ok(Node::JobDecl {
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
            } else if self.match_token(&[TokenType::Job]) {
                methods.push(self.job_declaration()?);
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

    fn variable_declaration(&mut self) -> Result<Node, String> {
        let name = match self.peek().token_type.clone() {
            TokenType::Identifier(name) => {
                self.advance();
                name
            },
            _ => return Err("Expected variable name".to_string()),
        };

        if self.match_token(&[TokenType::As]) {
            let type_annotation = Some(Box::new(self.type_annotation()?));
            let initializer = if self.match_token(&[TokenType::Is]) {
                Some(Box::new(self.expression()?))
            } else {
                None
            };
            
            Ok(Node::VariableDecl {
                name,
                type_annotation,
                initializer,
            })
        } else if self.match_token(&[TokenType::Is]) {
            let initializer = Some(Box::new(self.expression()?));
            Ok(Node::VariableDecl {
                name,
                type_annotation: None,
                initializer,
            })
        } else {
            Err("Expected 'as' or 'is' after variable name".to_string())
        }
    }

    // Helper methods
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.tokens[self.current].token_type == token_type
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(message.to_string())
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        match &self.peek().token_type {
            TokenType::Identifier(name) => {
                self.advance();
                Ok(name.clone())
            }
            _ => Err(message.to_string()),
        }
    }

    fn statement(&mut self) -> Result<Node, String> {
        if self.match_token(&[TokenType::When]) {
            self.when_statement()
        } else if self.match_token(&[TokenType::Loop]) {
            self.loop_statement()
        } else if self.match_token(&[TokenType::Show]) {
            self.show_statement()
        } else if self.match_token(&[TokenType::Raise]) {
            self.raise_statement()
        } else if self.match_token(&[TokenType::Output]) {
            self.return_statement()
        } else {
            self.expression_statement()
        }
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
            TokenType::TypeWhole => {
                self.advance();
                Ok(Node::TypeAnnotation("Whole".to_string()))
            },
            TokenType::TypeDecimal => {
                self.advance();
                Ok(Node::TypeAnnotation("Decimal".to_string()))
            },
            TokenType::TypeText => {
                self.advance();
                Ok(Node::TypeAnnotation("Text".to_string()))
            },
            TokenType::TypeLogic => {
                self.advance();
                Ok(Node::TypeAnnotation("Logic".to_string()))
            },
            TokenType::TypeVoid => {
                self.advance();
                Ok(Node::TypeAnnotation("Void".to_string()))
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
            TokenType::TypeMapping => {
                self.advance();
                if self.match_token(&[TokenType::Of]) {
                    let key_type = Box::new(self.type_annotation()?);
                    self.consume(&TokenType::To, "Expected 'to' after key type")?;
                    let value_type = Box::new(self.type_annotation()?);
                    Ok(Node::MappingType { key_type, value_type })
                } else {
                    Ok(Node::TypeAnnotation("Mapping".to_string()))
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
            TokenType::TypePerson => {
                self.advance();
                Ok(Node::TypeAnnotation("Person".to_string()))
            },
            TokenType::TypeBaseEntity => {
                self.advance();
                Ok(Node::TypeAnnotation("BaseEntity".to_string()))
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
        
        Ok(Node::JobDecl {
            name: "build".to_string(),
            params,
            return_type: None,
            body,
        })
    }

    fn expression(&mut self) -> Result<Node, String> {
        if self.match_token(&[TokenType::New]) {
            self.new_expression()
        } else if self.match_token(&[TokenType::Await]) {
            Ok(Node::AwaitExpr {
                value: Box::new(self.expression()?),
            })
        } else {
            self.assignment()
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
        let expr = self.or()?;

        if self.match_token(&[TokenType::Is]) {
            let value = Box::new(self.expression()?);
            
            if let Node::Variable(name) = expr {
                return Ok(Node::Assignment { name, value });
            }
            
            return Err("Invalid assignment target".to_string());
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Node, String> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let operator = self.previous().token_type;
            let right = Box::new(self.and()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Node, String> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let operator = self.previous().token_type;
            let right = Box::new(self.equality()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Node, String> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::Is]) {
            let operator = self.previous().token_type;
            let right = Box::new(self.comparison()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Node, String> {
        let mut expr = self.term()?;

        while self.match_token(&[TokenType::GreaterThan]) {
            let operator = self.previous().token_type;
            let right = Box::new(self.term()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Node, String> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().token_type;
            let right = Box::new(self.factor()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Node, String> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Multiply, TokenType::Divide]) {
            let operator = self.previous().token_type;
            let right = Box::new(self.unary()?);
            expr = Node::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Node, String> {
        if self.match_token(&[TokenType::Minus]) {
            let operator = self.previous().token_type;
            let right = Box::new(self.unary()?);
            Ok(Node::Binary {
                left: Box::new(Node::Literal(TokenType::Number(0.0))),
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
        if self.match_token(&[TokenType::New]) {
            let class_name = self.consume_identifier("Expected class name after 'new'")?;
            let mut args = Vec::new();
            
            if self.match_token(&[TokenType::With]) {
                args = self.argument_list()?;
            }
            
            Ok(Node::New { class_name, args })
        } else if let TokenType::String(s) = self.peek().token_type.clone() {
            self.advance();
            // Check for string interpolation
            if s.contains('{') && s.contains('}') {
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut chars = s.chars().peekable();
                
                while let Some(c) = chars.next() {
                    if c == '{' {
                        if !current.is_empty() {
                            parts.push(Node::Literal(TokenType::String(current.clone())));
                            current.clear();
                        }
                        
                        let mut expr = String::new();
                        while let Some(&next) = chars.peek() {
                            if next == '}' {
                                chars.next();
                                break;
                            }
                            expr.push(chars.next().unwrap());
                        }
                        
                        // Create a Variable node for the interpolated expression
                        parts.push(Node::Variable(expr.trim().to_string()));
                    } else {
                        current.push(c);
                    }
                }
                
                if !current.is_empty() {
                    parts.push(Node::Literal(TokenType::String(current)));
                }
                
                Ok(Node::StringInterpolation { parts })
            } else {
                Ok(Node::Literal(TokenType::String(s)))
            }
        } else if let TokenType::Number(_) | TokenType::Boolean(_) | TokenType::Null = self.peek().token_type {
            Ok(Node::Literal(self.advance().token_type))
        } else if let TokenType::Identifier(name) = self.peek().token_type.clone() {
            self.advance();
            Ok(Node::Variable(name))
        } else {
            Err("Expected expression".to_string())
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
        let value = Box::new(self.expression()?);
        Ok(Node::ShowStmt(value))
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
        if let TokenType::String(s) = self.advance().token_type {
            // Check for string interpolation
            if s.contains('{') && s.contains('}') {
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut chars = s.chars().peekable();
                
                while let Some(c) = chars.next() {
                    if c == '{' {
                        if !current.is_empty() {
                            parts.push(Node::Literal(TokenType::String(current.clone())));
                            current.clear();
                        }
                        
                        let mut expr = String::new();
                        while let Some(&next) = chars.peek() {
                            if next == '}' {
                                chars.next();
                                break;
                            }
                            expr.push(chars.next().unwrap());
                        }
                        
                        // Create a Variable node for the interpolated expression
                        parts.push(Node::Variable(expr.trim().to_string()));
                    } else {
                        current.push(c);
                    }
                }
                
                if !current.is_empty() {
                    parts.push(Node::Literal(TokenType::String(current)));
                }
                
                Ok(Node::StringInterpolation { parts })
            } else {
                Ok(Node::Literal(TokenType::String(s)))
            }
        } else {
            Err("Expected string literal".to_string())
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
}

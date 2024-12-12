#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    As,
    Is,
    Of,
    To,
    When,
    Or,
    Do,
    Fail,
    Always,
    Extends,
    Returns,
    Requires,
    Returning,
    New,
    With,
    Using,
    Loop,
    While,
    Yield,
    Match,
    Output,
    Raise,
    Show,
    Await,
    At,
    And,
    Each,
    Becomes,
    My,
    About,
    Me,
    
    // Declaration keywords
    Job,
    Object,
    Build,
    Defaults,

    // Types
    TypeWhole,
    TypeDecimal,
    TypeText,
    TypeLogic,
    TypeVoid,
    TypeList,
    TypeMapping,
    TypePromise,
    TypeAny,
    TypeNumber,
    TypeError,
    TypePerson,
    TypeBaseEntity,

    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    Null,

    // Symbols
    Colon,
    Comma,
    Dot,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Plus,
    Minus,
    Multiply,
    Divide,
    GreaterThan,
    BackSlash,      // For line continuation

    // Identifiers
    Identifier(String),

    // Comments
    Comment(String),

    EOF,
}

pub struct Tokenizer {
    source: Vec<char>,
    current: usize,
    start: usize,
    line: usize,
    column: usize,
}

impl Tokenizer {
    pub fn new(source: &str) -> Self {
        Tokenizer {
            source: source.chars().collect(),
            current: 0,
            start: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.start = self.current;
            if let Ok(token) = self.scan_token() {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            token_type: TokenType::EOF,
            literal: String::new(),
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    // Helper methods
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let current_char = self.source[self.current];
        self.current += 1;
        self.column += 1;
        current_char
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source[self.current]
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source[self.current + 1]
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn create_token(&self, token_type: TokenType) -> Token {
        let literal: String = self.source[self.start..self.current]
            .iter()
            .collect();
        
        Token {
            token_type,
            literal,
            line: self.line,
            column: self.column,
        }
    }

    fn number_token(&mut self) -> Result<Token, String> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // Look for a decimal part
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            // Consume the "."
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let number = self.source[self.start..self.current]
            .iter()
            .collect::<String>();
        let number = number.parse::<f64>().unwrap();
        Ok(Token {
            token_type: TokenType::Number(number),
            literal: number.to_string(),
            line: self.line,
            column: self.column,
        })
    }

    fn scan_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();
        self.start = self.current;

        if self.is_at_end() {
            return Ok(self.create_token(TokenType::EOF));
        }

        let c = self.advance();

        match c {
            // Handle comments
            '#' => {
                while !self.is_at_end() && self.peek() != '\n' {
                    self.advance();
                }
                let mut comment = self.source[self.start..self.current]
                    .iter()
                    .collect::<String>();
                comment.remove(0);
                comment.remove(comment.len() - 1);
                // println!("Comment: {}", comment);
                // println!("Line: {}", self.line);
                // println!("Column: {}", self.column);    
                // println!("Start: {}", self.start);
                // println!("Current: {}", self.current);
                // println!("Source: {}", self.source);

                Ok(Token {
                    token_type: TokenType::Comment(comment.clone()),
                    literal: comment.clone(),
                    line: self.line,
                    column: self.column,
                })
            },

            // String interpolation
            '"' => {
                while !self.is_at_end() && self.peek() != '"' {
                    if self.peek() == '{' {
                        // Handle string interpolation
                        self.advance(); // consume {
                        while !self.is_at_end() && self.peek() != '}' {
                            self.advance();
                        }
                        if !self.is_at_end() {
                            self.advance(); // consume }
                        }
                    } else {
                        self.advance();
                    }
                }
                
                if self.is_at_end() {
                    Err("Unterminated string".to_string())
                } else {
                    self.advance(); // closing quote
                    let string_content = self.source[self.start + 1..self.current - 1]
                        .iter()
                        .collect::<String>();
                    Ok(Token {
                        token_type: TokenType::String(string_content.clone()),
                        literal: string_content.clone(),
                        line: self.line,
                        column: self.column,
                    })
                }
            },

            // Line continuation
            '\\' => Ok(self.create_token(TokenType::BackSlash)),

            // Other symbols
            ':' => Ok(self.create_token(TokenType::Colon)),
            ',' => Ok(self.create_token(TokenType::Comma)),
            '.' => Ok(self.create_token(TokenType::Dot)),
            '[' => Ok(self.create_token(TokenType::OpenBracket)),
            ']' => Ok(self.create_token(TokenType::CloseBracket)),
            '(' => Ok(self.create_token(TokenType::OpenParen)),
            ')' => Ok(self.create_token(TokenType::CloseParen)),
            '+' => Ok(self.create_token(TokenType::Plus)),
            '-' => Ok(self.create_token(TokenType::Minus)),
            '*' => Ok(self.create_token(TokenType::Multiply)),
            '/' => Ok(self.create_token(TokenType::Divide)),
            '>' => Ok(self.create_token(TokenType::GreaterThan)),

            // Numbers and identifiers
            c if c.is_ascii_digit() => self.number_token(),
            c if c.is_alphabetic() || c == '_' => self.identifier_token(),
            
            _ => Err(format!("Unexpected character: {}", c)),
        }
    }

    fn identifier_token(&mut self) -> Result<Token, String> {
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = match text.as_str() {
            // Keywords
            "as" => TokenType::As,
            "is" => TokenType::Is,
            "of" => TokenType::Of,
            "to" => TokenType::To,
            "when" => TokenType::When,
            "or" => TokenType::Or,
            "do" => TokenType::Do,
            "fail" => TokenType::Fail,
            "always" => TokenType::Always,
            "inherits" => TokenType::Extends,
            "returns" => TokenType::Returns,
            "requires" => TokenType::Requires,
            "returning" => TokenType::Returning,
            "new" => TokenType::New,
            "with" => TokenType::With,
            "using" => TokenType::Using,
            "loop" => TokenType::Loop,
            "while" => TokenType::While,
            "yield" => TokenType::Yield,
            "match" => TokenType::Match,
            "output" => TokenType::Output,
            "raise" => TokenType::Raise,
            "show" => TokenType::Show,
            "await" => TokenType::Await,
            "at" => TokenType::At,
            "and" => TokenType::And,
            "each" => TokenType::Each,
            "becomes" => TokenType::Becomes,
            "my" => TokenType::My,
            "about" => TokenType::About,
            "me" => TokenType::Me,

            // Declaration keywords
            "Job" => TokenType::Job,
            "Object" => TokenType::Object,
            "build" => TokenType::Build,
            "defaults" => TokenType::Defaults,

            // Types
            "Whole" => TokenType::TypeWhole,
            "Decimal" => TokenType::TypeDecimal,
            "Text" => TokenType::TypeText,
            "Logic" => TokenType::TypeLogic,
            "Void" => TokenType::TypeVoid,
            "List" => TokenType::TypeList,
            "Mapping" => TokenType::TypeMapping,
            "Promise" => TokenType::TypePromise,
            "Any" => TokenType::TypeAny,
            "Number" => TokenType::TypeNumber,
            "Error" => TokenType::TypeError,
            "Person" => TokenType::TypePerson,
            "BaseEntity" => TokenType::TypeBaseEntity,

            // Boolean literals
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            "null" => TokenType::Null,

            // Default to identifier
            _ => TokenType::Identifier(text.clone()),
        };

        Ok(Token {
            token_type,
            literal: text,
            line: self.line,
            column: self.column,
        })
    }
}

// Add Display implementation for Token if not already present
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} '{}' (line: {}, col: {})", 
            self.token_type,
            self.literal,
            self.line,
            self.column
        )
    }
}
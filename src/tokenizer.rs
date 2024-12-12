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
    Emit,
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
    TypeWhole,  // Whole number
    TypeDecimal, // Decimal number
    TypeText, // Text
    TypeLogic, // Boolean 
    TypeVoid, // Null
    TypeList, // List
    TypeMapping, // Mapping
    TypePromise, // Future
    TypeAny, // Any
    TypeNumber, // Number
    TypeError, // Error

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
    NewLine,

    Includes,  // Add this new token
    LeftBrace,
    RightBrace,
    Quote,
    StringPart(String),
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
                _ => break,
            }
        }
    }

    fn create_token(&mut self, token_type: TokenType) -> Token {
        Token {
            token_type,
            literal: self.source[self.start..self.current].iter().collect::<String>(),
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
            '"' => self.string(),
            '{' => Ok(self.create_token(TokenType::LeftBrace)),
            '}' => Ok(self.create_token(TokenType::RightBrace)),
            // ... other cases ...
            _ => {
                // Read identifier first, then process it
                let ident = self.read_identifier();
                Ok(self.create_identifier_token(ident))
            },
        }
    }

    fn string(&mut self) -> Result<Token, String> {
        let mut string = String::new();
        
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '{' {
                if !string.is_empty() {
                    return Ok(Token {
                        token_type: TokenType::StringPart(string.clone()),
                        literal: string,
                        line: self.line,
                        column: self.column,
                    });
                }
                return Ok(self.create_token(TokenType::LeftBrace));
            }
            string.push(self.advance());
        }

        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }

        // Consume the closing quote
        self.advance();
        
        Ok(Token {
            token_type: TokenType::String(string.clone()),
            literal: string,
            line: self.line,
            column: self.column,
        })
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
            "Emit" => TokenType::Emit,
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

    fn identifier_type(&self, text: String) -> Result<Token, String> {
        println!("Processing identifier: {}", text);
        let token_type = match text.as_str() {
            "Mapping" => {
                println!("Found Mapping keyword");
                TokenType::TypeMapping
            },
            "Text" => {
                println!("Found Text keyword");
                TokenType::TypeText
            },
            "of" => TokenType::Of,
            "to" => TokenType::To,
            "includes" => TokenType::Includes,
            _ => {
                println!("Unknown identifier: {}", text);
                TokenType::Identifier(text.clone())
            },
        };

        Ok(Token {
            token_type,
            literal: text,
            line: self.line,
            column: self.column,
        })
    }

    fn read_identifier(&mut self) -> String {
        let start = self.start;
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }
        self.source[start..self.current].iter().collect()
    }

    fn create_identifier_token(&self, text: String) -> Token {
        let token_type = match text.as_str() {
            "is" => TokenType::Is,
            "as" => TokenType::As,
            "Mapping" => TokenType::TypeMapping,
            "Text" => TokenType::TypeText,
            "includes" => TokenType::Includes,
            "Object" => TokenType::Object,
            "Job" => TokenType::Job,
            "build" => TokenType::Build,
            "defaults" => TokenType::Defaults,
            "of" => TokenType::Of,
            "to" => TokenType::To,
            // "includes" => TokenType::Includes,
            "show" => TokenType::Show,
            "raise" => TokenType::Raise,
            "await" => TokenType::Await,
            "at" => TokenType::At,
            "and" => TokenType::And,
            "each" => TokenType::Each,
            "becomes" => TokenType::Becomes,
            "my" => TokenType::My,
            "about" => TokenType::About,
            "me" => TokenType::Me,
            "loop" => TokenType::Loop,
            "while" => TokenType::While,
            "Emit" => TokenType::Emit,
            "match" => TokenType::Match,
            "output" => TokenType::Output,
            "returns" => TokenType::Returns,
            "requires" => TokenType::Requires,
            "returning" => TokenType::Returning,
            "new" => TokenType::New,
            "with" => TokenType::With,
            "using" => TokenType::Using,
            _ => TokenType::Identifier(text.clone()),
        };

        Token {
            token_type,
            literal: text,
            line: self.line,
            column: self.column,
        }
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
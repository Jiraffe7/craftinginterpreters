use std::{mem, str::Chars};

pub enum LoxError {
    ParseError { line: usize, message: String },
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum TokenType {
    // Single-character tokens
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    // Literals
    IDENTIFIER,
    STRING,
    NUMBER(f64),

    // Keywords
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

impl TokenType {
    fn to_keyword(s: &str) -> Option<TokenType> {
        use TokenType::*;
        let t = match s {
            "and" => AND,
            "class" => CLASS,
            "else" => ELSE,
            "false" => FALSE,
            "fun" => FUN,
            "for" => FOR,
            "if" => IF,
            "nil" => NIL,
            "or" => OR,
            "print" => PRINT,
            "return" => RETURN,
            "super" => SUPER,
            "this" => THIS,
            "true" => TRUE,
            "var" => VAR,
            "while" => WHILE,
            _ => return None,
        };
        Some(t)
    }
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

pub struct Scanner<'a> {
    chars: itertools::PeekNth<Chars<'a>>,
    tokens: Vec<Token>,
    current_string: String,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            chars: itertools::peek_nth(source.chars()),
            tokens: Default::default(),
            current_string: Default::default(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: String::from(""),
            line: self.line,
        });

        Ok(self.tokens)
    }

    fn is_at_end(&mut self) -> bool {
        // self.current >= self.len // original
        self.chars.peek().is_none()
    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        use LoxError::*;
        use TokenType::*;

        let c = self.advance().expect("Reading past end");

        match c {
            '(' => self.add_token(LEFT_PAREN),
            ')' => self.add_token(RIGHT_PAREN),
            '{' => self.add_token(LEFT_BRACE),
            '}' => self.add_token(RIGHT_BRACE),
            ',' => self.add_token(COMMA),
            '.' => self.add_token(DOT),
            '-' => self.add_token(MINUS),
            '+' => self.add_token(PLUS),
            ';' => self.add_token(SEMICOLON),
            '*' => self.add_token(STAR),
            '!' => {
                let token = if self.match_char('=') {
                    BANG_EQUAL
                } else {
                    BANG
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.match_char('=') {
                    EQUAL_EQUAL
                } else {
                    EQUAL
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.match_char('=') {
                    LESS_EQUAL
                } else {
                    LESS
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.match_char('=') {
                    GREATER_EQUAL
                } else {
                    GREATER
                };
                self.add_token(token);
            }
            '/' => {
                if self.match_char('/') {
                    // A comment goes until the end of the line.
                    while self.chars.peek().filter(|c| **c != '\n').is_some() {
                        self.advance();
                    }
                    // ignore parsing comment
                    self.current_string.clear();
                } else {
                    self.add_token(SLASH);
                }
            }
            ' ' | '\r' | '\t' => return Ok(()),
            '\n' => {
                self.line += 1;
                return Ok(());
            }
            '"' => self.string()?,
            '0'..='9' => self.number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.identifier()?,
            _ => {
                return Err(ParseError {
                    line: self.line,
                    message: String::from("Unexpected character."),
                })
            }
        };

        Ok(())
    }

    fn identifier(&mut self) -> Result<(), LoxError> {
        while self
            .chars
            .peek()
            .filter(|c| matches!(**c, 'a'..='z' | 'A'..='Z' | '_'| '0'..='9'))
            .is_some()
        {
            self.advance();
        }
        if let Some(t) = TokenType::to_keyword(&self.current_string) {
            self.add_token(t);
        } else {
            self.add_token(TokenType::IDENTIFIER);
        }
        Ok(())
    }

    fn number(&mut self) -> Result<(), LoxError> {
        while self
            .chars
            .peek()
            .filter(|c| matches!(**c, '0'..='9'))
            .is_some()
        {
            self.advance();
        }

        // Look for a fractional part
        if self.chars.peek().filter(|c| **c == '.').is_some()
            && self
                .chars
                .peek_nth(1)
                .filter(|c| matches!(**c, '0'..='9'))
                .is_some()
        {
            // Consume the "."
            self.advance();

            while self
                .chars
                .peek()
                .filter(|c| matches!(**c, '0'..='9'))
                .is_some()
            {
                self.advance();
            }
        }
        let num: f64 = self.current_string.parse().expect("current_string not f64");
        self.add_token(TokenType::NUMBER(num));
        Ok(())
    }

    fn string(&mut self) -> Result<(), LoxError> {
        while self.chars.peek().filter(|c| **c != '"').is_some() {
            if self.chars.peek().filter(|c| **c == '\n').is_some() {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(LoxError::ParseError {
                line: self.line,
                message: String::from("Unterminated string."),
            });
        }
        // The closing ".
        self.advance();
        // remove the surrounding "
        self.current_string = self.current_string[1..self.current_string.len() - 1].to_string();
        self.add_token(TokenType::STRING);
        Ok(())
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        let c = self.chars.next()?;
        self.current_string.push(c);
        Some(c)
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = mem::take(&mut self.current_string);
        self.tokens.push(Token {
            token_type,
            lexeme: text,
            line: self.line,
        });
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        let c = match self.chars.peek() {
            Some(c) => c,
            None => return false,
        };
        if *c == expected {
            self.current += 1;
            self.advance();
            true
        } else {
            false
        }
    }
}

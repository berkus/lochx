use crate::error;

/// Current scanner state for iterating over the source input.
pub struct Scanner<'a> {
    source: &'a str,
    line: usize,
    start: usize,
    current: usize,
    tokens: Vec<Token<'a>>,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    r#type: TokenType,
    lexeme: &'a str, // @todo: Use Range into a source str (to print error information)
    line: usize,
}

impl<'a> Token<'a> {
    pub fn new(r#type: TokenType, lexeme: &'a str, line: usize) -> Self {
        Self {
            r#type,
            lexeme,
            line,
        }
    }
}

impl std::fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {:?} {}", self.line, self.r#type, self.lexeme)
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    EOF,

    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    KwAnd,
    KwClass,
    KwElse,
    KwFalse,
    KwFun,
    KfFor,
    KwIf,
    KwNil,
    KwOr,
    KwPrint,
    KwReturn,
    KwSuper,
    KwThis,
    KwTrue,
    KwVar,
    KwWhile,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            line: 1,
            current: 0,
            start: 0,
            tokens: vec![],
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token<'a>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.add_token(TokenType::EOF);
        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let r#type = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(r#type);
            }
            '=' => {
                let r#type = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(r#type);
            }
            '<' => {
                let r#type = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(r#type);
            }
            '>' => {
                let r#type = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(r#type);
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {
                // Ignore whitespace.
            }
            '\n' => {
                self.line += 1;
            }
            _ => {
                error(self.line, &format!("Unexpected character `{}`", c));
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current);
        self.current += 1;
        c.expect("Got past end of input")
    }

    /// Return true and advance if the next character is the expected one.
    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current) != Some(expected) {
            return false;
        }
        self.current += 1;
        return true;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source
            .chars()
            .nth(self.current)
            .expect("Got past end of input")
    }

    fn add_token(&mut self, r#type: TokenType) {
        let lexeme = &self.source[self.start..self.current];
        self.tokens.push(Token::new(r#type, lexeme, self.line));
    }
}

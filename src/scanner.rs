use {
    crate::{error::RuntimeError, literal::LiteralValue, runtime},
    maplit::hashmap,
    std::collections::HashMap,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub span: std::ops::Range<usize>,
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}..{}]", self.line, self.span.start, self.span.end)
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub position: SourcePosition,
    literal: Option<LiteralValue>,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme(runtime::source()))
    }
}

// @todo identify by absolute position in source_map
impl std::hash::Hash for Token {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.position.span.start);
        state.write_usize(self.position.span.end);
    }
}

impl Eq for Token {}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
    }
}

impl Token {
    pub fn new(r#type: TokenType, position: SourcePosition, literal: Option<LiteralValue>) -> Self {
        Self {
            r#type,
            position,
            literal,
        }
    }

    pub fn lexeme<'src>(&self, source: &'src str) -> &'src str {
        &source[self.position.span.clone()]
    }

    pub fn literal_num(&self) -> Option<f64> {
        match self.literal {
            Some(LiteralValue::Num(x)) => Some(x),
            _ => None,
        }
    }

    pub fn literal_str(&self) -> Option<String> {
        match self.literal {
            Some(LiteralValue::Str(ref s)) => Some(s.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    KwFor,
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

trait IsIdentifier {
    fn is_identifier(&self) -> bool;
}

impl IsIdentifier for char {
    fn is_identifier(&self) -> bool {
        self.is_alphanumeric() || *self == '_'
    }
}

/// Current scanner state for iterating over the source input.
pub struct Scanner<'src> {
    source: &'src str,
    scan_offset: usize, // start offset for piecewise scanning
    line: usize,
    start: usize,
    current: usize,
    tokens: Vec<Token>,
    keywords: HashMap<&'static str, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, scan_offset: usize) -> Self {
        Self {
            source,
            scan_offset,
            line: 1,
            current: 0,
            start: 0,
            tokens: vec![],
            keywords: hashmap! {
                "and" => TokenType::KwAnd,
                "class" => TokenType::KwClass,
                "else" => TokenType::KwElse,
                "false" => TokenType::KwFalse,
                "for" => TokenType::KwFor,
                "fun" => TokenType::KwFun,
                "if" => TokenType::KwIf,
                "nil" => TokenType::KwNil,
                "or" => TokenType::KwOr,
                "print" => TokenType::KwPrint,
                "return" => TokenType::KwReturn,
                "super" => TokenType::KwSuper,
                "this" => TokenType::KwThis,
                "true" => TokenType::KwTrue,
                "var" => TokenType::KwVar,
                "while" => TokenType::KwWhile,
            },
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
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
            '"' => self.string(),
            '0'..='9' => self.number(),
            d if d.is_alphabetic() => self.identifier(),
            ' ' | '\r' | '\t' => {
                // Ignore whitespace.
            }
            '\n' => {
                self.line += 1;
            }
            _ => {
                crate::error(
                    RuntimeError::ScanError {
                        location: self.current_location(),
                    },
                    &format!("Unexpected character `{}`", c),
                );
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
        self.peek_offset(0)
    }

    fn peek_next(&self) -> char {
        self.peek_offset(1)
    }

    // @internal
    fn peek_offset(&self, offset: usize) -> char {
        if self.current + offset >= self.source.len() {
            return '\0';
        }
        self.source
            .chars()
            .nth(self.current + offset)
            .expect("Got past end of input")
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            crate::error(
                RuntimeError::ScanError {
                    location: self.current_location(),
                },
                &format!("Unterminated string starting at {}.", self.start),
            );
            return;
        }
        // The closing ".
        self.advance();

        // Skip " " around the string value.
        let value = &self.source[self.start + 1..self.current - 1];

        self.add_token_with_value(TokenType::String, LiteralValue::Str(value.into()));
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }
        self.add_token_with_value(
            TokenType::Number,
            LiteralValue::Num(self.source[self.start..self.current].parse().expect("TODO")),
        );
    }

    fn identifier(&mut self) {
        while self.peek().is_identifier() {
            self.advance();
        }

        let lexeme = self.lexeme();
        if self.keywords.contains_key(&lexeme) {
            self.add_token(self.keywords[&lexeme]);
        } else {
            self.add_token(TokenType::Identifier);
        }
    }

    fn lexeme(&self) -> &str {
        &self.source[self.start..self.current]
    }

    fn current_location(&self) -> SourcePosition {
        SourcePosition {
            line: self.line,
            span: self.start + self.scan_offset..self.current + self.scan_offset,
        }
    }

    fn add_token(&mut self, r#type: TokenType) {
        self.tokens
            .push(Token::new(r#type, self.current_location(), None));
    }

    fn add_token_with_value(&mut self, r#type: TokenType, value: LiteralValue) {
        self.tokens
            .push(Token::new(r#type, self.current_location(), Some(value)));
    }
}

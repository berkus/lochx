use {
    crate::{error::RuntimeError, literal::LiteralValue, runtime},
    small_map::SmallMap,
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
    Eof,

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
    source: &'src str,                               // Utf8 source
    scan_offset: usize,                              // Start offset for piecewise scanning
    line: usize,                                     // Current line number
    start_byte: usize,                               // Byte position inside the utf8 source
    current_byte: usize,                             // Byte position inside the utf8 source
    current_char: usize,                             // Char position inside the utf8 source
    tokens: Vec<Token>,                              // List of collected tokens
    keywords: SmallMap<16, &'static str, TokenType>, // List of recognized keywords
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, scan_offset: usize) -> Self {
        let words = [
            ("and", TokenType::KwAnd),
            ("class", TokenType::KwClass),
            ("else", TokenType::KwElse),
            ("false", TokenType::KwFalse),
            ("for", TokenType::KwFor),
            ("fun", TokenType::KwFun),
            ("if", TokenType::KwIf),
            ("nil", TokenType::KwNil),
            ("or", TokenType::KwOr),
            ("print", TokenType::KwPrint),
            ("return", TokenType::KwReturn),
            ("super", TokenType::KwSuper),
            ("this", TokenType::KwThis),
            ("true", TokenType::KwTrue),
            ("var", TokenType::KwVar),
            ("while", TokenType::KwWhile),
        ];
        let mut keywords = SmallMap::with_capacity(words.len());
        for w in words {
            keywords.insert(w.0, w.1);
        }
        Self {
            source,
            scan_offset,
            line: 1,
            current_char: 0,
            current_byte: 0,
            start_byte: 0,
            tokens: vec![],
            keywords,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start_byte = self.current_byte;
            self.scan_token();
        }
        self.add_token(TokenType::Eof);
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
        self.current_byte >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self
            .source
            .chars()
            .nth(self.current_char)
            .expect("Got past end of input in advance");
        self.current_char += 1;
        self.current_byte += c.len_utf8();
        c
    }

    /// Return true and advance if the next character is the expected one.
    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current_char) != Some(expected) {
            return false;
        }
        self.current_char += 1;
        self.current_byte += expected.len_utf8();
        true
    }

    fn peek(&self) -> char {
        self.peek_offset(0)
    }

    fn peek_next(&self) -> char {
        self.peek_offset(1)
    }

    // @internal
    fn peek_offset(&self, byte_and_char_offset: usize) -> char {
        // @fixme broken
        if self.current_byte + byte_and_char_offset >= self.source.len() {
            return '\0';
        }
        self.source
            .chars()
            .nth(self.current_char + byte_and_char_offset) // @fixme broken
            .expect("Got past end of input in peek_offset")
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
                &format!("Unterminated string starting at {}.", self.start_byte),
            );
            return;
        }
        // The closing ".
        self.advance();

        // Skip " " around the string value.
        let value = &self.source[self.start_byte + 1..self.current_byte - 1];

        self.add_token_with_value(TokenType::String, LiteralValue::Str(value.into()));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        self.add_token_with_value(
            TokenType::Number,
            LiteralValue::Num(
                self.source[self.start_byte..self.current_byte]
                    .parse()
                    .expect("TODO"),
            ),
        );
    }

    fn identifier(&mut self) {
        while self.peek().is_identifier() {
            self.advance();
        }

        let token = if let Some(&v) = self.keywords.get(self.lexeme()) {
            v
        } else {
            TokenType::Identifier
        };
        self.add_token(token);
    }

    fn lexeme(&self) -> &str {
        &self.source[self.start_byte..self.current_byte]
    }

    fn current_location(&self) -> SourcePosition {
        SourcePosition {
            line: self.line,
            span: self.start_byte + self.scan_offset..self.current_byte + self.scan_offset,
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

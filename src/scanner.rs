use {crate::error, maplit::hashmap, std::collections::HashMap};

/// Current scanner state for iterating over the source input.
pub struct Scanner<'a> {
    source: &'a str,
    line: usize,
    start: usize,
    current: usize,
    tokens: Vec<Token>,
    keywords: HashMap<&'static str, TokenType>,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType,
    lexeme: String, // @todo: Use Range into a source str (to print error information)
    line: usize,
    literal: Option<LiteralValue>,
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Str(String),
    Num(f64),
    Nil,
    Bool(bool),
}

impl Token {
    pub fn new(
        r#type: TokenType,
        lexeme: &str,
        line: usize,
        literal: Option<LiteralValue>,
    ) -> Self {
        Self {
            r#type,
            lexeme: lexeme.to_string(),
            line,
            literal,
        }
    }

    pub fn lexeme(&self) -> String {
        self.lexeme.clone()
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

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {:?} {}", self.line, self.r#type, self.lexeme)
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
            keywords: hashmap! {
                "and" => TokenType::KwAnd,
                "class" => TokenType::KwClass,
                "else" => TokenType::KwElse,
                "false" => TokenType::KwFalse,
                "for" => TokenType::KfFor,
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
            error(
                self.line,
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
        while self.peek().is_alphanumeric() {
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

    fn add_token(&mut self, r#type: TokenType) {
        let lexeme = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(r#type, lexeme, self.line, None));
    }

    // Ignores lexeme for now, but for debugging we probably want to keep it around anyway?
    fn add_token_with_value(&mut self, r#type: TokenType, value: LiteralValue) {
        self.tokens
            .push(Token::new(r#type, "", self.line, Some(value)));
    }
}

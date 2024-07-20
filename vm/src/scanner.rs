use {
    crate::token::{SourcePosition, Token, TokenKind},
    maplit::hashmap,
    std::collections::HashMap,
};

/// Current scanner state for iterating over the source input.
pub struct Scanner<'src> {
    source: &'src str,                          // Utf8 source
    line: usize,                                // Current line number
    start_byte: usize,                          // Byte position inside the utf8 source
    current_byte: usize,                        // Byte position inside the utf8 source
    current_char: usize,                        // Char position inside the utf8 source
    keywords: HashMap<&'static str, TokenKind>, // List of recognized keywords
}

trait IsIdentifier {
    fn is_identifier(&self) -> bool;
}

impl IsIdentifier for char {
    fn is_identifier(&self) -> bool {
        self.is_alphanumeric() || *self == '_'
    }
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str, _scan_offset: usize) -> Self {
        Self {
            source,
            line: 1,
            current_char: 0,
            current_byte: 0,
            start_byte: 0,
            keywords: hashmap! {
                "and" => TokenKind::KwAnd,
                "class" => TokenKind::KwClass,
                "else" => TokenKind::KwElse,
                "false" => TokenKind::KwFalse,
                "for" => TokenKind::KwFor,
                "fun" => TokenKind::KwFun,
                "if" => TokenKind::KwIf,
                "nil" => TokenKind::KwNil,
                "or" => TokenKind::KwOr,
                "print" => TokenKind::KwPrint,
                "return" => TokenKind::KwReturn,
                "super" => TokenKind::KwSuper,
                "this" => TokenKind::KwThis,
                "true" => TokenKind::KwTrue,
                "var" => TokenKind::KwVar,
                "while" => TokenKind::KwWhile,
            },
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();
        self.start_byte = self.current_byte;
        if self.is_at_end() {
            return self.make_token(TokenKind::Eof);
        }
        let c = self.advance();
        match c {
            '(' => return self.make_token(TokenKind::LeftParen),
            ')' => return self.make_token(TokenKind::RightParen),
            '{' => return self.make_token(TokenKind::LeftBrace),
            '}' => return self.make_token(TokenKind::RightBrace),
            ';' => return self.make_token(TokenKind::Semicolon),
            ',' => return self.make_token(TokenKind::Comma),
            '.' => return self.make_token(TokenKind::Dot),
            '-' => return self.make_token(TokenKind::Minus),
            '+' => return self.make_token(TokenKind::Plus),
            '/' => return self.make_token(TokenKind::Slash),
            '*' => return self.make_token(TokenKind::Star),
            '!' => {
                let kind = if self.matches('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                return self.make_token(kind);
            }
            '=' => {
                let kind = if self.matches('=') {
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equal
                };
                return self.make_token(kind);
            }
            '<' => {
                let kind = if self.matches('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                return self.make_token(kind);
            }
            '>' => {
                let kind = if self.matches('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                return self.make_token(kind);
            }
            '"' => return self.string(),
            '0'..='9' => return self.number(),
            d if d.is_alphabetic() => return self.identifier(),

            _ => return self.error_token("Unexpected character."),
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    // Ignore whitespace.
                    self.advance();
                }
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn string(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }
        // The closing ".
        self.advance();

        self.make_token(TokenKind::String)
    }

    fn number(&mut self) -> Token {
        while self.peek().is_ascii_digit() {
            self.advance();
        }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }
        self.make_token(TokenKind::Number)
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_identifier() {
            self.advance();
        }

        let lexeme = &self.source[self.start_byte..self.current_byte];
        if self.keywords.contains_key(&lexeme) {
            self.make_token(self.keywords[&lexeme])
        } else {
            self.make_token(TokenKind::Identifier)
        }
    }

    fn make_token(&self, kind: TokenKind) -> Token {
        Token {
            kind,
            position: self.current_position(),
        }
    }

    fn error_token(&self, error: &'static str) -> Token {
        Token {
            kind: TokenKind::Error(error),
            position: self.current_position(),
        }
    }

    fn current_position(&self) -> SourcePosition {
        SourcePosition {
            line: self.line,
            span: self.start_byte..self.current_byte,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current_byte >= self.source.len()
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
}

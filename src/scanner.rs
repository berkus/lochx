pub struct Scanner<'a> {
    source: &'a str,
    line: usize,
    start: usize,
    current: usize,
}

#[derive(Debug)]
pub enum Token {
    EOF,

    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals
    Identifier, String, Number,

    // Keywords
    KwAnd, KwClass, KwElse, KwFalse, KwFun, KfFor, KwIf, KwNil, KwOr,
    KwPrint, KwReturn, KwSuper, KwThis, KwTrue, KwVar, KwWhile,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            line: 1,
            current: 0,
            start: 0
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        let mut tokens = vec![];
        while !self.is_at_end() {
            self.start = self.current;
            tokens.push(self.scan_token());
        }
        tokens.push(Token::EOF);
        tokens
    }

    fn scan_token(&mut self) -> Token {
        let c = self.advance();
        match c {
            '(' => return Token::LeftParen,
            ')' => return Token::RightParen,
            '{' => return Token::LeftBrace,
            '}' => return Token::RightBrace,
            ',' => return Token::Comma,
            '.' => return Token::Dot,
            '-' => return Token::Minus,
            '+' => return Token::Plus,
            ';' => return Token::Semicolon,
            '*' => return Token::Star,
            _ => todo!(),
        }
        Token::EOF // @todo
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current);
        self.current += 1;
        c.expect("Got past end of input")
    }
}

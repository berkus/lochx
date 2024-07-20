use {
    crate::{
        chunk::Chunk,
        scanner::Scanner,
        token::{Token, TokenKind},
    },
    culpa::throws,
};

// Compiler?
struct Parser {
    previous: Token,
    current: Token,
    had_error: bool,
}

impl Parser {
    #[throws(InterpretError)]
    pub fn compile(&mut self, source: &str, chunk: &mut Chunk) -> bool {
        let scanner = Scanner::new(source, 0);
        self.advance();
        self.expression();
        self.consume(TokenKind::Eof, "Expected end of expression");
        !self.had_error
    }

    fn advance(&mut self) {
        self.previous = self.current;
        loop {
            self.current = scanner.scan_token();
            if let TokenKind::Error(message) = self.current.kind {
                self.error_at_current(message);
            } else {
                return;
            }
        }
    }

    fn expression() {}

    fn consume(&mut self, t: TokenKind, message: &'static str) {
        if self.current.kind == t {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn error_at_current(&mut self, message: &'static str) {
        let token = self.current.clone();
        self.error_at(&token, message);
    }

    fn error(&mut self, message: &'static str) {
        let token = self.previous.clone();
        self.error_at(&token, message);
    }

    fn error_at(&mut self, token: &Token, message: &'static str) {
        eprint!("[line {}] Error", token.position.line);

        match token.kind {
            TokenKind::Eof => eprint!(" at end"),
            TokenKind::Error(_) => {}
            _ => eprint!(" at '{}'", token.position), // must be lexeme
        }

        eprintln!(": {}", message);
        self.had_error = true;
    }
}

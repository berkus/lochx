use {
    crate::{
        expr::{self, Expr},
        scanner::{LiteralValue, Token, TokenType},
        stmt::Stmt,
    },
    anyhow::{anyhow, Error},
    culpa::{throw, throws},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

/// Recursive descent parser for the Lox grammar:
/// ```text
/// program        → statement* EOF ;
/// statement      → exprStmt
///                | printStmt ;
/// exprStmt       → expression ";" ;
/// printStmt      → "print" expression ";" ;
/// expression     → equality ;
/// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
/// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term           → factor ( ( "-" | "+" ) factor )* ;
/// factor         → unary ( ( "/" | "*" ) unary )* ;
/// unary          → ( "!" | "-" ) unary
///                | primary ;
/// primary        → NUMBER | STRING | "true" | "false" | "nil"
///                | "(" expression ")" ;
/// ```
/// Grammar productions are in order of increasing precedence from top to bottom.
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    #[throws]
    pub fn parse(&mut self) -> Vec<Stmt> {
        self.program()?
    }

    #[throws]
    fn program(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        statements
    }

    #[throws]
    fn statement(&mut self) -> Stmt {
        if self.match_any(vec![TokenType::KwPrint]) {
            return self.print_stmt()?;
        }
        self.expr_stmt()?
    }

    #[throws]
    fn print_stmt(&mut self) -> Stmt {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Stmt::Print(expr)
    }

    #[throws]
    fn expr_stmt(&mut self) -> Stmt {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.")?;
        Stmt::Expression(expr)
    }

    #[throws]
    fn expression(&mut self) -> Expr {
        self.equality()?
    }

    #[throws]
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison()?;

        while self.match_any(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary(expr::Binary {
                op: op.clone(),
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        expr
    }

    #[throws]
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term()?;

        while self.match_any(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let right = self.term()?;
            expr = Expr::Binary(expr::Binary {
                op: op.clone(),
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        expr
    }

    #[throws]
    fn term(&mut self) -> Expr {
        let mut expr = self.factor()?;

        while self.match_any(vec![TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(expr::Binary {
                op: op.clone(),
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        expr
    }

    #[throws]
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary()?;

        while self.match_any(vec![TokenType::Slash, TokenType::Star]) {
            let op = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(expr::Binary {
                op: op.clone(),
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        expr
    }

    #[throws]
    fn unary(&mut self) -> Expr {
        if self.match_any(vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous();
            let right = self.unary()?;
            return Expr::Unary(expr::Unary {
                op: op.clone(),
                right: Box::new(right),
            });
        }

        self.primary()?
    }

    #[throws]
    fn primary(&mut self) -> Expr {
        if self.match_any(vec![TokenType::KwFalse]) {
            return Expr::Literal(expr::Literal {
                value: LiteralValue::Bool(false),
            });
        }
        if self.match_any(vec![TokenType::KwTrue]) {
            return Expr::Literal(expr::Literal {
                value: LiteralValue::Bool(true),
            });
        }
        if self.match_any(vec![TokenType::KwNil]) {
            return Expr::Literal(expr::Literal {
                value: LiteralValue::Nil,
            });
        }
        if self.match_any(vec![TokenType::Number]) {
            return Expr::Literal(expr::Literal {
                value: LiteralValue::Num(
                    self.previous()
                        .literal_num()
                        .expect("We got a numeric literal"),
                ),
            });
        }
        if self.match_any(vec![TokenType::String]) {
            return Expr::Literal(expr::Literal {
                value: LiteralValue::Str(
                    self.previous()
                        .literal_str()
                        .expect("We got a string literal"),
                ),
            });
        }
        if self.check(TokenType::LeftParen) {
            self.advance();
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            return Expr::Grouping(expr::Grouping {
                expr: Box::new(expr),
            });
        }
        // @todo Throw ParseError with location info
        throw!(anyhow!("Expected expression"));
    }

    fn match_any(&mut self, types: Vec<TokenType>) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    // @todo Throw ParseError with location info
    #[throws]
    fn consume(&mut self, t: TokenType, message: &str) {
        if self.check(t) {
            return self.advance();
        }
        throw!(anyhow!(
            "{} (expected {:?}, got {})",
            message.to_string(),
            t,
            self.peek()
        ));
    }

    /// Synchronize parser stream to the next non-error token.
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().r#type == TokenType::Semicolon {
                return;
            }

            match self.peek().r#type {
                TokenType::KwClass
                | TokenType::KwFun
                | TokenType::KwFor
                | TokenType::KwIf
                | TokenType::KwPrint
                | TokenType::KwReturn
                | TokenType::KwVar
                | TokenType::KwWhile => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn check(&self, t: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().r#type == t
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous();
    }

    fn is_at_end(&self) -> bool {
        self.peek().r#type == TokenType::EOF
    }

    // Don't borrow here to make code simpler, for speed we should get back to borrowing
    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    // Don't borrow here to make code simpler, for speed we should get back to borrowing
    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}

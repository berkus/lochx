use {
    crate::{
        callable,
        environment::EnvironmentImpl,
        error::RuntimeError,
        expr::{self, Expr},
        literal::LiteralValue,
        scanner::{Token, TokenType},
        stmt::{self, Stmt},
    },
    culpa::{throw, throws},
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

/// Recursive descent parser for the Lox grammar:
/// ```text
/// program        → declaration* EOF ;
/// declaration    → classDecl
///                | funDecl
///                | varDecl
///                | statement ;
/// classDecl      → "class" IDENTIFIER "{" function* "}" ;
/// funDecl        → "fun" function ;
/// function       → IDENTIFIER "(" parameters? ")" block ;
/// varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
/// statement      → exprStmt
///                | forStmt
///                | ifStmt
///                | printStmt
///                | returnStmt
///                | whileStmt
///                | block ;
/// exprStmt       → expression ";" ;
/// forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
///                  expression? ";"
///                  expression? ")" statement ;
/// ifStmt         → "if" "(" expression ")" statement
///                  ("else" statement )? ;
/// returnStmt     → "return" expression? ";" ;
/// printStmt      → "print" expression ";" ;
/// whileStmt      → "while" "(" expression ")" statement ;
/// block          → "{" declaration* "}" ;
/// expression     → assignment ;
/// assignment     → ( call "." )? IDENTIFIER "=" assignment
///                | logic_or ;
/// logic_or       → logic_and ( "or" logic_and )* ;
/// logic_and      → equality ( "and" equality )* ;
/// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
/// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term           → factor ( ( "-" | "+" ) factor )* ;
/// factor         → unary ( ( "/" | "*" ) unary )* ;
/// unary          → ( "!" | "-" ) unary | call ;
/// call           → primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
/// parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
/// arguments      → expression ( "," expression )* ;
/// primary        → NUMBER | STRING | IDENTIFIER | "true" | "false" | "nil"
///                | "(" expression ")" ;
/// ```
/// Grammar productions are in order of increasing precedence from top to bottom.
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    #[throws(RuntimeError)]
    pub fn parse(&mut self) -> Vec<Stmt> {
        self.program()?
    }

    #[throws(RuntimeError)]
    fn program(&mut self) -> Vec<Stmt> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration_with_error_handling()?);
        }
        statements
    }

    #[throws(RuntimeError)]
    fn declaration_with_error_handling(&mut self) -> Stmt {
        let decl = self.declaration();
        if let Err(e) = decl {
            let token = self.peek();
            crate::error(
                RuntimeError::ParseError {
                    token: token.clone(),
                    expected: TokenType::EOF,
                    message: format!("Unexpected declaration. {}", e),
                },
                "Declaration error",
            );
            self.synchronize();
            return Stmt::ParseError { token };
        }
        decl?
    }

    #[throws(RuntimeError)]
    fn declaration(&mut self) -> Stmt {
        if self.match_any(vec![TokenType::KwClass]) {
            return self.class_declaration()?;
        }
        if self.match_any(vec![TokenType::KwFun]) {
            return self.function("function")?;
        }
        if self.match_any(vec![TokenType::KwVar]) {
            return self.var_declaration()?;
        }
        self.statement()?
    }

    #[throws(RuntimeError)]
    fn class_declaration(&mut self) -> Stmt {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;
        let mut methods = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;
        Stmt::Class(stmt::Class { name, methods })
    }

    #[throws(RuntimeError)]
    fn function(&mut self, kind: &'static str) -> Stmt {
        let name = self.consume(
            TokenType::Identifier,
            format!("Expected {kind} name.").as_str(),
        )?;
        self.consume(
            TokenType::LeftParen,
            format!("Expected '(' after {kind} name.").as_str(),
        )?;

        let mut parameters = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() > 255 {
                    throw!(RuntimeError::TooManyArguments(self.peek())) // @todo TooManyParameters
                }
                parameters.push(self.consume(TokenType::Identifier, "Expected parameter name.")?);
                if !self.match_any(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            format!("Expected ')' after {kind} parameters.").as_str(),
        )?;

        self.consume(
            TokenType::LeftBrace,
            format!("Expected '{{' before {kind} body.").as_str(),
        )?;
        let body = self.block()?;
        let closure = EnvironmentImpl::new(); // Dummy.
        Stmt::FunctionDecl(callable::Function {
            name,
            parameters,
            body,
            closure,
        })
    }

    #[throws(RuntimeError)]
    fn var_declaration(&mut self) -> Stmt {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?;
        let initializer = if self.match_any(vec![TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(expr::Literal {
                value: LiteralValue::Nil,
            })
        };
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        )?;
        Stmt::VarDecl(stmt::VarDecl { name, initializer })
    }

    #[throws(RuntimeError)]
    fn statement(&mut self) -> Stmt {
        if self.match_any(vec![TokenType::KwFor]) {
            return self.for_stmt()?;
        }
        if self.match_any(vec![TokenType::KwIf]) {
            return self.if_stmt()?;
        }
        if self.match_any(vec![TokenType::KwPrint]) {
            return self.print_stmt()?;
        }
        if self.match_any(vec![TokenType::KwReturn]) {
            return self.return_stmt()?;
        }
        if self.match_any(vec![TokenType::KwWhile]) {
            return self.while_stmt()?;
        }
        if self.match_any(vec![TokenType::LeftBrace]) {
            return self.block_stmt()?;
        }
        self.expr_stmt()?
    }

    #[throws(RuntimeError)]
    fn for_stmt(&mut self) -> Stmt {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;
        let initializer = if self.match_any(vec![TokenType::Semicolon]) {
            None
        } else if self.match_any(vec![TokenType::KwVar]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_stmt()?)
        };
        let condition = if !self.match_any(vec![TokenType::Semicolon]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            TokenType::Semicolon,
            "Expected ';' after for loop condition.",
        )?;
        let increment = if !self.match_any(vec![TokenType::RightParen]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expected ')' after for clauses.")?;
        let body = self.statement()?;

        // Desugar into a while loop:
        // {
        //   initializer;
        //   while (condition) {
        //     body;
        //     increment;
        //   }
        // }
        let body = if let Some(increment) = increment {
            Stmt::Block(vec![body, Stmt::Expression(increment)])
        } else {
            body
        };
        let condition = if let Some(condition) = condition {
            condition
        } else {
            Expr::Literal(expr::Literal {
                value: LiteralValue::Bool(true),
            })
        };

        let body = Stmt::While(stmt::WhileStmt {
            condition,
            body: Box::new(body),
        });

        let body = if let Some(initializer) = initializer {
            Stmt::Block(vec![initializer, body])
        } else {
            body
        };
        body
    }

    #[throws(RuntimeError)]
    fn if_stmt(&mut self) -> Stmt {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.")?;
        let expr = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after 'if' condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_any(vec![TokenType::KwElse]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Stmt::If(stmt::IfStmt {
            condition: expr,
            then_branch,
            else_branch,
        })
    }

    #[throws(RuntimeError)]
    fn print_stmt(&mut self) -> Stmt {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Stmt::Print(expr)
    }

    #[throws(RuntimeError)]
    fn return_stmt(&mut self) -> Stmt {
        let keyword = self.previous();
        let value = if !self.check(TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(expr::Literal {
                value: LiteralValue::Nil,
            })
        };
        self.consume(TokenType::Semicolon, "Expected ';' after return value.")?;
        Stmt::Return(stmt::Return { keyword, value })
    }

    #[throws(RuntimeError)]
    fn while_stmt(&mut self) -> Stmt {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expected ')' after 'while' condition.",
        )?;
        let body = Box::new(self.statement()?);
        Stmt::While(stmt::WhileStmt { condition, body })
    }

    #[throws(RuntimeError)]
    fn expr_stmt(&mut self) -> Stmt {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Stmt::Expression(expr)
    }

    // Wrap block into the block statement.
    #[throws(RuntimeError)]
    fn block_stmt(&mut self) -> Stmt {
        Stmt::Block(self.block()?)
    }

    // Shared block parser, will be reused for function bodies.
    #[throws(RuntimeError)]
    fn block(&mut self) -> Vec<Stmt> {
        let mut stmts = vec![];
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration_with_error_handling()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        stmts
    }

    #[throws(RuntimeError)]
    fn expression(&mut self) -> Expr {
        self.assignment()?
    }

    #[throws(RuntimeError)]
    fn assignment(&mut self) -> Expr {
        let expr = self.logic_or()?;
        if self.match_any(vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;
            match expr {
                Expr::Variable(expr::Var { name, .. }) => {
                    return Expr::Assign(expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                }
                Expr::Get(expr::Getter { name, object }) => {
                    return Expr::Set(expr::Setter {
                        name,
                        object,
                        value: Box::new(value),
                    })
                }
                _ => {
                    throw!(RuntimeError::InvalidAssignmentTarget(
                        equals,
                        "Can't assign to non-variable"
                    ))
                }
            }
        }
        expr
    }

    #[throws(RuntimeError)]
    fn logic_or(&mut self) -> Expr {
        let mut expr = self.logic_and()?;

        while self.match_any(vec![TokenType::KwOr]) {
            let op = self.previous();
            let right = self.logic_and()?;
            expr = Expr::Logical(expr::Logical {
                op: op.clone(),
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        expr
    }

    #[throws(RuntimeError)]
    fn logic_and(&mut self) -> Expr {
        let mut expr = self.equality()?;

        while self.match_any(vec![TokenType::KwAnd]) {
            let op = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(expr::Logical {
                op: op.clone(),
                left: Box::new(expr),
                right: Box::new(right),
            });
        }

        expr
    }

    #[throws(RuntimeError)]
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

    #[throws(RuntimeError)]
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

    #[throws(RuntimeError)]
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

    #[throws(RuntimeError)]
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

    #[throws(RuntimeError)]
    fn unary(&mut self) -> Expr {
        if self.match_any(vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous();
            let right = self.unary()?;
            return Expr::Unary(expr::Unary {
                op: op.clone(),
                right: Box::new(right),
            });
        }

        self.call()?
    }

    #[throws(RuntimeError)]
    fn call(&mut self) -> Expr {
        let mut expr = self.primary()?;

        loop {
            if self.match_any(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_any(vec![TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier, "Expect property name after '.'")?;
                expr = Expr::Get(expr::Getter {
                    name,
                    object: Box::new(expr),
                });
            } else {
                break;
            }
        }

        expr
    }

    #[throws(RuntimeError)]
    fn finish_call(&mut self, callee: Expr) -> Expr {
        let mut arguments = vec![];
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() > 255 {
                    throw!(RuntimeError::TooManyArguments(self.peek()))
                }
                arguments.push(self.expression()?);
                if !self.match_any(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expected ')' after arguments.")?;

        Expr::Call(expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    #[throws(RuntimeError)]
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
        if self.match_any(vec![TokenType::KwThis]) {
            return Expr::This(expr::This {
                keyword: self.previous(),
            });
        }
        if self.match_any(vec![TokenType::Identifier]) {
            return Expr::Variable(expr::Var {
                name: self.previous().clone(),
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
        throw!(RuntimeError::ExpectedExpression(self.peek()));
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

    #[throws(RuntimeError)]
    fn consume(&mut self, t: TokenType, message: &str) -> Token {
        if self.check(t) {
            return self.advance();
        }
        throw!(RuntimeError::ParseError {
            token: self.peek(),
            expected: t,
            message: message.into()
        });
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

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
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

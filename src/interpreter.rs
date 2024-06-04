use {
    crate::{
        callable::{self, Callable},
        environment::Environment,
        error::RuntimeError,
        expr::{self, Acceptor as ExprAcceptor, Expr},
        literal::{LiteralValue, LochxCallable},
        scanner::TokenType,
        stmt::{self, Acceptor as StmtAcceptor, Stmt},
    },
    core::cell::RefCell,
    culpa::{throw, throws},
    liso::{liso, OutputOnly},
    std::rc::Rc,
};

pub struct Interpreter {
    out: OutputOnly,
    pub(super) globals: Rc<RefCell<Environment>>,
    current_env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new(out: OutputOnly) -> Self {
        let env = Environment::new();
        env.borrow_mut().define(
            "clock".into(),
            LiteralValue::Callable(LochxCallable::NativeFunction(Box::new(
                callable::NativeFunction {
                    arity: 0,
                    body: callable::clock,
                },
            ))),
        );
        Self {
            out,
            globals: env.clone(),
            current_env: env,
        }
    }

    #[throws(RuntimeError)]
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            self.execute(&stmt)?;
        }
    }

    #[throws(RuntimeError)]
    fn execute(&mut self, stmt: &Stmt) {
        stmt.accept(self)?;
    }

    #[throws(RuntimeError)]
    pub(super) fn execute_block(&mut self, stmts: Vec<Stmt>, env: Rc<RefCell<Environment>>) {
        let previous = self.current_env.clone();
        self.current_env = env;
        for stmt in stmts {
            if let Err(e) = self.execute(&stmt) {
                self.current_env = previous;
                throw!(e);
            }
        }
        self.current_env = previous;
    }

    #[throws(RuntimeError)]
    fn evaluate(&mut self, expr: &Expr) -> LiteralValue {
        expr.accept(self)?
    }
}

impl stmt::Visitor for Interpreter {
    type ReturnType = ();

    #[throws(RuntimeError)]
    fn visit_print_stmt(&mut self, stmt: &Expr) -> Self::ReturnType {
        let expr = self.evaluate(stmt)?;
        self.out
            .wrapln(liso!(fg = magenta, format!("{}", expr), reset));
    }

    #[throws(RuntimeError)]
    fn visit_expression_stmt(&mut self, stmt: &Expr) -> Self::ReturnType {
        self.evaluate(stmt)?;
    }

    #[throws(RuntimeError)]
    fn visit_vardecl_stmt(&mut self, stmt: &stmt::VarDecl) -> Self::ReturnType {
        let value = self.evaluate(&stmt.initializer)?;
        self.current_env
            .borrow_mut()
            .define(stmt.name.lexeme().clone(), value);
    }

    #[throws(RuntimeError)]
    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> Self::ReturnType {
        self.execute_block(
            stmts.to_vec(),
            Environment::nested(self.current_env.clone()),
        )?;
    }

    #[throws(RuntimeError)]
    fn visit_if_stmt(&mut self, stmt: &stmt::IfStmt) -> Self::ReturnType {
        let expr = self.evaluate(&stmt.condition)?;
        if expr.is_truthy() {
            self.execute(stmt.then_branch.as_ref())?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }
    }

    #[throws(RuntimeError)]
    fn visit_while_stmt(&mut self, stmt: &stmt::WhileStmt) -> Self::ReturnType {
        while self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(stmt.body.as_ref())?;
        }
    }

    #[throws(RuntimeError)]
    fn visit_fundecl_stmt(&mut self, stmt: &callable::Function) -> Self::ReturnType {
        self.current_env.borrow_mut().define(
            stmt.name.lexeme(),
            LiteralValue::Callable(LochxCallable::Function(Box::new(stmt.clone()))),
        );
    }

    #[throws(RuntimeError)]
    fn visit_return_stmt(&mut self, stmt: &stmt::Return) -> Self::ReturnType {
        throw!(RuntimeError::ReturnValue(self.evaluate(&stmt.value)?))
    }
}

impl expr::Visitor for Interpreter {
    type ReturnType = LiteralValue;

    #[throws(RuntimeError)]
    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Self::ReturnType {
        let left = self.evaluate(expr.left.as_ref())?;
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.op.r#type {
            TokenType::Plus => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l + r),
                (LiteralValue::Str(l), LiteralValue::Str(r)) => LiteralValue::Str(l + &r),
                _ => todo!(),
            },
            TokenType::Minus => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l - r),
                _ => todo!(),
            },
            TokenType::Star => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l * r),
                _ => todo!(),
            },
            TokenType::Slash => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l / r),
                _ => todo!(),
            },
            TokenType::Greater => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l > r),
                _ => todo!(),
            },
            TokenType::GreaterEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l >= r),
                _ => todo!(),
            },
            TokenType::Less => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l < r),
                _ => todo!(),
            },
            TokenType::LessEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l <= r),
                _ => todo!(),
            },
            TokenType::BangEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l != r),
                (LiteralValue::Str(l), LiteralValue::Str(r)) => LiteralValue::Bool(l != r),
                _ => LiteralValue::Bool(true),
            },
            TokenType::EqualEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l == r),
                (LiteralValue::Str(l), LiteralValue::Str(r)) => LiteralValue::Bool(l == r),
                _ => LiteralValue::Bool(false),
            },
            _ => unimplemented!(),
        }
    }

    #[throws(RuntimeError)]
    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Self::ReturnType {
        let right = self.evaluate(expr.right.as_ref())?;
        match expr.op.r#type {
            TokenType::Minus => match right {
                LiteralValue::Num(n) => LiteralValue::Num(-n),
                _ => todo!(),
            },
            TokenType::Bang => LiteralValue::Bool(!right.is_truthy()),
            _ => unreachable!(),
        }
    }

    #[throws(RuntimeError)]
    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Self::ReturnType {
        self.evaluate(expr.expr.as_ref())?
    }

    #[throws(RuntimeError)]
    fn visit_literal_expr(&self, expr: &expr::Literal) -> Self::ReturnType {
        expr.value.clone()
    }

    #[throws(RuntimeError)]
    fn visit_var_expr(&self, expr: &expr::Var) -> Self::ReturnType {
        self.current_env.borrow().get(expr.name.clone())?
    }

    #[throws(RuntimeError)]
    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Self::ReturnType {
        let value = self.evaluate(expr.value.as_ref())?;
        self.current_env
            .borrow_mut()
            .assign(expr.name.clone(), value.clone())?;
        value
    }

    #[throws(RuntimeError)]
    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Self::ReturnType {
        let left = self.evaluate(expr.left.as_ref())?;

        if expr.op.r#type == TokenType::KwOr {
            if left.is_truthy() {
                return left;
            }
        } else {
            if !left.is_truthy() {
                return left;
            }
        }

        self.evaluate(expr.right.as_ref())?
    }

    #[throws(RuntimeError)]
    fn visit_call_expr(&mut self, expr: &expr::Call) -> Self::ReturnType {
        let callee = self.evaluate(expr.callee.as_ref())?;

        match callee {
            LiteralValue::Callable(callable) => {
                let callable = match callable {
                    LochxCallable::Function(f) => f as Box<dyn Callable>,
                    LochxCallable::NativeFunction(f) => f as Box<dyn Callable>,
                };

                if expr.arguments.len() != callable.arity() {
                    throw!(RuntimeError::InvalidArity(
                        expr.paren.clone(),
                        callable.arity(),
                        expr.arguments.len()
                    ))
                }

                let mut arguments = vec![];
                for arg in expr.arguments.iter() {
                    arguments.push(self.evaluate(arg)?);
                }
                return callable.call(self, arguments)?;
            }
            _ => throw!(RuntimeError::NotACallable),
        };
    }
}

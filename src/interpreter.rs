use {
    crate::{
        environment::Environment,
        error::RuntimeError,
        expr::{self, Acceptor as ExprAcceptor, Expr},
        scanner::{LiteralValue, TokenType},
        stmt::{self, Acceptor as StmtAcceptor, Stmt},
    },
    culpa::throws,
    liso::{liso, OutputOnly},
};

pub struct Interpreter {
    out: OutputOnly,
    env: Environment,
}

impl Interpreter {
    pub fn new(out: OutputOnly) -> Self {
        Self {
            out,
            env: Environment::default(),
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
    fn evaluate(&self, expr: &Expr) -> LiteralValue {
        expr.accept(self)?
    }

    /// nil and false are falsy, everything else is truthy
    fn is_truthy(&self, value: &LiteralValue) -> bool {
        match value {
            LiteralValue::Nil => false,
            LiteralValue::Bool(b) => *b,
            _ => true,
        }
    }
}

impl stmt::Visitor for Interpreter {
    type ReturnType = ();

    #[throws(RuntimeError)]
    fn visit_print_stmt(&self, stmt: &Expr) -> Self::ReturnType {
        let expr = self.evaluate(stmt)?;
        self.out
            .wrapln(liso!(fg = magenta, format!("{}", expr), reset));
    }

    #[throws(RuntimeError)]
    fn visit_expression_stmt(&self, stmt: &Expr) -> Self::ReturnType {
        self.evaluate(stmt)?;
    }

    #[throws(RuntimeError)]
    fn visit_vardecl_stmt(&mut self, stmt: &stmt::VarDecl) -> Self::ReturnType {
        let value = self.evaluate(&stmt.initializer)?;
        self.env.define(stmt.name.lexeme().clone(), value);
    }
}

impl expr::Visitor for Interpreter {
    type ReturnType = LiteralValue;

    #[throws(RuntimeError)]
    fn visit_binary_expr(&self, expr: &expr::Binary) -> Self::ReturnType {
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
    fn visit_unary_expr(&self, expr: &expr::Unary) -> Self::ReturnType {
        let right = self.evaluate(expr.right.as_ref())?;
        match expr.op.r#type {
            TokenType::Minus => match right {
                LiteralValue::Num(n) => LiteralValue::Num(-n),
                _ => todo!(),
            },
            TokenType::Bang => LiteralValue::Bool(!self.is_truthy(&right)),
            _ => unreachable!(),
        }
    }

    #[throws(RuntimeError)]
    fn visit_grouping_expr(&self, expr: &expr::Grouping) -> Self::ReturnType {
        self.evaluate(expr.expr.as_ref())?
    }

    #[throws(RuntimeError)]
    fn visit_literal_expr(&self, expr: &expr::Literal) -> Self::ReturnType {
        expr.value.clone()
    }

    #[throws(RuntimeError)]
    fn visit_var_expr(&self, expr: &expr::Var) -> Self::ReturnType {
        self.env.get(expr.name.clone())?
    }
}

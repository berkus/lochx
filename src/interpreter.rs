use crate::{
    expr::{self, Acceptor, Expr},
    scanner::{LiteralValue, TokenType},
};

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&self, expr: &Expr) -> LiteralValue {
        self.evaluate(expr)
    }

    fn evaluate(&self, expr: &Expr) -> LiteralValue {
        expr.accept(self)
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

impl expr::Visitor for Interpreter {
    type ReturnType = LiteralValue;

    fn visit_binary_expr(&self, expr: &expr::Binary) -> Self::ReturnType {
        let left = self.evaluate(expr.left.as_ref());
        let right = self.evaluate(expr.right.as_ref());

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

    fn visit_unary_expr(&self, expr: &expr::Unary) -> Self::ReturnType {
        let right = self.evaluate(expr.right.as_ref());
        match expr.op.r#type {
            TokenType::Minus => match right {
                LiteralValue::Num(n) => LiteralValue::Num(-n),
                _ => todo!(),
            },
            TokenType::Bang => LiteralValue::Bool(!self.is_truthy(&right)),
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(&self, expr: &expr::Grouping) -> Self::ReturnType {
        self.evaluate(expr.expr.as_ref())
    }

    fn visit_literal_expr(&self, expr: &expr::Literal) -> Self::ReturnType {
        expr.value.clone()
    }
}

use crate::{
    expr::{self, Acceptor, Expr},
    scanner::LiteralValue,
};

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn print(&self, e: &Expr) -> String {
        e.accept(self)
    }

    fn parenthesize(&self, name: String, exprs: Vec<Box<Expr>>) -> String {
        let mut s = "(".to_string() + &name;
        for expr in exprs {
            s += " ";
            s += &expr.accept(self);
        }
        s += ")";
        s
    }
}

impl expr::Visitor for AstPrinter {
    type ReturnType = String;

    fn visit_binary_expr(&self, expr: &expr::Binary) -> Self::ReturnType {
        self.parenthesize(
            expr.op.lexeme(),
            vec![expr.left.clone(), expr.right.clone()],
        )
    }

    fn visit_unary_expr(&self, expr: &expr::Unary) -> Self::ReturnType {
        self.parenthesize(expr.op.lexeme(), vec![expr.right.clone()])
    }

    fn visit_grouping_expr(&self, expr: &expr::Grouping) -> Self::ReturnType {
        self.parenthesize("group".to_string(), vec![expr.expr.clone()])
    }

    fn visit_literal_expr(&self, expr: &expr::Literal) -> Self::ReturnType {
        match expr.value.clone() {
            LiteralValue::Num(n) => format!("{}", n),
            LiteralValue::Str(s) => format!("\"{}\"", s),
            LiteralValue::Nil => "nil".to_string(),
            LiteralValue::Bool(b) => {
                if b {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
        }
    }
}

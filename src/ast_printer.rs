use {
    crate::{
        error::RuntimeError,
        expr::{self, Acceptor as ExprAcceptor, Expr},
        scanner::LiteralValue,
        stmt::{self, Acceptor as StmtAcceptor, Stmt},
    },
    culpa::throws,
};

pub struct AstPrinter;

impl AstPrinter {
    pub fn new() -> Self {
        Self {}
    }

    #[throws(RuntimeError)]
    pub fn print_expr(&self, e: &Expr) -> String {
        e.accept(self)?
    }

    #[throws(RuntimeError)]
    pub fn print_stmt(&mut self, statements: Vec<Stmt>) -> String {
        let mut str = String::new();
        for stmt in statements {
            str += &stmt.accept(self)?
        }
        str
    }

    #[throws(RuntimeError)]
    fn parenthesize(&self, name: String, exprs: Vec<Box<Expr>>) -> String {
        let mut s = "(".to_string() + &name;
        for expr in exprs {
            s += " ";
            s += &expr.accept(self)?;
        }
        s += ")";
        s
    }
}

impl stmt::Visitor for AstPrinter {
    type ReturnType = String;

    #[throws(RuntimeError)]
    fn visit_print_stmt(&self, stmt: &Expr) -> Self::ReturnType {
        format!(
            "{};",
            self.parenthesize("print".into(), vec![Box::new(stmt.clone())])?
        )
    }

    #[throws(RuntimeError)]
    fn visit_expression_stmt(&self, stmt: &Expr) -> Self::ReturnType {
        format!(
            "{};",
            self.parenthesize("".into(), vec![Box::new(stmt.clone())])?
        )
    }
}

impl expr::Visitor for AstPrinter {
    type ReturnType = String;

    #[throws(RuntimeError)]
    fn visit_binary_expr(&self, expr: &expr::Binary) -> Self::ReturnType {
        self.parenthesize(
            expr.op.lexeme(),
            vec![expr.left.clone(), expr.right.clone()],
        )?
    }

    #[throws(RuntimeError)]
    fn visit_unary_expr(&self, expr: &expr::Unary) -> Self::ReturnType {
        self.parenthesize(expr.op.lexeme(), vec![expr.right.clone()])?
    }

    #[throws(RuntimeError)]
    fn visit_grouping_expr(&self, expr: &expr::Grouping) -> Self::ReturnType {
        self.parenthesize("group".to_string(), vec![expr.expr.clone()])?
    }

    #[throws(RuntimeError)]
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

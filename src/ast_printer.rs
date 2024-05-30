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

    #[allow(dead_code)]
    #[throws(RuntimeError)]
    pub fn print_expr(&mut self, e: &Expr) -> String {
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
    fn parenthesize(&mut self, name: String, exprs: Vec<Box<Expr>>) -> String {
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
    fn visit_print_stmt(&mut self, stmt: &Expr) -> Self::ReturnType {
        format!(
            "{};",
            self.parenthesize("print".into(), vec![Box::new(stmt.clone())])?
        )
    }

    #[throws(RuntimeError)]
    fn visit_expression_stmt(&mut self, stmt: &Expr) -> Self::ReturnType {
        format!(
            "{};",
            self.parenthesize("".into(), vec![Box::new(stmt.clone())])?
        )
    }

    #[throws(RuntimeError)]
    fn visit_if_stmt(&mut self, stmt: &stmt::IfStmt) -> Self::ReturnType {
        format!(
            "(if {} {} else {})",
            stmt.condition.accept(self)?,
            stmt.then_branch.accept(self)?,
            stmt.else_branch
                .clone()
                .map_or(Ok("None".into()), |b| b.accept(self))?
        )
    }

    #[throws(RuntimeError)]
    fn visit_vardecl_stmt(&mut self, stmt: &stmt::VarDecl) -> Self::ReturnType {
        format!(
            "var {} = {};",
            stmt.name,
            self.parenthesize("".into(), vec![Box::new(stmt.initializer.clone())])?
        )
    }

    #[throws(RuntimeError)]
    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> Self::ReturnType {
        format!("{{ {} }};", self.print_stmt(stmts.to_vec())?)
    }
}

impl expr::Visitor for AstPrinter {
    type ReturnType = String;

    #[throws(RuntimeError)]
    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Self::ReturnType {
        self.parenthesize(
            expr.op.lexeme(),
            vec![expr.left.clone(), expr.right.clone()],
        )?
    }

    #[throws(RuntimeError)]
    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Self::ReturnType {
        self.parenthesize(expr.op.lexeme(), vec![expr.right.clone()])?
    }

    #[throws(RuntimeError)]
    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Self::ReturnType {
        self.parenthesize("group".to_string(), vec![expr.expr.clone()])?
    }

    #[throws(RuntimeError)]
    fn visit_literal_expr(&self, expr: &expr::Literal) -> Self::ReturnType {
        match expr.value.clone() {
            LiteralValue::Num(n) => format!("{}", n).trim_end_matches(".0").to_string(),
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

    #[throws(RuntimeError)]
    fn visit_var_expr(&self, expr: &expr::Var) -> Self::ReturnType {
        format!("(var {})", expr.name)
    }

    #[throws(RuntimeError)]
    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Self::ReturnType {
        format!("(assign {} <- {:?})", expr.name, expr.value)
    }
}

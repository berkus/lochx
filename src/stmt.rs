use crate::expr::Expr;

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
}

/// Statements visitor.
pub trait Visitor {
    type ReturnType;

    fn visit_print_stmt(&self, stmt: &Expr) -> Self::ReturnType;
    fn visit_expression_stmt(&self, stmt: &Expr) -> Self::ReturnType;
}

/// Statement visitor acceptor.
pub trait Acceptor {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType;
}

impl Acceptor for Stmt {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType {
        match self {
            Stmt::Print(e) => visitor.visit_print_stmt(e),
            Stmt::Expression(e) => visitor.visit_expression_stmt(e),
        }
    }
}

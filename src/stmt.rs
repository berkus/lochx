use crate::{expr::Expr, scanner::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
    VarDecl(VarDecl),
}

#[derive(Debug, Clone)]
struct VarDecl {
    pub name: Token,
    pub initializer: Expr,
}

/// Statements visitor.
pub trait Visitor {
    type ReturnType;

    fn visit_print_stmt(&self, stmt: &Expr) -> Self::ReturnType;
    fn visit_expression_stmt(&self, stmt: &Expr) -> Self::ReturnType;
    fn visit_vardecl_stmt(&self, stmt: &VarDecl) -> Self::ReturnType;
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
            Stmt::VarDecl(d) => visitor.visit_vardecl_stmt(d),
        }
    }
}

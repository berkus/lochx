use {
    crate::{error::RuntimeError, expr::Expr, scanner::Token},
    culpa::throws,
};

#[derive(Debug, Clone)]
pub enum Stmt {
    ParseError, // @todo add erroneous token stream here?
    Print(Expr),
    Expression(Expr),
    VarDecl(VarDecl),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: Token,
    pub initializer: Expr,
}

/// Statements visitor.
pub trait Visitor {
    type ReturnType;

    #[throws(RuntimeError)]
    fn visit_print_stmt(&self, stmt: &Expr) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_expression_stmt(&self, stmt: &Expr) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_vardecl_stmt(&mut self, stmt: &VarDecl) -> Self::ReturnType;
}

/// Statement visitor acceptor.
pub trait Acceptor {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType;
}

impl Acceptor for Stmt {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        match self {
            Stmt::Print(e) => visitor.visit_print_stmt(e)?,
            Stmt::Expression(e) => visitor.visit_expression_stmt(e)?,
            Stmt::VarDecl(d) => visitor.visit_vardecl_stmt(d)?,
            Stmt::ParseError => todo!(),
        }
    }
}

use {
    crate::{error::RuntimeError, literal::LiteralValue, scanner::Token},
    culpa::throws,
};

/// Expressions visitor.
pub trait Visitor {
    type ReturnType;

    // @todo generate this with a make_visitor_trait! macro?
    #[throws(RuntimeError)]
    fn visit_assign_expr(&mut self, expr: &Assign) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_binary_expr(&mut self, expr: &Binary) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_logical_expr(&mut self, expr: &Logical) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_unary_expr(&mut self, expr: &Unary) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_grouping_expr(&mut self, expr: &Grouping) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_literal_expr(&self, expr: &Literal) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_var_expr(&self, expr: &Var) -> Self::ReturnType;
}

/// Expression visitor acceptor.
pub trait Acceptor {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType;
}

/// Expression AST node.
#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Assign),
    Binary(Binary),
    Logical(Logical),
    Unary(Unary),
    Grouping(Grouping),
    Literal(Literal),
    Variable(Var),
}

#[derive(Debug, Clone)]
pub struct Unary {
    pub op: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Box<Expr>,
    pub op: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Logical {
    pub left: Box<Expr>,
    pub op: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Grouping {
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LiteralValue,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub name: Token,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub name: Token,
    pub value: Box<Expr>,
}

impl Acceptor for Expr {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        match self {
            Expr::Assign(e) => e.accept(visitor)?,
            Expr::Binary(e) => e.accept(visitor)?,
            Expr::Logical(e) => e.accept(visitor)?,
            Expr::Unary(e) => e.accept(visitor)?,
            Expr::Grouping(e) => e.accept(visitor)?,
            Expr::Literal(e) => e.accept(visitor)?,
            Expr::Variable(e) => e.accept(visitor)?,
        }
    }
}

impl Acceptor for Assign {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_assign_expr(self)?
    }
}

impl Acceptor for Binary {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_binary_expr(self)?
    }
}

impl Acceptor for Logical {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_logical_expr(self)?
    }
}

impl Acceptor for Unary {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_unary_expr(self)?
    }
}

impl Acceptor for Grouping {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_grouping_expr(self)?
    }
}

impl Acceptor for Literal {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_literal_expr(self)?
    }
}

impl Acceptor for Var {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_var_expr(self)?
    }
}

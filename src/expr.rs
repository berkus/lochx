use crate::scanner::{LiteralValue, Token};

/// Expressions visitor.
pub trait Visitor {
    type ReturnType;

    // @todo generate this with a make_visitor_trait! macro?
    fn visit_binary_expr(&self, expr: &Binary) -> Self::ReturnType;
    fn visit_unary_expr(&self, expr: &Unary) -> Self::ReturnType;
    fn visit_grouping_expr(&self, expr: &Grouping) -> Self::ReturnType;
    fn visit_literal_expr(&self, expr: &Literal) -> Self::ReturnType;
}

/// Expression visitor acceptor.
pub trait Acceptor {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType;
}

/// Expression AST node.
#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Binary(Box<Binary<'a>>),
    Unary(Box<Unary<'a>>),
    Grouping(Box<Grouping<'a>>),
    Literal(Box<Literal>),
}

impl<'a> Acceptor for Expr<'a> {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType {
        match self {
            Expr::Binary(e) => e.accept(visitor),
            Expr::Unary(e) => e.accept(visitor),
            Expr::Grouping(e) => e.accept(visitor),
            Expr::Literal(e) => e.accept(visitor),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binary<'a> {
    pub left: Box<Expr<'a>>,
    pub op: Token<'a>,
    pub right: Box<Expr<'a>>,
}

impl<'a> Acceptor for Binary<'a> {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType {
        visitor.visit_binary_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Unary<'a> {
    pub op: Token<'a>,
    pub right: Box<Expr<'a>>,
}

impl<'a> Acceptor for Unary<'a> {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType {
        visitor.visit_unary_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Grouping<'a> {
    pub expr: Box<Expr<'a>>,
}

impl<'a> Acceptor for Grouping<'a> {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType {
        visitor.visit_grouping_expr(self)
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: LiteralValue,
}

impl Acceptor for Literal {
    fn accept<V: Visitor>(&self, visitor: &V) -> V::ReturnType {
        visitor.visit_literal_expr(self)
    }
}

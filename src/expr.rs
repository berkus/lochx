use {
    crate::{error::RuntimeError, literal::LiteralValue, scanner::Token},
    culpa::throws,
};

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
    Call(Call),
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

#[derive(Debug, Clone)]
pub struct Call {
    pub callee: Box<Expr>,
    pub paren: Token,
    pub arguments: Vec<Expr>,
}

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
    #[throws(RuntimeError)]
    fn visit_call_expr(&mut self, expr: &Call) -> Self::ReturnType;
}

/// Expression visitor acceptor.
pub trait Acceptor {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType;
}

impl Acceptor for Expr {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        match self {
            Expr::Assign(e) => visitor.visit_assign_expr(e)?,
            Expr::Binary(e) => visitor.visit_binary_expr(e)?,
            Expr::Logical(e) => visitor.visit_logical_expr(e)?,
            Expr::Unary(e) => visitor.visit_unary_expr(e)?,
            Expr::Grouping(e) => visitor.visit_grouping_expr(e)?,
            Expr::Literal(e) => visitor.visit_literal_expr(e)?,
            Expr::Variable(e) => visitor.visit_var_expr(e)?,
            Expr::Call(e) => visitor.visit_call_expr(e)?,
        }
    }
}

use {
    crate::{callable::Function, error::RuntimeError, expr::Expr, scanner::Token},
    culpa::throws,
};

#[derive(Debug, Clone)]
pub enum Stmt {
    ParseError, // @todo add erroneous token stream here?
    Print(Expr),
    Expression(Expr),
    VarDecl(VarDecl),
    If(IfStmt),
    While(WhileStmt),
    Block(Vec<Stmt>),
    FunctionDecl(Function),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: Token,
    pub initializer: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Box<Stmt>,
}

/// Statements visitor.
pub trait Visitor {
    type ReturnType;

    #[throws(RuntimeError)]
    fn visit_print_stmt(&mut self, stmt: &Expr) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_expression_stmt(&mut self, stmt: &Expr) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_vardecl_stmt(&mut self, stmt: &VarDecl) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_fundecl_stmt(&mut self, stmt: &Function) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> Self::ReturnType;
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
            Stmt::If(i) => visitor.visit_if_stmt(i)?,
            Stmt::While(w) => visitor.visit_while_stmt(w)?,
            Stmt::VarDecl(d) => visitor.visit_vardecl_stmt(d)?,
            Stmt::Block(b) => visitor.visit_block_stmt(b)?,
            Stmt::FunctionDecl(f) => visitor.visit_fundecl_stmt(f)?,
            Stmt::ParseError => todo!(),
        }
    }
}

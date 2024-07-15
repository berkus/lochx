use {
    crate::{callable::Function, error::RuntimeError, expr::Expr, scanner::Token},
    culpa::throws,
    std::rc::Rc,
};

/// Statement AST node.
#[derive(Debug, Clone)]
pub enum Stmt {
    ParseError { token: Token },
    Print(Expr),
    Return(Return),
    Expression(Expr),
    VarDecl(VarDecl),
    If(IfStmt),
    While(WhileStmt),
    Block(Vec<Stmt>),
    FunctionDecl(Function),
    Class(Class),
}

#[derive(Debug, Clone)]
pub struct Return {
    pub keyword: Token,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: Token,
    pub initializer: Expr,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Rc<Stmt>,
    pub else_branch: Option<Rc<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Rc<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Token,
    pub methods: Vec<Stmt>,       // actually, Vec<Function>...
    pub superclass: Option<Expr>, // actually, Expr::Var
}

/// Statements visitor.
pub trait Visitor {
    type ReturnType: Default;

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
    #[throws(RuntimeError)]
    fn visit_return_stmt(&mut self, stmt: &Return) -> Self::ReturnType;
    #[throws(RuntimeError)]
    fn visit_class_stmt(&mut self, stmt: &Class) -> Self::ReturnType;
}

/// Statement visitor acceptor.
pub trait Acceptor {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType;
}

impl Stmt {
    pub fn function(&self) -> &Function {
        match self {
            Stmt::FunctionDecl(f) => f,
            _ => panic!("Not a function"),
        }
    }
}

impl Acceptor for Stmt {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        match self {
            Stmt::Print(e) => visitor.visit_print_stmt(e)?,
            Stmt::Expression(e) => visitor.visit_expression_stmt(e)?,
            Stmt::If(i) => i.accept(visitor)?,
            Stmt::While(w) => w.accept(visitor)?,
            Stmt::VarDecl(d) => d.accept(visitor)?,
            Stmt::Block(b) => visitor.visit_block_stmt(b)?,
            Stmt::FunctionDecl(f) => f.accept(visitor)?,
            Stmt::Return(r) => r.accept(visitor)?,
            Stmt::Class(c) => c.accept(visitor)?,
            Stmt::ParseError { token } => {
                crate::error(
                    RuntimeError::ParseError {
                        token: token.clone(),
                        expected: crate::scanner::TokenType::Eof,
                        message: "Parse error".into(),
                    },
                    "Synchronizing parse state",
                );
                V::ReturnType::default()
            }
        }
    }
}

impl Acceptor for Return {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_return_stmt(self)?
    }
}

impl Acceptor for VarDecl {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_vardecl_stmt(self)?
    }
}

impl Acceptor for IfStmt {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_if_stmt(self)?
    }
}

impl Acceptor for WhileStmt {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_while_stmt(self)?
    }
}

impl Acceptor for Function {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_fundecl_stmt(self)?
    }
}

impl Acceptor for Class {
    #[throws(RuntimeError)]
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::ReturnType {
        visitor.visit_class_stmt(self)?
    }
}

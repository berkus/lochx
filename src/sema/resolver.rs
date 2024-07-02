// Only a few kinds of nodes are interesting when it comes to resolving variables:
//
// - A block statement introduces a new scope for the statements it contains.
// - A function declaration introduces a new scope for its body and binds its parameters in that scope.
// - A variable declaration adds a new variable to the current scope.
// - Variable and assignment expressions need to have their variables resolved.

use {
    crate::{
        callable,
        error::RuntimeError,
        expr::{self, Acceptor as _},
        runtime,
        scanner::Token,
        stmt::{self, Acceptor as _},
        Interpreter,
    },
    culpa::throws,
    std::collections::HashMap,
};

type Scope = HashMap<String, bool>;

pub struct Resolver<'interp> {
    scopes: Vec<Scope>,
    interpreter: &'interp mut Interpreter,
}

impl<'interp> Resolver<'interp> {
    pub fn new(interpreter: &'interp mut Interpreter) -> Self {
        Self {
            scopes: vec![],
            interpreter,
        }
    }

    #[throws(RuntimeError)]
    pub fn resolve(&mut self, stmts: &Vec<stmt::Stmt>) {
        self.resolve_stmts(stmts)?
    }

    #[throws(RuntimeError)]
    fn resolve_stmts(&mut self, statements: &Vec<stmt::Stmt>) {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
    }

    #[throws(RuntimeError)]
    fn resolve_stmt(&mut self, statement: &stmt::Stmt) {
        statement.accept(self)?;
    }

    #[throws(RuntimeError)]
    fn resolve_expr(&mut self, expression: &expr::Expr) {
        expression.accept(self)?;
    }

    fn resolve_local(&mut self, name: &Token) {
        for (index, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme(runtime::source())) {
                self.interpreter.resolve(name, index);
            }
        }
    }

    #[throws(RuntimeError)]
    fn resolve_function(&mut self, func: &callable::Function) {
        self.begin_scope();
        for param in &func.parameters {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(&func.body)?;
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        match self.scopes.last_mut() {
            Some(x) => {
                x.entry(name.lexeme(runtime::source()))
                    .and_modify(|v| *v = false)
                    .or_insert(false);
            }
            None => {}
        }
    }

    fn define(&mut self, name: &Token) {
        match self.scopes.last_mut() {
            Some(x) => {
                x.entry(name.lexeme(runtime::source()))
                    .and_modify(|v| *v = true)
                    .or_insert(true);
            }
            None => {}
        }
    }
}

impl expr::Visitor for Resolver<'_> {
    type ReturnType = ();

    #[throws(RuntimeError)]
    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Self::ReturnType {
        self.resolve_expr(expr.value.as_ref())?;
        self.resolve_local(&expr.name);
    }

    #[throws(RuntimeError)]
    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Self::ReturnType {
        self.resolve_expr(expr.left.as_ref())?;
        self.resolve_expr(expr.right.as_ref())?;
    }

    #[throws(RuntimeError)]
    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Self::ReturnType {
        self.resolve_expr(expr.left.as_ref())?;
        self.resolve_expr(expr.right.as_ref())?;
    }

    #[throws(RuntimeError)]
    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Self::ReturnType {
        self.resolve_expr(expr.right.as_ref())?;
    }

    #[throws(RuntimeError)]
    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Self::ReturnType {
        self.resolve_expr(expr.expr.as_ref())?;
    }

    #[throws(RuntimeError)]
    fn visit_literal_expr(&self, _expr: &expr::Literal) -> Self::ReturnType {
        // Do nothing.
    }

    #[throws(RuntimeError)]
    fn visit_var_expr(&mut self, expr: &expr::Var) -> Self::ReturnType {
        if let Some(item) = self.scopes.last() {
            if let Some(entry) = item.get(&expr.name.lexeme(runtime::source())) {
                if *entry == false {
                    crate::error(
                        RuntimeError::InvalidAssignmentTarget(expr.name.clone()),
                        "Can't read local variable in its own initializer",
                    );
                }
            }
        }

        self.resolve_local(&expr.name);
    }

    #[throws(RuntimeError)]
    fn visit_call_expr(&mut self, expr: &expr::Call) -> Self::ReturnType {
        self.resolve_expr(expr.callee.as_ref())?;
        for argument in &expr.arguments {
            self.resolve_expr(argument)?;
        }
    }
}

impl stmt::Visitor for Resolver<'_> {
    type ReturnType = ();

    #[throws(RuntimeError)]
    fn visit_print_stmt(&mut self, stmt: &expr::Expr) -> Self::ReturnType {
        self.resolve_expr(stmt)?;
    }

    #[throws(RuntimeError)]
    fn visit_expression_stmt(&mut self, stmt: &expr::Expr) -> Self::ReturnType {
        self.resolve_expr(stmt)?;
    }

    #[throws(RuntimeError)]
    fn visit_if_stmt(&mut self, stmt: &stmt::IfStmt) -> Self::ReturnType {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(stmt.then_branch.as_ref())?;
        if let Some(branch) = &stmt.else_branch {
            self.resolve_stmt(branch.as_ref())?;
        }
    }

    #[throws(RuntimeError)]
    fn visit_while_stmt(&mut self, stmt: &stmt::WhileStmt) -> Self::ReturnType {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.body)?;
    }

    #[throws(RuntimeError)]
    fn visit_vardecl_stmt(&mut self, stmt: &stmt::VarDecl) -> Self::ReturnType {
        self.declare(&stmt.name);
        self.resolve_expr(&stmt.initializer)?;
        self.define(&stmt.name);
    }

    #[throws(RuntimeError)]
    fn visit_fundecl_stmt(&mut self, stmt: &callable::Function) -> Self::ReturnType {
        self.declare(&stmt.name);
        self.define(&stmt.name);
        self.resolve_function(stmt)?;
    }

    #[throws(RuntimeError)]
    fn visit_block_stmt(&mut self, stmts: &Vec<stmt::Stmt>) -> Self::ReturnType {
        self.begin_scope();
        self.resolve_stmts(stmts)?;
        self.end_scope();
    }

    #[throws(RuntimeError)]
    fn visit_return_stmt(&mut self, stmt: &stmt::Return) -> Self::ReturnType {
        self.resolve_expr(&stmt.value)?;
    }
}
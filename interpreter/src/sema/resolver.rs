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
    culpa::{throw, throws},
    std::collections::{hash_map::Entry, HashMap},
};

type Scope = HashMap<String, bool>;

#[derive(Copy, Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Copy, Clone, PartialEq)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver<'interp> {
    scopes: Vec<Scope>,
    interpreter: &'interp mut Interpreter,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'interp> Resolver<'interp> {
    pub fn new(interpreter: &'interp mut Interpreter) -> Self {
        Self {
            scopes: vec![],
            interpreter,
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    #[throws(RuntimeError)]
    pub fn resolve(&mut self, stmts: &[stmt::Stmt]) {
        self.resolve_stmts(stmts)?
    }

    #[throws(RuntimeError)]
    fn resolve_stmts(&mut self, statements: &[stmt::Stmt]) {
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
            if scope.contains_key(name.lexeme(runtime::source())) {
                self.interpreter.resolve(name, index);
            }
        }
    }

    #[throws(RuntimeError)]
    fn resolve_function(&mut self, func: &callable::Function, ftype: FunctionType) {
        let enclosing_function = self.current_function;
        self.current_function = ftype;
        self.begin_scope();
        for param in &func.parameters {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_stmts(&func.body)?;
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    #[throws(RuntimeError)]
    fn declare(&mut self, name: &Token) {
        if let Some(x) = self.scopes.last_mut() {
            match x.entry(name.lexeme(runtime::source()).into()) {
                Entry::Occupied(_) => {
                    throw!(RuntimeError::DuplicateDeclaration(
                        name.clone(),
                        "Already a variable with this name in this scope",
                    ));
                }
                Entry::Vacant(e) => {
                    e.insert(false);
                }
            }
        }
    }

    fn define_by_name(&mut self, name: impl AsRef<str>) {
        if let Some(x) = self.scopes.last_mut() {
            x.entry(name.as_ref().into())
                .and_modify(|v| *v = true)
                .or_insert(true);
        }
    }

    fn define(&mut self, name: &Token) {
        self.define_by_name(name.lexeme(runtime::source()))
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
            if let Some(entry) = item.get(expr.name.lexeme(runtime::source())) {
                if !(*entry) {
                    throw!(RuntimeError::InvalidAssignmentTarget(
                        expr.name.clone(),
                        "Can't read local variable in its own initializer",
                    ));
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

    #[throws(RuntimeError)]
    fn visit_get_expr(&mut self, expr: &expr::Getter) -> Self::ReturnType {
        self.resolve_expr(expr.object.as_ref())?;
    }

    #[throws(RuntimeError)]
    fn visit_set_expr(&mut self, expr: &expr::Setter) -> Self::ReturnType {
        self.resolve_expr(expr.value.as_ref())?;
        self.resolve_expr(expr.object.as_ref())?;
    }

    #[throws(RuntimeError)]
    fn visit_this_expr(&mut self, expr: &expr::This) -> Self::ReturnType {
        if self.current_class == ClassType::None {
            throw!(RuntimeError::NonClassThis(
                expr.keyword.clone(),
                "Can't use `this` outside of class"
            ));
        }
        self.resolve_local(&expr.keyword);
    }

    #[throws(RuntimeError)]
    fn visit_super_expr(&mut self, expr: &expr::Super) -> Self::ReturnType {
        match self.current_class {
            ClassType::None => throw!(RuntimeError::InvalidSuper(
                expr.keyword.clone(),
                "Can't use `super` outside of a class."
            )),
            ClassType::Class => throw!(RuntimeError::InvalidSuper(
                expr.keyword.clone(),
                "Can't use `super` without a superclass."
            )),
            _ => self.resolve_local(&expr.keyword),
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
        self.declare(&stmt.name)?;
        self.resolve_expr(&stmt.initializer)?;
        self.define(&stmt.name);
    }

    #[throws(RuntimeError)]
    fn visit_fundecl_stmt(&mut self, stmt: &callable::Function) -> Self::ReturnType {
        self.declare(&stmt.name)?;
        self.define(&stmt.name);
        self.resolve_function(stmt, FunctionType::Function)?;
    }

    #[throws(RuntimeError)]
    fn visit_block_stmt(&mut self, stmts: &[stmt::Stmt]) -> Self::ReturnType {
        self.begin_scope();
        self.resolve_stmts(stmts)?;
        self.end_scope();
    }

    #[throws(RuntimeError)]
    fn visit_return_stmt(&mut self, stmt: &stmt::Return) -> Self::ReturnType {
        match self.current_function {
            FunctionType::None => {
                throw!(RuntimeError::TopLevelReturn(
                    stmt.keyword.clone(),
                    "Can't return from top-level code"
                ));
            }
            FunctionType::Initializer => {
                if stmt.value.is_some() {
                    throw!(RuntimeError::ValueReturnFromInitializer(
                        stmt.keyword.clone(),
                        "Can't return value from initializer"
                    ));
                }
            }
            _ => {}
        }
        if stmt.value.is_some() {
            self.resolve_expr(&stmt.value.clone().unwrap())?;
        }
    }

    #[throws(RuntimeError)]
    fn visit_class_stmt(&mut self, stmt: &stmt::Class) -> Self::ReturnType {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(&stmt.name)?;
        self.define(&stmt.name);

        if let Some(expr::Expr::Variable(superc)) = &stmt.superclass {
            if superc.name.lexeme(runtime::source()) == stmt.name.lexeme(runtime::source()) {
                throw!(RuntimeError::RecursiveClass(superc.name.clone()));
            }

            self.current_class = ClassType::SubClass;

            self.resolve_expr(&stmt.superclass.clone().unwrap())?;
            self.begin_scope();
            self.define_by_name("super");
        }

        self.begin_scope();
        self.define_by_name("this");

        for method in &stmt.methods {
            let fun = method.function();
            let function_type = if fun.is_init() {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(fun, function_type)?;
        }

        self.end_scope();

        if let Some(expr::Expr::Variable(_)) = &stmt.superclass {
            self.end_scope();
        }

        self.current_class = enclosing_class;
    }
}

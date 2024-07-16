use {
    crate::{
        callable::{self, Callable},
        class::{self, Class, LochxInstance},
        environment::{Environment, EnvironmentImpl, Environmental},
        error::RuntimeError,
        expr::{self, Acceptor as ExprAcceptor, Expr},
        literal::{LiteralValue, LochxCallable},
        runtime::source,
        scanner::{Token, TokenType},
        stmt::{self, Acceptor as StmtAcceptor, Stmt},
    },
    culpa::{throw, throws},
    liso::{liso, OutputOnly},
    small_map::SmallMap,
    std::rc::Rc,
};

pub struct Interpreter {
    out: OutputOnly,
    pub(super) globals: Environment,
    locals: SmallMap<16, Token, usize>,
    current_env: Environment,
}

impl Interpreter {
    pub fn new(out: OutputOnly) -> Self {
        let mut env = EnvironmentImpl::new();
        env.define(
            "clock",
            LiteralValue::Callable(LochxCallable::NativeFunction(Rc::new(
                callable::NativeFunction {
                    arity: 0,
                    body: callable::clock,
                },
            ))),
        )
        .expect("oof");
        Self {
            out,
            globals: env.clone(),
            locals: SmallMap::new(),
            current_env: env,
        }
    }

    #[throws(RuntimeError)]
    pub fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.execute(stmt)?;
        }
    }

    pub fn resolve(&mut self, token: &Token, index: usize) {
        if let Some(v) = self.locals.get_mut(token) {
            *v = index;
        } else {
            self.locals.insert(token.clone(), index);
        }
    }

    #[throws(RuntimeError)]
    fn execute(&mut self, stmt: &Stmt) {
        stmt.accept(self)?;
    }

    #[throws(RuntimeError)]
    pub(super) fn execute_block(&mut self, stmts: &[Stmt], env: Environment) {
        let previous = self.current_env.clone();
        self.current_env = env;
        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                self.current_env = previous;
                throw!(e);
            }
        }
        self.current_env = previous;
    }

    #[throws(RuntimeError)]
    fn evaluate(&mut self, expr: &Expr) -> LiteralValue {
        expr.accept(self)?
    }

    #[throws(RuntimeError)]
    fn look_up_variable(&mut self, token: &Token) -> LiteralValue {
        let distance = self.locals.get(token);
        if let Some(distance) = distance {
            self.current_env.get_at(*distance, token.clone())?
        } else {
            self.globals.get(token.clone())?
        }
    }
}

impl stmt::Visitor for Interpreter {
    type ReturnType = ();

    #[throws(RuntimeError)]
    fn visit_print_stmt(&mut self, stmt: &Expr) -> Self::ReturnType {
        let expr = self.evaluate(stmt)?;
        self.out
            .wrapln(liso!(fg = magenta, format!("{}", expr), reset));
    }

    #[throws(RuntimeError)]
    fn visit_expression_stmt(&mut self, stmt: &Expr) -> Self::ReturnType {
        self.evaluate(stmt)?;
    }

    #[throws(RuntimeError)]
    fn visit_vardecl_stmt(&mut self, stmt: &stmt::VarDecl) -> Self::ReturnType {
        let value = self.evaluate(&stmt.initializer)?;
        self.current_env.define(stmt.name.lexeme(source()), value)?;
    }

    #[throws(RuntimeError)]
    fn visit_block_stmt(&mut self, stmts: &[Stmt]) -> Self::ReturnType {
        self.execute_block(stmts, EnvironmentImpl::nested(self.current_env.clone()))?;
    }

    #[throws(RuntimeError)]
    fn visit_if_stmt(&mut self, stmt: &stmt::IfStmt) -> Self::ReturnType {
        let expr = self.evaluate(&stmt.condition)?;
        if expr.is_truthy() {
            self.execute(stmt.then_branch.as_ref())?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }
    }

    #[throws(RuntimeError)]
    fn visit_while_stmt(&mut self, stmt: &stmt::WhileStmt) -> Self::ReturnType {
        while self.evaluate(&stmt.condition)?.is_truthy() {
            self.execute(stmt.body.as_ref())?;
        }
    }

    #[throws(RuntimeError)]
    fn visit_fundecl_stmt(&mut self, stmt: &callable::Function) -> Self::ReturnType {
        let fun = callable::Function {
            name: stmt.name.clone(),
            parameters: stmt.parameters.clone(),
            body: stmt.body.clone(),
            closure: EnvironmentImpl::nested(self.current_env.clone()),
            is_initializer: false,
        };
        self.current_env
            .define(stmt.name.lexeme(source()), fun.into())?;
    }

    #[throws(RuntimeError)]
    fn visit_return_stmt(&mut self, stmt: &stmt::Return) -> Self::ReturnType {
        throw!(RuntimeError::ReturnValue(if stmt.value.is_some() {
            self.evaluate(&stmt.value.clone().unwrap())?
        } else {
            LiteralValue::Nil
        }))
    }

    #[throws(RuntimeError)]
    fn visit_class_stmt(&mut self, stmt: &stmt::Class) -> Self::ReturnType {
        let superclass = if let Some(superc) = &stmt.superclass {
            let superclass = self.evaluate(superc)?;
            if let LiteralValue::Callable(LochxCallable::Class(baseclass)) = superclass {
                Some(baseclass)
            } else {
                if let Expr::Variable(name) = superc {
                    throw!(RuntimeError::NotAClassBase(name.name.clone()));
                }
                throw!(RuntimeError::GenericError)
            }
        } else {
            None
        };

        self.current_env
            .define(stmt.name.lexeme(source()), LiteralValue::Nil)?;
        let previous = if superclass.is_some() {
            let previous = self.current_env.clone();
            self.current_env = EnvironmentImpl::nested(self.current_env.clone());
            self.current_env
                .define("super", superclass.clone().unwrap().into())?;
            previous
        } else {
            self.current_env.clone()
        };

        let mut methods =
            SmallMap::<16, String, callable::Function>::with_capacity(stmt.methods.len());
        for m in stmt.methods.iter().map(|m| m.function()) {
            let fun = callable::Function {
                closure: self.current_env.clone(),
                is_initializer: m.is_init(),
                ..m.clone()
            };
            methods.insert(m.name.lexeme(source()).into(), fun);
        }
        let class = class::Class::new(stmt.name.lexeme(source()).into(), superclass, methods);
        self.current_env = previous;
        self.current_env.assign(stmt.name.clone(), class.into())?;
    }
}

impl expr::Visitor for Interpreter {
    type ReturnType = LiteralValue;

    #[throws(RuntimeError)]
    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Self::ReturnType {
        let left = self.evaluate(expr.left.as_ref())?;
        let right = self.evaluate(expr.right.as_ref())?;

        match expr.op.r#type {
            TokenType::Plus => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l + r),
                (LiteralValue::Str(l), LiteralValue::Str(r)) => LiteralValue::Str(l + &r),
                (LiteralValue::Num(l), LiteralValue::Str(r)) => {
                    LiteralValue::Str(format!("{}{}", l, r))
                }
                (LiteralValue::Str(l), LiteralValue::Num(r)) => {
                    LiteralValue::Str(format!("{}{}", l, r))
                }
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::Minus => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l - r),
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::Star => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l * r),
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::Slash => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Num(l / r),
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::Greater => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l > r),
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::GreaterEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l >= r),
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::Less => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l < r),
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::LessEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l <= r),
                _ => invalid_binop_arguments(expr.op.clone()),
            },
            TokenType::BangEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l != r),
                (LiteralValue::Str(l), LiteralValue::Str(r)) => LiteralValue::Bool(l != r),
                _ => LiteralValue::Bool(true),
            },
            TokenType::EqualEqual => match (left, right) {
                (LiteralValue::Num(l), LiteralValue::Num(r)) => LiteralValue::Bool(l == r),
                (LiteralValue::Str(l), LiteralValue::Str(r)) => LiteralValue::Bool(l == r),
                _ => LiteralValue::Bool(false),
            },
            _ => invalid_binop_arguments(expr.op.clone()),
        }
    }

    #[throws(RuntimeError)]
    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Self::ReturnType {
        let right = self.evaluate(expr.right.as_ref())?;
        match expr.op.r#type {
            TokenType::Minus => match right {
                LiteralValue::Num(n) => LiteralValue::Num(-n),
                _ => invalid_unop_arguments(expr.op.clone()),
            },
            TokenType::Bang => LiteralValue::Bool(!right.is_truthy()),
            _ => unreachable!(),
        }
    }

    #[throws(RuntimeError)]
    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Self::ReturnType {
        self.evaluate(expr.expr.as_ref())?
    }

    #[throws(RuntimeError)]
    fn visit_literal_expr(&self, expr: &expr::Literal) -> Self::ReturnType {
        expr.value.clone()
    }

    #[throws(RuntimeError)]
    fn visit_var_expr(&mut self, expr: &expr::Var) -> Self::ReturnType {
        self.look_up_variable(&expr.name)?
    }

    #[throws(RuntimeError)]
    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Self::ReturnType {
        let value = self.evaluate(expr.value.as_ref())?;
        let distance = self.locals.get(&expr.name);
        if let Some(d) = distance {
            self.current_env
                .assign_at(*d, expr.name.clone(), value.clone())?;
        } else {
            self.globals.assign(expr.name.clone(), value.clone())?;
        }
        value
    }

    #[throws(RuntimeError)]
    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Self::ReturnType {
        let left = self.evaluate(expr.left.as_ref())?;

        if expr.op.r#type == TokenType::KwOr {
            if left.is_truthy() {
                return left;
            }
        } else if !left.is_truthy() {
            return left;
        }

        self.evaluate(expr.right.as_ref())?
    }

    #[throws(RuntimeError)]
    fn visit_call_expr(&mut self, expr: &expr::Call) -> Self::ReturnType {
        let callee = self.evaluate(expr.callee.as_ref())?;

        match callee {
            LiteralValue::Callable(callable) => {
                let callable = match callable {
                    LochxCallable::Function(f) => f as Rc<dyn Callable>,
                    LochxCallable::NativeFunction(f) => f as Rc<dyn Callable>,
                    LochxCallable::Class(c) => c as Rc<dyn Callable>,
                };

                if expr.arguments.len() != callable.arity() {
                    throw!(RuntimeError::InvalidArity(
                        expr.paren.clone(),
                        callable.arity(),
                        expr.arguments.len()
                    ))
                }

                let mut arguments = Vec::with_capacity(expr.arguments.len());
                for arg in expr.arguments.iter() {
                    arguments.push(self.evaluate(arg)?);
                }
                return callable.call(self, &arguments)?;
            }
            _ => throw!(RuntimeError::NotACallable(expr.paren.clone())),
        };
    }

    #[throws(RuntimeError)]
    fn visit_get_expr(&mut self, expr: &expr::Getter) -> Self::ReturnType {
        let object = self.evaluate(expr.object.as_ref())?;
        match object {
            LiteralValue::Instance(i) => i.read().unwrap().get(expr.name.clone())?,
            _ => throw!(RuntimeError::InvalidPropertyAccess(
                expr.name.clone(),
                "Only instances have properties."
            )),
        }
    }

    #[throws(RuntimeError)]
    fn visit_set_expr(&mut self, expr: &expr::Setter) -> Self::ReturnType {
        let mut object = self.evaluate(expr.object.as_ref())?;
        match &mut object {
            LiteralValue::Instance(i) => {
                let value = self.evaluate(expr.value.as_ref())?;
                i.write().unwrap().set(expr.name.clone(), value.clone());
                return value;
            }
            _ => throw!(RuntimeError::InvalidPropertyAccess(
                expr.name.clone(),
                "Only instances have fields"
            )),
        }
    }

    #[throws(RuntimeError)]
    fn visit_this_expr(&mut self, expr: &expr::This) -> Self::ReturnType {
        self.look_up_variable(&expr.keyword)?
    }

    #[throws(RuntimeError)]
    fn visit_super_expr(&mut self, expr: &expr::Super) -> Self::ReturnType {
        let distance = self.locals.get(&expr.keyword);
        if let Some(distance) = distance {
            let superclass: Rc<Class> = self
                .current_env
                .get_at_by_name(*distance, "super")?
                .try_into()?;
            let object: LochxInstance = self
                .current_env
                .get_at_by_name(distance - 1, "this")?
                .try_into()?;
            let method = superclass.find_method(expr.method.clone())?;
            method.bind(&object)?.into()
        } else {
            throw!(RuntimeError::GenericError)
        }
    }
}

fn invalid_binop_arguments(op: Token) -> LiteralValue {
    crate::error(
        RuntimeError::ParseError {
            token: op.clone(),
            expected: TokenType::Eof,
            message: "Unexpected arguments".into(),
        },
        "Invalid arguments to binary expression",
    );
    LiteralValue::Nil
}

fn invalid_unop_arguments(op: Token) -> LiteralValue {
    crate::error(
        RuntimeError::ParseError {
            token: op.clone(),
            expected: TokenType::Eof,
            message: "Unexpected arguments".into(),
        },
        "Invalid arguments to unary expression",
    );
    LiteralValue::Nil
}

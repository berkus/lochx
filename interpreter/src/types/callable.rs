use {
    crate::{
        class::LochxInstance,
        environment::{Environment, EnvironmentImpl, Environmental},
        error::RuntimeError,
        interpreter::Interpreter,
        literal::LiteralValue,
        runtime::source,
        scanner::Token,
        stmt::Stmt,
    },
    culpa::{throw, throws},
    std::{fmt::Display, time::SystemTime},
};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Stmt>,
    pub closure: Environment,
    pub is_initializer: bool,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<fn {}({})>",
            self.name,
            self.parameters
                .iter()
                .map(|p| p.lexeme(source()).into())
                .collect::<Vec<String>>()
                .join(",")
        )
    }
}

impl Function {
    #[throws(RuntimeError)]
    pub fn bind(&self, instance: &LochxInstance) -> Self {
        let mut closure = EnvironmentImpl::nested(self.closure.clone());
        closure.define("this", LiteralValue::Instance(instance.clone()))?;
        Self {
            closure,
            ..self.clone()
        }
    }

    pub fn is_init(&self) -> bool {
        self.name.lexeme(source()) == "init"
    }
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub body: fn(&mut Interpreter, &[LiteralValue]) -> Result<LiteralValue, RuntimeError>,
}

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[LiteralValue],
    ) -> Result<LiteralValue, RuntimeError>;
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    #[throws(RuntimeError)]
    fn call(&self, interpreter: &mut Interpreter, arguments: &[LiteralValue]) -> LiteralValue {
        let mut environment = EnvironmentImpl::nested(self.closure.clone());
        for (param, arg) in self.parameters.iter().zip(arguments.iter()) {
            environment.define(param.lexeme(source()), arg.clone())?;
        }
        let ret = interpreter.execute_block(&self.body, environment);
        if let Err(e) = ret {
            match e {
                RuntimeError::ReturnValue(v) => {
                    if self.is_initializer {
                        return self.closure.get_at_by_name(0, "this")?;
                    }
                    return v;
                }
                _ => throw!(e),
            }
        }
        if self.is_initializer {
            return self.closure.get_at_by_name(0, "this")?;
        }
        LiteralValue::Nil
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    #[throws(RuntimeError)]
    fn call(&self, interpreter: &mut Interpreter, arguments: &[LiteralValue]) -> LiteralValue {
        (self.body)(interpreter, arguments)?
    }
}

// Native functions

#[throws(RuntimeError)]
pub fn clock(_no_interp: &mut Interpreter, _no_args: &[LiteralValue]) -> LiteralValue {
    LiteralValue::Num(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| RuntimeError::ClockBackwards)?
            .as_secs_f64(),
    )
}

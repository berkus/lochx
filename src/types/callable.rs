use {
    crate::{
        class::LochxInstance,
        environment::{Environment, EnvironmentImpl},
        error::RuntimeError,
        interpreter::Interpreter,
        literal::LiteralValue,
        runtime::source,
        scanner::Token,
        stmt::Stmt,
    },
    anyhow::anyhow,
    culpa::{throw, throws},
    std::{fmt::Display, time::SystemTime},
};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Stmt>,
    pub closure: Environment,
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
    pub fn bind(&self, instance: &LochxInstance) -> Self {
        let closure = EnvironmentImpl::nested(self.closure.clone());
        closure
            .write()
            .expect("write lock in bind")
            .define("this", LiteralValue::Instance(instance.clone()));
        Self {
            closure,
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub body: fn(&mut Interpreter, Vec<LiteralValue>) -> Result<LiteralValue, RuntimeError>,
}

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LiteralValue>,
    ) -> Result<LiteralValue, RuntimeError>;
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    #[throws(RuntimeError)]
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<LiteralValue>) -> LiteralValue {
        let environment = EnvironmentImpl::nested(self.closure.clone());
        for (param, arg) in self.parameters.iter().zip(arguments.iter()) {
            environment
                .write()
                .map_err(|_| RuntimeError::EnvironmentError(anyhow!("write lock in call")))? // @todo miette!
                .define(param.lexeme(source()), arg.clone());
        }
        let ret = interpreter.execute_block(self.body.clone(), environment);
        if let Err(e) = ret {
            match e {
                RuntimeError::ReturnValue(v) => return v,
                _ => throw!(e),
            }
        }
        LiteralValue::Nil
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    #[throws(RuntimeError)]
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<LiteralValue>) -> LiteralValue {
        (self.body)(interpreter, arguments)?
    }
}

// Native functions

#[throws(RuntimeError)]
pub fn clock(_no_interp: &mut Interpreter, _no_args: Vec<LiteralValue>) -> LiteralValue {
    LiteralValue::Num(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| RuntimeError::ClockBackwards)?
            .as_secs_f64(),
    )
}

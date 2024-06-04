use {
    crate::{
        environment::Environment, error::RuntimeError, interpreter::Interpreter,
        literal::LiteralValue, scanner::Token, stmt::Stmt,
    },
    culpa::{throw, throws},
    std::time::SystemTime,
};

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub parameters: Vec<Token>,
    pub body: Vec<Stmt>,
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
        let environment = Environment::nested(interpreter.globals.clone());
        for (param, arg) in self.parameters.iter().zip(arguments.iter()) {
            environment
                .borrow_mut()
                .define(param.lexeme().clone(), arg.clone());
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

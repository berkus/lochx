use {
    crate::{
        callable::{Function, NativeFunction},
        class::{Class, LochxInstance},
        error::RuntimeError,
    },
    culpa::throw,
};

#[derive(Debug, Clone, Default)]
pub enum LiteralValue {
    Str(String),
    Num(f64),
    #[default]
    Nil,
    Bool(bool),
    Callable(LochxCallable), // Function or NativeFunction call
    Instance(LochxInstance),
}

#[derive(Debug, Clone)]
pub enum LochxCallable {
    Function(Box<Function>),
    NativeFunction(Box<NativeFunction>),
    Class(Box<Class>),
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LiteralValue::Str(s) => s.clone(),
                LiteralValue::Num(n) => n.to_string().trim_end_matches(".0").to_string(),
                LiteralValue::Nil => "nil".to_string(),
                LiteralValue::Bool(b) => b.to_string(),
                LiteralValue::Callable(c) => match c {
                    LochxCallable::Function(f) => format!("<fun {}>", f.name),
                    LochxCallable::NativeFunction(_) => format!("<native fun>"),
                    LochxCallable::Class(c) => format!("<class {}>", c.name),
                },
                LiteralValue::Instance(i) => format!("<{} instance>", i.read().unwrap().class.name),
            }
        )
    }
}

impl LiteralValue {
    /// nil and false are falsy, everything else is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Nil => false,
            LiteralValue::Bool(b) => *b,
            _ => true,
        }
    }
}

impl From<Class> for LiteralValue {
    fn from(value: Class) -> Self {
        Self::Callable(LochxCallable::Class(Box::new(value)))
    }
}

impl From<Box<Class>> for LiteralValue {
    fn from(value: Box<Class>) -> Self {
        Self::Callable(LochxCallable::Class(value))
    }
}

impl From<Function> for LiteralValue {
    fn from(value: Function) -> Self {
        Self::Callable(LochxCallable::Function(Box::new(value)))
    }
}

impl From<Box<Function>> for LiteralValue {
    fn from(value: Box<Function>) -> Self {
        Self::Callable(LochxCallable::Function(value))
    }
}

impl TryFrom<LiteralValue> for Class {
    type Error = RuntimeError;

    fn try_from(value: LiteralValue) -> Result<Self, Self::Error> {
        Ok(
            match match value {
                LiteralValue::Callable(c) => c,
                _ => throw!(RuntimeError::GenericError),
            } {
                LochxCallable::Class(f) => *f,
                _ => throw!(RuntimeError::GenericError),
            },
        )
    }
}

impl TryFrom<LiteralValue> for Function {
    type Error = RuntimeError;

    fn try_from(value: LiteralValue) -> Result<Self, Self::Error> {
        Ok(
            match match value {
                LiteralValue::Callable(c) => c,
                _ => throw!(RuntimeError::GenericError),
            } {
                LochxCallable::Function(f) => *f,
                _ => throw!(RuntimeError::GenericError),
            },
        )
    }
}

impl TryFrom<LiteralValue> for LochxInstance {
    type Error = RuntimeError;

    fn try_from(value: LiteralValue) -> Result<Self, Self::Error> {
        Ok(match value {
            LiteralValue::Instance(i) => i,
            _ => throw!(RuntimeError::GenericError),
        })
    }
}

use crate::{
    callable::{Function, NativeFunction},
    class::{Class, LochxInstance},
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

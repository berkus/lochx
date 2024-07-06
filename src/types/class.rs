use {
    crate::{
        callable::Callable, error::RuntimeError, interpreter::Interpreter, literal::LiteralValue,
        runtime, scanner::Token,
    },
    culpa::throws,
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
};

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

// Emulate pointers to instances, as they exist by-reference.
pub type LochxInstance = Arc<RwLock<LochxInstanceImpl>>;

#[derive(Debug, Clone)]
pub struct LochxInstanceImpl {
    pub class: Class,
    fields: HashMap<String, LiteralValue>,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    #[throws(RuntimeError)]
    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<LiteralValue>) -> LiteralValue {
        LiteralValue::Instance(Arc::new(RwLock::new(LochxInstanceImpl::new(self.clone()))))
    }
}

impl LochxInstanceImpl {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    #[throws(RuntimeError)]
    pub fn get(&self, name: Token) -> LiteralValue {
        let key = name.lexeme(runtime::source());
        self.fields
            .get(&key)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedProperty(name))?
    }

    pub fn set(&mut self, name: Token, value: LiteralValue) {
        let key = name.lexeme(runtime::source());
        *self.fields.entry(key.clone()).or_default() = value;
    }
}

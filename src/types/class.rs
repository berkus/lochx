use {
    crate::{
        callable::{Callable, Function},
        error::RuntimeError,
        interpreter::Interpreter,
        literal::LiteralValue,
        runtime,
        scanner::Token,
    },
    culpa::throws,
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
};

/// Class holds methods.
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    methods: HashMap<String, Function>,
}

// Emulate pointers to instances, as they exist by-reference.
pub type LochxInstance = Arc<RwLock<LochxInstanceImpl>>;

/// Instance holds fields.
#[derive(Debug, Clone)]
pub struct LochxInstanceImpl {
    pub class: Class,
    fields: HashMap<String, LiteralValue>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>) -> Self {
        Self { name, methods }
    }

    pub fn find_method_by_name(&self, method_name: impl AsRef<str>) -> Option<Function> {
        self.methods.get(method_name.as_ref()).cloned()
    }

    #[throws(RuntimeError)]
    pub fn find_method(&self, method_name: Token) -> Function {
        self.find_method_by_name(method_name.lexeme(runtime::source()))
            .ok_or_else(|| RuntimeError::UndefinedProperty(method_name))?
    }
}

impl std::fmt::Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        let init = self.find_method_by_name("init");
        if init.is_some() {
            init.unwrap().arity()
        } else {
            0
        }
    }

    #[throws(RuntimeError)]
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<LiteralValue>) -> LiteralValue {
        let instance = LochxInstanceImpl::new(self.clone()).wrapped();
        let init = self.find_method_by_name("init");
        if init.is_some() {
            init.unwrap().bind(&instance).call(interpreter, arguments)?;
        }
        LiteralValue::Instance(instance)
    }
}

impl LochxInstanceImpl {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    fn wrapped(&self) -> LochxInstance {
        Arc::new(RwLock::new(self.clone()))
    }

    #[throws(RuntimeError)]
    pub fn get(&self, name: Token) -> LiteralValue {
        let key = name.lexeme(runtime::source());
        self.fields.get(&key).cloned().map_or_else(
            || {
                self.class.find_method(name.clone()).map(|f| {
                    LiteralValue::Callable(crate::literal::LochxCallable::Function(Box::new(
                        f.bind(&self.wrapped()),
                    )))
                })
            },
            Ok,
        )?
    }

    pub fn set(&mut self, name: Token, value: LiteralValue) {
        let key = name.lexeme(runtime::source());
        *self.fields.entry(key.clone()).or_default() = value;
    }
}
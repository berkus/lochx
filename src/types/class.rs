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
    small_map::SmallMap,
    std::{rc::Rc, sync::RwLock},
};

/// Class holds methods.
#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    superclass: Option<Rc<Class>>,
    methods: SmallMap<16, String, Function>,
}

#[allow(unused)]
struct MethodsDisplayWrap(SmallMap<16, String, Function>);

impl std::fmt::Display for MethodsDisplayWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, method) in self.0.iter() {
            writeln!(f, "{}->{}", name, method)?;
        }
        Ok(())
    }
}

// Emulate pointers to instances, as they exist by-reference.
pub type LochxInstance = Rc<RwLock<LochxInstanceImpl>>;

/// Instance holds fields.
#[derive(Debug, Clone)]
pub struct LochxInstanceImpl {
    pub class: Class,
    fields: SmallMap<16, String, LiteralValue>,
}

impl Class {
    pub fn new(
        name: String,
        superclass: Option<Rc<Class>>,
        methods: SmallMap<16, String, Function>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method_by_name(&self, method_name: impl AsRef<str>) -> Option<Function> {
        if let Some(m) = self.methods.get(method_name.as_ref()) {
            return Some(m.clone());
        }
        self.superclass
            .clone()
            .and_then(|sc| sc.find_method_by_name(method_name))
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
        self.find_method_by_name("init")
            .map(|init| init.arity())
            .unwrap_or(0)
    }

    #[throws(RuntimeError)]
    fn call(&self, interpreter: &mut Interpreter, arguments: &[LiteralValue]) -> LiteralValue {
        let instance = LochxInstanceImpl::new(self.clone()).wrapped();
        self.find_method_by_name("init").map_or_else(
            || Ok(LiteralValue::Nil),
            |init| init.bind(&instance)?.call(interpreter, arguments),
        )?;
        LiteralValue::Instance(instance)
    }
}

impl LochxInstanceImpl {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: SmallMap::new(),
        }
    }

    fn wrapped(&self) -> LochxInstance {
        Rc::new(RwLock::new(self.clone()))
    }

    #[throws(RuntimeError)]
    pub fn get(&self, name: Token) -> LiteralValue {
        let key = name.lexeme(runtime::source());
        if let Some(v) = self.fields.get(key) {
            return v.clone();
        } else {
            let f = self.class.find_method(name.clone())?;
            return f.bind(&self.wrapped())?.into();
        }
    }

    pub fn set(&mut self, name: Token, value: LiteralValue) {
        let key = name.lexeme(runtime::source());
        if let Some(v) = self.fields.get_mut(key) {
            *v = value;
        } else {
            self.fields.insert(key.into(), value);
        }
    }
}

use {
    crate::{
        error::RuntimeError,
        scanner::{LiteralValue, Token},
    },
    core::cell::RefCell,
    culpa::{throw, throws},
    std::{collections::HashMap, rc::Rc},
};

pub struct Environment {
    values: HashMap<String, LiteralValue>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn nested(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(parent.clone()),
        }))
    }
}

impl Environment {
    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    #[throws(RuntimeError)]
    pub fn get(&self, name: Token) -> LiteralValue {
        if self.values.contains_key(&name.lexeme()) {
            return self.values.get(&name.lexeme()).unwrap().clone();
        }
        if let Some(parent) = &self.enclosing {
            return parent.borrow().get(name)?;
        }
        throw!(RuntimeError::UndefinedVariable(name.lexeme()))
    }

    #[throws(RuntimeError)]
    pub fn assign(&mut self, name: Token, value: LiteralValue) {
        if self.values.contains_key(&name.lexeme()) {
            self.values.entry(name.lexeme()).and_modify(|e| *e = value);
            return;
        }
        if let Some(parent) = &self.enclosing {
            parent.borrow_mut().assign(name, value)?;
            return;
        }
        throw!(RuntimeError::UndefinedVariable(name.lexeme()))
    }
}

use {
    crate::{error::RuntimeError, literal::LiteralValue, scanner::Token},
    anyhow::anyhow,
    culpa::{throw, throws},
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
};

pub type Environment = Arc<RwLock<EnvironmentImpl>>;

#[derive(Debug)]
pub struct EnvironmentImpl {
    values: HashMap<String, LiteralValue>,
    enclosing: Option<Environment>,
}

impl EnvironmentImpl {
    pub fn new() -> Environment {
        Arc::new(RwLock::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn nested(parent: Environment) -> Environment {
        Arc::new(RwLock::new(Self {
            values: HashMap::new(),
            enclosing: Some(parent.clone()),
        }))
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    #[throws(RuntimeError)]
    pub fn get(&self, name: Token) -> LiteralValue {
        if self.values.contains_key(&name.lexeme()) {
            return self.values.get(&name.lexeme()).unwrap().clone();
        }
        if let Some(parent) = &self.enclosing {
            return parent
                .read()
                .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get")))?
                .get(name)?;
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
            parent
                .write()
                .map_err(|_| RuntimeError::EnvironmentError(anyhow!("write lock in assign")))?
                .assign(name, value)?;
            return;
        }
        throw!(RuntimeError::UndefinedVariable(name.lexeme()))
    }
}

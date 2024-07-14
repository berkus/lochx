use {
    crate::{error::RuntimeError, literal::LiteralValue, runtime::source, scanner::Token},
    anyhow::anyhow,
    culpa::{throw, throws},
    std::{
        collections::HashMap,
        sync::{Arc, RwLock},
    },
};

pub type Environment = Arc<RwLock<EnvironmentImpl>>;

pub trait Environmental {
    #[throws(RuntimeError)]
    fn define(&mut self, name: impl AsRef<str>, value: LiteralValue);
    #[throws(RuntimeError)]
    fn get(&self, name: Token) -> LiteralValue;
    #[throws(RuntimeError)]
    fn get_by_name(&self, name: impl AsRef<str>) -> LiteralValue;
    #[throws(RuntimeError)]
    fn get_at(&self, distance: usize, name: Token) -> LiteralValue;
    #[throws(RuntimeError)]
    fn get_at_by_name(&self, distance: usize, name: impl AsRef<str>) -> LiteralValue;
    #[throws(RuntimeError)]
    fn assign(&mut self, name: Token, value: LiteralValue);
    #[throws(RuntimeError)]
    fn assign_at(&mut self, distance: usize, name: Token, value: LiteralValue);
}

impl Environmental for Environment {
    #[throws(RuntimeError)]
    fn define(&mut self, name: impl AsRef<str>, value: LiteralValue) {
        self.write()
            .map_err(|_| {
                RuntimeError::EnvironmentError(anyhow!("write lock in define"))
                // @todo miette!
            })?
            .define(name, value)?;
    }

    #[throws(RuntimeError)]
    fn get(&self, name: Token) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get")))? // @todo miette!
            .get(name)?
    }

    #[throws(RuntimeError)]
    fn get_by_name(&self, name: impl AsRef<str>) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get_by_name")))? // @todo miette!
            .get_by_name(name)?
    }

    #[throws(RuntimeError)]
    fn get_at(&self, distance: usize, name: Token) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get_at")))? // @todo miette!
            .get_at(distance, name)?
    }

    #[throws(RuntimeError)]
    fn get_at_by_name(&self, distance: usize, name: impl AsRef<str>) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get_at_by_name")))? // @todo miette!
            .get_at_by_name(distance, name)?
    }

    #[throws(RuntimeError)]
    fn assign(&mut self, name: Token, value: LiteralValue) {
        self.write()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("write lock in assign")))? // @todo miette!
            .assign(name, value)?
    }

    #[throws(RuntimeError)]
    fn assign_at(&mut self, distance: usize, name: Token, value: LiteralValue) {
        self.write()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("write lock in assign_at")))? // @todo miette!
            .assign_at(distance, name, value)?
    }
}

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

    #[throws(RuntimeError)]
    fn ancestor(&self, distance: usize) -> Environment {
        let mut parent = self.enclosing.clone();
        for _ in distance..1 {
            if let Some(p) = parent {
                let p = p.read().map_err(|_| {
                    RuntimeError::EnvironmentError(anyhow!("read lock in ancestor"))
                })?; // @todo miette!
                parent = p.enclosing.clone();
            }
        }
        if parent.is_none() {
            panic!("Environment stacks misaligned");
        }
        parent.unwrap()
    }
}

impl Environmental for EnvironmentImpl {
    #[throws(RuntimeError)]
    fn define(&mut self, name: impl AsRef<str>, value: LiteralValue) {
        self.values.insert(name.as_ref().into(), value);
    }

    #[throws(RuntimeError)]
    fn get(&self, name: Token) -> LiteralValue {
        if self.values.contains_key(name.lexeme(source())) {
            return self.values.get(name.lexeme(source())).unwrap().clone();
        }
        // @todo Use ancestor(distance=1):
        if let Some(parent) = &self.enclosing {
            return parent
                .read()
                .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get")))? // @todo miette!
                .get(name)?;
        }
        throw!(RuntimeError::UndefinedVariable(
            name.clone(),
            name.to_string().into()
        ))
    }

    #[throws(RuntimeError)]
    fn get_by_name(&self, name: impl AsRef<str>) -> LiteralValue {
        if self.values.contains_key(name.as_ref()) {
            return self.values.get(name.as_ref()).unwrap().clone();
        }
        // @todo Use ancestor(distance=1):
        if let Some(parent) = &self.enclosing {
            return parent
                .read()
                .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get")))? // @todo miette!
                .get_by_name(name)?;
        }
        throw!(RuntimeError::UndefinedVariableName(name.as_ref().into(),))
    }

    #[throws(RuntimeError)]
    fn get_at(&self, distance: usize, name: Token) -> LiteralValue {
        if distance == 0 {
            return self.get(name)?;
        }
        self.ancestor(distance)?
            .read()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get_at")))? // @todo miette!
            .get(name)?
    }

    #[throws(RuntimeError)]
    fn get_at_by_name(&self, distance: usize, name: impl AsRef<str>) -> LiteralValue {
        if distance == 0 {
            return self.get_by_name(name)?;
        }
        self.ancestor(distance)?
            .read()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get_at")))? // @todo miette!
            .get_by_name(name)?
    }

    #[throws(RuntimeError)]
    fn assign(&mut self, name: Token, value: LiteralValue) {
        if self.values.contains_key(&name.to_string()) {
            self.values
                .entry(name.to_string().into())
                .and_modify(|e| *e = value);
            return;
        }
        // @todo Use ancestor(distance=1):
        if let Some(parent) = &self.enclosing {
            parent
                .write()
                .map_err(|_| RuntimeError::EnvironmentError(anyhow!("write lock in assign")))? // @todo miette!
                .assign(name, value)?;
            return;
        }
        throw!(RuntimeError::UndefinedVariable(
            name.clone(),
            name.to_string()
        ))
    }

    #[throws(RuntimeError)]
    fn assign_at(&mut self, distance: usize, name: Token, value: LiteralValue) {
        if distance == 0 {
            return self.assign(name, value)?;
        }
        self.ancestor(distance)?
            .write()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("write lock in assign_at")))? // @todo miette!
            .assign(name, value)?;
    }
}

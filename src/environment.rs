use {
    crate::{error::RuntimeError, literal::LiteralValue, scanner::SourceToken},
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
    pub fn get(&self, name: SourceToken) -> LiteralValue {
        if self.values.contains_key(name.to_str()) {
            return self.values.get(name.to_str()).unwrap().clone();
        }
        // @todo Use ancestor(distance=1):
        if let Some(parent) = &self.enclosing {
            return parent
                .read()
                .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get")))? // @todo miette!
                .get(name)?;
        }
        throw!(RuntimeError::UndefinedVariable(
            name.token.clone(),
            name.to_str().into()
        ))
    }

    #[throws(RuntimeError)]
    pub fn get_at(&self, distance: usize, name: SourceToken) -> LiteralValue {
        if distance == 0 {
            return self.get(name)?;
        }
        self.ancestor(distance)?
            .read()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("read lock in get_at")))? // @todo miette!
            .get(name)?
    }

    #[throws(RuntimeError)]
    pub fn assign(&mut self, name: SourceToken, value: LiteralValue) {
        if self.values.contains_key(name.to_str()) {
            self.values
                .entry(name.to_str().into())
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
            name.token.clone(),
            name.to_str().into()
        ))
    }

    #[throws(RuntimeError)]
    pub fn assign_at(&mut self, distance: usize, name: SourceToken, value: LiteralValue) {
        if distance == 0 {
            return self.assign(name, value)?;
        }
        self.ancestor(distance)?
            .write()
            .map_err(|_| RuntimeError::EnvironmentError(anyhow!("write lock in assign_at")))? // @todo miette!
            .assign(name, value)?;
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

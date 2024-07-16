use {
    crate::{error::RuntimeError, literal::LiteralValue, runtime::source, scanner::Token},
    culpa::{throw, throws},
    small_map::SmallMap,
    std::{rc::Rc, sync::RwLock},
};

pub type Environment = Rc<RwLock<EnvironmentImpl>>;

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
                RuntimeError::EnvironmentError("write lock in define")
                // @todo miette!
            })?
            .define(name, value)?;
    }

    #[throws(RuntimeError)]
    fn get(&self, name: Token) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError("read lock in get"))? // @todo miette!
            .get(name)?
    }

    #[throws(RuntimeError)]
    fn get_by_name(&self, name: impl AsRef<str>) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError("read lock in get_by_name"))? // @todo miette!
            .get_by_name(name)?
    }

    #[throws(RuntimeError)]
    fn get_at(&self, distance: usize, name: Token) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError("read lock in get_at"))? // @todo miette!
            .get_at(distance, name)?
    }

    #[throws(RuntimeError)]
    fn get_at_by_name(&self, distance: usize, name: impl AsRef<str>) -> LiteralValue {
        self.read()
            .map_err(|_| RuntimeError::EnvironmentError("read lock in get_at_by_name"))? // @todo miette!
            .get_at_by_name(distance, name)?
    }

    #[throws(RuntimeError)]
    fn assign(&mut self, name: Token, value: LiteralValue) {
        self.write()
            .map_err(|_| RuntimeError::EnvironmentError("write lock in assign"))? // @todo miette!
            .assign(name, value)?
    }

    #[throws(RuntimeError)]
    fn assign_at(&mut self, distance: usize, name: Token, value: LiteralValue) {
        self.write()
            .map_err(|_| RuntimeError::EnvironmentError("write lock in assign_at"))? // @todo miette!
            .assign_at(distance, name, value)?
    }
}

#[derive(Debug)]
pub struct EnvironmentImpl {
    values: SmallMap<32, String, LiteralValue>,
    enclosing: Option<Environment>,
}

impl EnvironmentImpl {
    pub fn new() -> Environment {
        Rc::new(RwLock::new(Self {
            values: SmallMap::new(),
            enclosing: None,
        }))
    }

    pub fn nested(parent: Environment) -> Environment {
        Rc::new(RwLock::new(Self {
            values: SmallMap::new(),
            enclosing: Some(parent.clone()),
        }))
    }

    #[throws(RuntimeError)]
    fn ancestor(&self, distance: usize) -> Environment {
        let mut parent = self.enclosing.clone();
        for _ in distance..1 {
            if let Some(p) = parent {
                parent = p
                    .read()
                    .map_err(|_| RuntimeError::EnvironmentError("read lock in ancestor"))? // @todo miette!
                    .enclosing
                    .clone();
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
        if let Some(v) = self.values.get(name.lexeme(source())) {
            return v.clone();
        }
        // @todo Use ancestor(distance=1):
        if let Some(parent) = &self.enclosing {
            return parent
                .read()
                .map_err(|_| RuntimeError::EnvironmentError("read lock in get"))? // @todo miette!
                .get(name)?;
        }
        throw!(RuntimeError::UndefinedVariable(
            name.clone(),
            name.to_string()
        ))
    }

    #[throws(RuntimeError)]
    fn get_by_name(&self, name: impl AsRef<str>) -> LiteralValue {
        if let Some(v) = self.values.get(name.as_ref()) {
            return v.clone();
        }
        // @todo Use ancestor(distance=1):
        if let Some(parent) = &self.enclosing {
            return parent
                .read()
                .map_err(|_| RuntimeError::EnvironmentError("read lock in get"))? // @todo miette!
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
            .map_err(|_| RuntimeError::EnvironmentError("read lock in get_at"))? // @todo miette!
            .get(name)?
    }

    #[throws(RuntimeError)]
    fn get_at_by_name(&self, distance: usize, name: impl AsRef<str>) -> LiteralValue {
        if distance == 0 {
            return self.get_by_name(name)?;
        }
        self.ancestor(distance)?
            .read()
            .map_err(|_| RuntimeError::EnvironmentError("read lock in get_at"))? // @todo miette!
            .get_by_name(name)?
    }

    #[throws(RuntimeError)]
    fn assign(&mut self, name: Token, value: LiteralValue) {
        if let Some(v) = self.values.get_mut(&name.to_string()) {
            *v = value;
            return;
        }
        // @todo Use ancestor(distance=1):
        if let Some(parent) = &self.enclosing {
            parent
                .write()
                .map_err(|_| RuntimeError::EnvironmentError("write lock in assign"))? // @todo miette!
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
            .map_err(|_| RuntimeError::EnvironmentError("write lock in assign_at"))? // @todo miette!
            .assign(name, value)?;
    }
}

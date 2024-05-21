use {
    crate::{
        error::RuntimeError,
        scanner::{LiteralValue, Token},
    },
    culpa::{throw, throws},
    std::collections::HashMap,
};

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    #[throws(RuntimeError)]
    pub fn get(&self, name: Token) -> LiteralValue {
        if !self.values.contains_key(&name.lexeme()) {
            throw!(RuntimeError::UndefinedVariable(name.lexeme()))
        }
        self.values.get(&name.lexeme()).unwrap().clone()
    }

    #[throws(RuntimeError)]
    pub fn assign(&mut self, name: Token, value: LiteralValue) {
        if !self.values.contains_key(&name.lexeme()) {
            throw!(RuntimeError::UndefinedVariable(name.lexeme()))
        }
        self.values.entry(name.lexeme()).and_modify(|e| *e = value);
    }
}

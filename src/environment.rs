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
        if self.values.contains_key(&name.lexeme()) {
            return self.values.get(&name.lexeme()).unwrap().clone();
        }
        throw!(RuntimeError::UndefinedVariable(name.lexeme()))
    }
}

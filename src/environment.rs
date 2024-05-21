use {
    crate::scanner::{LiteralValue, Token},
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

    pub fn get(&self, name: Token) -> Option<LiteralValue> {
        if self.values.contains_key(&name.lexeme()) {
            return self.values.get(&name.lexeme()).cloned();
        }
        None
    }
}

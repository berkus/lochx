#[derive(Debug, Clone)]
pub enum LiteralValue {
    Str(String),
    Num(f64),
    Nil,
    Bool(bool),
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LiteralValue::Str(s) => s.clone(),
                LiteralValue::Num(n) => n.to_string().trim_end_matches(".0").to_string(),
                LiteralValue::Nil => "nil".to_string(),
                LiteralValue::Bool(b) => b.to_string(),
            }
        )
    }
}

impl LiteralValue {
    /// nil and false are falsy, everything else is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Nil => false,
            LiteralValue::Bool(b) => *b,
            _ => true,
        }
    }
}

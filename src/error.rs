use {crate::scanner::Token, thiserror::Error};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("invalid assignment target. Expected variable name.")]
    InvalidAssignmentTarget(Token),
}

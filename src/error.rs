use {
    crate::{literal::LiteralValue, scanner::Token},
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Not an error, a function return mechanism")]
    ReturnValue(LiteralValue),
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("invalid assignment target. Expected variable name.")]
    InvalidAssignmentTarget(Token),
    #[error("Expected expression")]
    ExpectedExpression,
    #[error("Too many arguments. Expected less than 256.")]
    TooManyArguments(Token),
    #[error("Can call only functions and classes.")]
    NotACallable,
    #[error("Expected {1} arguments but got {2}.")]
    InvalidArity(Token, usize, usize),
    #[error("Clock may have gone backwards")]
    ClockBackwards,
    #[error("Cannot obtain the environment due to {0}.")]
    EnvironmentError(anyhow::Error),
}

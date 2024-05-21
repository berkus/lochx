use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("undefined variable '{0}'")]
    UndefinedVariable(String),
}

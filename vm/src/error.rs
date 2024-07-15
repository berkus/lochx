use {miette::ErrReport, thiserror::Error};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Usage: {0}")]
    Usage(ErrReport),
    #[error("Could not read from file {0}")]
    IoError(#[from] std::io::Error),
}

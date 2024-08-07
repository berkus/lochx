use {
    crate::{
        literal::LiteralValue,
        scanner::{SourcePosition, Token, TokenType},
    },
    thiserror::Error,
};

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Not an error, a function return mechanism.")]
    ReturnValue(LiteralValue),
    #[error("Return statement at top level.")]
    TopLevelReturn(Token, &'static str), // note
    #[error("Can't return explicit value from initializer.")]
    ValueReturnFromInitializer(Token, &'static str), // note
    #[error("Scanning error.")]
    ScanError { location: SourcePosition },
    #[error("Parsing error.")]
    ParseError {
        token: Token,
        expected: TokenType,
        message: String,
    },
    #[error("Duplicate declaration.")]
    DuplicateDeclaration(Token, &'static str), // note
    #[error("Invalid assignment target. Expected variable name.")]
    InvalidAssignmentTarget(Token, &'static str), // note
    #[error("Expected expression.")]
    ExpectedExpression(Token),
    #[error("Too many arguments. Expected less than 256.")]
    TooManyArguments(Token),
    #[error("Can call only functions and classes.")]
    NotACallable(Token),
    #[error("Expected {1} arguments but got {2}.")]
    InvalidArity(Token, usize, usize),
    #[error("Invalid field/property access.")]
    InvalidPropertyAccess(Token, &'static str), // note
    #[error("Property {0} is undefined.")]
    UndefinedProperty(Token),
    #[error("Undefined variable '{1}'.")]
    UndefinedVariable(Token, String),
    #[error("Undefined variable '{0}'.")]
    UndefinedVariableName(String),
    #[error("`this` keyword outside of class.")]
    NonClassThis(Token, &'static str), // note
    #[error("A class can't inherit from itself.")]
    RecursiveClass(Token),
    #[error("Superclass must be a class.")]
    NotAClassBase(Token),
    #[error("Invalid use of `super`.")]
    InvalidSuper(Token, &'static str), // note
    #[error("Generic error.")]
    GenericError,
    #[error("Clock may have gone backwards.")]
    ClockBackwards,
    #[error("Cannot obtain the environment due to {0}.")]
    EnvironmentError(&'static str),
    #[error("Cannot read source file {0}.")]
    IoError(#[from] std::io::Error),
    #[error("Usage: {0}.")]
    Usage(miette::ErrReport),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Eof,
    Error(&'static str),

    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    KwAnd,
    KwClass,
    KwElse,
    KwFalse,
    KwFun,
    KwFor,
    KwIf,
    KwNil,
    KwOr,
    KwPrint,
    KwReturn,
    KwSuper,
    KwThis,
    KwTrue,
    KwVar,
    KwWhile,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub position: SourcePosition,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub span: std::ops::Range<usize>,
}

impl std::fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}..{}]", self.line, self.span.start, self.span.end)
    }
}

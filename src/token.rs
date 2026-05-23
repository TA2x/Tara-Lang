#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    // keywords
    Make,
    Show,
    When,
    Otherwise,
    During,
    For,
    Return,
    Func,

    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    DoubleEqual,
    NotEqual,
    Not,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Colon,
    Comma,

    EOF,
    Unknown(char),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,// The exact text slice from the source that produced this token.
    pub line: usize,// Line number in the source (1-based), useful for error messages.
    pub column: usize,// Column number in the source (1-based).
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}

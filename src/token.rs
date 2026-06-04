#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),

    Make,
    Show,
    When,
    Otherwise,
    During,
    For,
    Return,
    Func,

    TypeInt,
    TypeFloat,
    TypeBool,
    TypeString,
    TypeVoid,

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
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Self { token_type, lexeme, line, column }
    }
}
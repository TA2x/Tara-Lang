use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Nil,
    StringLit(String),
    Identifier(String),

    // Keywords
    Make,
    Show,
    When,
    Otherwise,
    During,
    For,
    Return,
    Func,
    And,
    Or,

    // Type names used by function declarations
    TypeInt,
    TypeFloat,
    TypeBool,
    TypeString,
    TypeVoid,

    // Arithmetic
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Logical operators
    AndAnd,
    OrOr,

    // Assignment / comparison / logical
    Equal,
    DoubleEqual,
    NotEqual,
    Not,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Colon,
    Comma,

    Eof,
    Unknown(char),
}

impl TokenKind {
    pub fn display(&self) -> String {
        match self {
            TokenKind::Integer(n) => n.to_string(),
            TokenKind::Float(f) => f.to_string(),
            TokenKind::Boolean(b) => b.to_string(),
            TokenKind::Nil => "nil".into(),
            TokenKind::StringLit(s) => format!("\"{}\"", s),
            TokenKind::Identifier(s) => s.clone(),
            TokenKind::Make => "make".into(),
            TokenKind::Show => "show".into(),
            TokenKind::When => "if".into(),
            TokenKind::Otherwise => "else".into(),
            TokenKind::During => "while".into(),
            TokenKind::For => "for".into(),
            TokenKind::Return => "return".into(),
            TokenKind::Func => "func".into(),
            TokenKind::And => "and".into(),
            TokenKind::Or => "or".into(),
            TokenKind::TypeInt => "int".into(),
            TokenKind::TypeFloat => "float".into(),
            TokenKind::TypeBool => "bool".into(),
            TokenKind::TypeString => "String".into(),
            TokenKind::TypeVoid => "void".into(),
            TokenKind::Plus => "+".into(),
            TokenKind::Minus => "-".into(),
            TokenKind::Star => "*".into(),
            TokenKind::Slash => "/".into(),
            TokenKind::Percent => "%".into(),
            TokenKind::AndAnd => "&&".into(),
            TokenKind::OrOr => "||".into(),
            TokenKind::Equal => "=".into(),
            TokenKind::DoubleEqual => "==".into(),
            TokenKind::Not => "!".into(),
            TokenKind::NotEqual => "!=".into(),
            TokenKind::Less => "<".into(),
            TokenKind::LessEqual => "<=".into(),
            TokenKind::Greater => ">".into(),
            TokenKind::GreaterEqual => ">=".into(),
            TokenKind::LeftParen => "(".into(),
            TokenKind::RightParen => ")".into(),
            TokenKind::LeftBrace => "{".into(),
            TokenKind::RightBrace => "}".into(),
            TokenKind::Semicolon => ";".into(),
            TokenKind::Colon => ":".into(),
            TokenKind::Comma => ",".into(),
            TokenKind::Eof => "<EOF>".into(),
            TokenKind::Unknown(c) => c.to_string(),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, col: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            col,
        }
    }
}

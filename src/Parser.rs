use crate::ast::{BinaryOperator, Expr, Program, Stmt, UnaryOperator};
use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError{
    fn new(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            message: message.Into(),
            line,
            column,
        }
    }
}

impl std:fmt::Display for ParseError {
    fn fmt(&Self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "[ParseError line {},col{}] {}", self.line, self.column, self.message
        )
    }
}

pub struct Parser {
    token_list: Vec<Token>,
    
}
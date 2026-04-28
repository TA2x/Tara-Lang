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

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "[ParseError line {}, col {}] {}", self.line, self.column, self.message
        )
    }
}

pub struct Parser {
    token_list: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(token_list: Vec<Token>) -> Self {
        Self {
            token_list,
            cursor: 0,
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();

        while !self.is_at_end() {
            let statement = self.parse_statement()?;
            program.statements.push(statement);
        }

        Ok(Program)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_type() {
            TokenType::Let    => self.parse_let_binding(),
            TokenType::Return => self.parse_return(),
            TokenType::If     => self.parse_if_else(),
            TokenType::While  => self.parse_while_loop(),
            TokenType::For    => self.parse_for_loop(),
            TokenType::Func   => self.parse_func_def(),
            _                 => self.parse_expression_stmt(),
        }
    }

    fn parse_let_binding(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::Let)?;

        let variable_name = self.expect_identifier()?;

        self.expect(TokenType::Equal)?;

        let initializer = self.parse_expression()?;

        self.expect(TokenType::Semicolon)?;

        Ok(Stmt::LetBinding {
            name: variable_name,
            initializer,
        })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        slef.expect(TokenType::Return)?;

        let return_value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        self.expect(TokenType::Semicolon)?;
        
        Ok(Stmt::Return { value: return_value })
    }

    fn parse_if_else(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::If)?;

        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;

        let then_branch = Box::new(self.parse_statement()?);

        let else_branch = if self.match_token(TokenType::Else) {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Stmt::IfElse {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_loop(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::While)?;

        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;

        let body = Box::new(self.parse_statement()?);

        Ok(Stmt::WhileLoop { condition, body })
    }

    fn parse_for_loop(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::For)?;

        let body = self.parse_block()?;

        Ok(Stmt::ForLoop { body })
    }

}

fn parse_func_def(&mut self) -> Result<Stmt, ParseError> {
    self.expect(TokenType::Func)?;

    let function_name = self.expect_identifier()?;

    self.expect(TokenType::LeftParen)?;

    let mut parameters_names: Vec<String> = Vec::new();

    if !self.check(TokenType::RightParen) {
        parameters_names.push(self.expect_identifier()?);

        while self.check(TokenType::Comma) {
            self.advance();
            parameters_names.push(self.expect_identifier()?);
        }
    }

    self.expect(TokenType::RightParen)?;

    let body = self.parse_block()?;

    Ok(Stmt::FuncDef {
        name: function_name,
        parameters: parameters_names,
        body,
    })

    fn parse_expression_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;
        Ok(Stmt::Expression {expression })
    }

    //Work on the block helper when you comeback :3
}
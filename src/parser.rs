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
            message: message.into(),
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

        Ok(program)
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
        self.expect(TokenType::Return)?;

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

        let then_branch = self.parse_block()?;

        let else_branch = if self.check(TokenType::Else) {
            self.advance();
            Some(self.parse_block()?)
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

        let body = self.parse_block()?;

        Ok(Stmt::WhileLoop { condition, body })
    }

    fn parse_for_loop(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::For)?;

        let body = self.parse_block()?;

        Ok(Stmt::ForLoop { body })
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
    }


    fn parse_expression_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expression = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;
        Ok(Stmt::ExpressionStmt(expression))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(TokenType::LeftBrace)?;

        let mut statements: Vec<Stmt> = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenType::RightBrace)?;

        Ok(statements)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;

        loop {
            let operator = match self.peek_type() {
                TokenType::DoubleEqual => BinaryOperator::DoubleEqual,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => break,
            };

            self.advance();

            let right = self.parse_additive()?;

            left = Expr::BinaryOp {
                left_operand: Box::new(left),
                operator,
                right_operand: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let operator = match self.peek_type() {
                TokenType::Plus => BinaryOperator::Add,
                TokenType::Minus => BinaryOperator::Subtract,
                _ => break,
            };

            self.advance();

            let right = self.parse_multiplicative()?;

            left = Expr::BinaryOp {
                left_operand: Box::new(left),
                operator,
                right_operand: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let operator = match self.peek_type() {
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                _ => break,
            };

            self.advance();

            let right = self.parse_unary()?;

            left = Expr::BinaryOp {
                left_operand: Box::new(left),
                operator,
                right_operand: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let operator = match self.peek_type() {
            TokenType::Minus => Some(UnaryOperator::Negate),
            TokenType::Not => Some(UnaryOperator::Not),
            _ => None,
        };
        if let Some(unary_op) = operator {
            self.advance();

            let operand = self.parse_unary()?;

            return Ok(Expr::UnaryOp {
                operator: unary_op,
                operand: Box::new(operand),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {

        match self.peek_type() {
            TokenType::Integer(_) => {
                let consumed_token = self.advance().unwrap();
                if let TokenType::Integer(numeric_value) = consumed_token.token_type {
                    Ok(Expr::Integer(numeric_value))
                } else {
                    unreachable!("peek guaranteed Integer variant")
                }
            }

            TokenType::Float(_) => {
                let consumed_token = self.advance().unwrap();
                if let TokenType::Float(numeric_value) = consumed_token.token_type {
                    Ok(Expr::Float(numeric_value))
                } else {
                    unreachable!("peek guaranteed Float variant")
                }
            }

            TokenType::Identifier(_) => {
                let consumed_token = self.advance().unwrap();
                let identifier_name = match consumed_token.token_type {
                    TokenType::Identifier(name) => name,
                    _ => unreachable!("peek guaranteed Identifier variant"),
                };

                if self.check(TokenType::LeftParen) {
                    self.advance();

                    let mut arguments: Vec<Expr> = Vec::new();

                    if !self.check(TokenType::RightParen) {
                        arguments.push(self.parse_expression()?);

                        while self.check(TokenType::Comma) {
                            self.advance();
                            arguments.push(self.parse_expression()?);
                        }
                    }

                    self.expect(TokenType::RightParen)?;

                    Ok(Expr::Call {
                        callee: identifier_name,
                        arguments,
                    })
                } else {
                    Ok(Expr::Identifier(identifier_name))
                }
            }

            TokenType::LeftParen => {
                self.advance();
                let inner_expr = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(Expr::Grouped(Box::new(inner_expr)))
            }

            _ => {
                let bad_token = self.current_token();
                Err(ParseError::new(
                    format!(
                        "unexpected token '{}' - expected a literal, identifier, or '('",
                        bad_token.lexeme
                    ),
                    bad_token.line,
                    bad_token.column,
                ))
            }
        }
    }

    fn peek_type(&self) -> TokenType {
        self.token_list
            .get(self.cursor)
            .map(|token| token.token_type.clone())
            .unwrap_or(TokenType::EOF)
    }

    fn check(&self, expected_type: TokenType) -> bool {
        std::mem::discriminant(&self.peek_type()) == std::mem::discriminant(&expected_type)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.cursor < self.token_list.len() {
            let consumed = std::mem::replace(
                &mut self.token_list[self.cursor],
                Token::new(TokenType::EOF, "\0".to_string(), 0, 0),
            );
            self.cursor += 1;
            Some(consumed)
        } else {
            None
        }
    }

    fn expect(&mut self, expected_type: TokenType) -> Result<Token, ParseError> {
        if self.check(expected_type.clone()) {
            Ok(self.advance().unwrap())
        } else {
            let bad_token = self.current_token();
            Err(ParseError::new(
                format!(
                    "expected '{}' but found '{}'",
                    token_type_display(&expected_type),
                    bad_token.lexeme
                ),
                bad_token.line,
                bad_token.column,
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
         if matches!(self.peek_type(), TokenType::Identifier(_)) {
            let consumed = self.advance().unwrap();
            match consumed.token_type {
                TokenType::Identifier(name) => Ok(name),
                _ => unreachable!("matches! guaranteed Identifier variant"),
            }
         } else {
            let bad_token = self.current_token();
            Err(ParseError::new(
                format!(
                    "expected identifier but found '{}'",
                    bad_token.lexeme
                ),
                bad_token.line,
                bad_token.column,
            ))
         }
    }

    fn current_token(&self) -> &Token {
        self.token_list
            .get(self.cursor)
            .or_else(|| self.token_list.last())
            .expect("token_list must not be empty - lexer always appends EOF")
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_type(), TokenType::EOF)
    }
}

fn token_type_display(token_type: &TokenType) -> &'static str {
    match token_type {
        TokenType::Let          => "let",
        TokenType::If           => "if",
        TokenType::Else         => "else",
        TokenType::While        => "while",
        TokenType::For          => "for",
        TokenType::Return       => "return",
        TokenType::Func         => "func",
        TokenType::Plus         => "+",
        TokenType::Minus        => "-",
        TokenType::Star         => "*",
        TokenType::Slash        => "/",
        TokenType::Equal        => "=",
        TokenType::DoubleEqual  => "==",
        TokenType::Not          => "!",
        TokenType::NotEqual     => "!=",
        TokenType::Less         => "<",
        TokenType::LessEqual    => "<=",
        TokenType::Greater      => ">",
        TokenType::GreaterEqual => ">=",
        TokenType::LeftParen    => "(",
        TokenType::RightParen   => ")",
        TokenType::LeftBrace    => "{",
        TokenType::RightBrace   => "}",
        TokenType::Semicolon    => ";",
        TokenType::Colon        => ":",
        TokenType::Comma        => ",",
        TokenType::EOF          => "<EOF>",
        _                       => "<token>",
    }
}
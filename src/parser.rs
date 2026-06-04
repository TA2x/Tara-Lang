use crate::ast::{BinaryOperator, Expr, Program, Stmt, TypeAnnotation, UnaryOperator};
use crate::token::{Token, TokenType};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseError {
    fn new(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self { message: message.into(), line, column }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ParseError line {}, col {}] {}", self.line, self.column, self.message)
    }
}

pub struct Parser {
    token_list: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(token_list: Vec<Token>) -> Self {
        Self { token_list, cursor: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();
        while !self.is_at_end() {
            program.statements.push(self.parse_statement()?);
        }
        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        let stmt = match self.peek_type() {
            TokenType::Make   => self.parse_make()?,
            TokenType::Show   => self.parse_show()?,
            TokenType::Return => self.parse_return()?,
            TokenType::When   => self.parse_when()?,
            TokenType::During => self.parse_during()?,
            TokenType::For    => self.parse_for()?,
            TokenType::Func   => self.parse_func_def()?,
            _                 => self.parse_expression_stmt()?,
        };

        // Optional semicolon after most statements (but not after blocks)
        match &stmt {
            Stmt::When { .. } | Stmt::During { .. } | Stmt::For { .. } | Stmt::FuncDef { .. } => {
                // These have blocks, no semicolon needed
            }
            _ => {
                if self.check(TokenType::Semicolon) {
                    self.advance();
                }
            }
        }

        Ok(stmt)
    }

    fn parse_make(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::Make)?;
        let name        = self.expect_identifier()?;
        self.expect(TokenType::Equal)?;
        let initializer = self.parse_expression()?;
        // Don't consume semicolon here - let the caller decide
        Ok(Stmt::Make { name, initializer })
    }

    fn parse_show(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::Show)?;
        self.expect(TokenType::LeftParen)?;

        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            arguments.push(self.parse_expression()?);
            while self.check(TokenType::Comma) {
                self.advance();
                arguments.push(self.parse_expression()?);
            }
        }

        self.expect(TokenType::RightParen)?;
        Ok(Stmt::Show { arguments })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::Return)?;
        let value = if self.check(TokenType::Semicolon) || self.check(TokenType::RightBrace) || self.is_at_end() {
            None
        } else {
            Some(self.parse_expression()?)
        };
        Ok(Stmt::Return { value })
    }

    fn parse_when(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::When)?;
        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;

        let then_branch = self.parse_block()?;
        let otherwise_branch = if self.check(TokenType::Otherwise) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::When { condition, then_branch, otherwise_branch })
    }

    fn parse_during(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::During)?;
        self.expect(TokenType::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenType::RightParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::During { condition, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::For)?;
        self.expect(TokenType::LeftParen)?;

        let init = if self.check(TokenType::Make) {
            let stmt = self.parse_make()?;
            // parse_make already handles optional semicolon, but in for loop we need it
            self.expect(TokenType::Semicolon)?;
            Some(Box::new(stmt))
        } else if self.check(TokenType::Semicolon) {
            self.advance();
            None
        } else {
            let expr = self.parse_expression()?;
            self.expect(TokenType::Semicolon)?;
            Some(Box::new(Stmt::ExpressionStmt(expr)))
        };

        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenType::Semicolon)?;

        let update = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };
        self.expect(TokenType::RightParen)?;

        let body = self.parse_block()?;
        Ok(Stmt::For { init, condition, update, body })
    }

    fn parse_func_def(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenType::Func)?;
        let return_type = self.parse_type_annotation()?;
        let name        = self.expect_identifier()?;

        self.expect(TokenType::LeftParen)?;

        let mut parameters: Vec<(String, TypeAnnotation)> = Vec::new();
        if !self.check(TokenType::RightParen) {
            parameters.push(self.parse_parameter()?);
            while self.check(TokenType::Comma) {
                self.advance();
                parameters.push(self.parse_parameter()?);
            }
        }

        self.expect(TokenType::RightParen)?;
        let body = self.parse_block()?;

        Ok(Stmt::FuncDef { name, return_type, parameters, body })
    }

    fn parse_parameter(&mut self) -> Result<(String, TypeAnnotation), ParseError> {
        let param_name = self.expect_identifier()?;
        self.expect(TokenType::Colon)?;
        let param_type = self.parse_type_annotation()?;
        Ok((param_name, param_type))
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, ParseError> {
        let annotation = match self.peek_type() {
            TokenType::TypeInt    => TypeAnnotation::Int,
            TokenType::TypeFloat  => TypeAnnotation::Float,
            TokenType::TypeBool   => TypeAnnotation::Bool,
            TokenType::TypeString => TypeAnnotation::Str,
            TokenType::TypeVoid   => TypeAnnotation::Void,
            _ => {
                let bad = self.current_token();
                return Err(ParseError::new(
                    format!("expected a type (int, float, bool, String, void) but found '{}'", bad.lexeme),
                    bad.line,
                    bad.column,
                ));
            }
        };
        self.advance();
        Ok(annotation)
    }
    fn parse_expression_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression()?;
        Ok(Stmt::ExpressionStmt(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(TokenType::LeftBrace)?;
        let mut stmts = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.expect(TokenType::RightBrace)?;
        Ok(stmts)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_comparison()?;

        if self.check(TokenType::Equal) {
            self.advance();
            let value = self.parse_assignment()?;
            match expr {
                Expr::Identifier(name) => return Ok(Expr::Assignment { name, value: Box::new(value) }),
                _ => {
                    let bad = self.current_token();
                    return Err(ParseError::new("Invalid assignment target", bad.line, bad.column));
                }
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;

        loop {
            let op = match self.peek_type() {
                TokenType::DoubleEqual  => BinaryOperator::DoubleEqual,
                TokenType::NotEqual     => BinaryOperator::NotEqual,
                TokenType::Less         => BinaryOperator::Less,
                TokenType::LessEqual    => BinaryOperator::LessEqual,
                TokenType::Greater      => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp { left_operand: Box::new(left), operator: op, right_operand: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let op = match self.peek_type() {
                TokenType::Plus  => BinaryOperator::Add,
                TokenType::Minus => BinaryOperator::Subtract,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp { left_operand: Box::new(left), operator: op, right_operand: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek_type() {
                TokenType::Star  => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp { left_operand: Box::new(left), operator: op, right_operand: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let op = match self.peek_type() {
            TokenType::Minus => Some(UnaryOperator::Negate),
            TokenType::Not   => Some(UnaryOperator::Not),
            _                => None,
        };

        if let Some(unary_op) = op {
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::UnaryOp { operator: unary_op, operand: Box::new(operand) });
        }

        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek_type() {
            TokenType::Integer(_) => {
                let token = self.advance().unwrap();
                if let TokenType::Integer(n) = token.token_type { Ok(Expr::Integer(n)) }
                else { unreachable!() }
            }
            TokenType::Float(_) => {
                let token = self.advance().unwrap();
                if let TokenType::Float(f) = token.token_type { Ok(Expr::Float(f)) }
                else { unreachable!() }
            }
            TokenType::Boolean(_) => {
                let token = self.advance().unwrap();
                if let TokenType::Boolean(b) = token.token_type { Ok(Expr::Boolean(b)) }
                else { unreachable!() }
            }
            TokenType::String(_) => {
                let token = self.advance().unwrap();
                if let TokenType::String(s) = token.token_type { Ok(Expr::String(s)) }
                else { unreachable!() }
            }
            TokenType::Identifier(_) => {
                let token = self.advance().unwrap();
                let name = match token.token_type {
                    TokenType::Identifier(n) => n,
                    _ => unreachable!(),
                };

                if self.check(TokenType::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(TokenType::RightParen) {
                        args.push(self.parse_expression()?);
                        while self.check(TokenType::Comma) {
                            self.advance();
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.expect(TokenType::RightParen)?;
                    Ok(Expr::Call { callee: name, arguments: args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            TokenType::LeftParen => {
                self.advance();
                let inner = self.parse_expression()?;
                self.expect(TokenType::RightParen)?;
                Ok(Expr::Grouped(Box::new(inner)))
            }
            _ => {
                let bad = self.current_token();
                Err(ParseError::new(
                    format!("unexpected token '{}' — expected a literal, identifier, or '('", bad.lexeme),
                    bad.line,
                    bad.column,
                ))
            }
        }
    }

    fn peek_type(&self) -> TokenType {
        self.token_list
            .get(self.cursor)
            .map(|t| t.token_type.clone())
            .unwrap_or(TokenType::EOF)
    }

    fn check(&self, expected: TokenType) -> bool {
        std::mem::discriminant(&self.peek_type()) == std::mem::discriminant(&expected)
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

    fn expect(&mut self, expected: TokenType) -> Result<Token, ParseError> {
        if self.check(expected.clone()) {
            Ok(self.advance().unwrap())
        } else {
            let bad = self.current_token();
            Err(ParseError::new(
                format!("expected '{}' but found '{}'", token_type_display(&expected), bad.lexeme),
                bad.line,
                bad.column,
            ))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if matches!(self.peek_type(), TokenType::Identifier(_)) {
            let token = self.advance().unwrap();
            match token.token_type {
                TokenType::Identifier(name) => Ok(name),
                _ => unreachable!(),
            }
        } else {
            let bad = self.current_token();
            Err(ParseError::new(
                format!("expected identifier but found '{}'", bad.lexeme),
                bad.line,
                bad.column,
            ))
        }
    }

    fn current_token(&self) -> &Token {
        self.token_list
            .get(self.cursor)
            .or_else(|| self.token_list.last())
            .expect("token_list is never empty")
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek_type(), TokenType::EOF)
    }
}

fn token_type_display(t: &TokenType) -> &'static str {
    match t {
        TokenType::Make         => "make",
        TokenType::Show         => "show",
        TokenType::When         => "when",
        TokenType::Otherwise    => "otherwise",
        TokenType::During       => "during",
        TokenType::For          => "for",
        TokenType::Return       => "return",
        TokenType::Func         => "func",
        TokenType::TypeInt      => "int",
        TokenType::TypeFloat    => "float",
        TokenType::TypeBool     => "bool",
        TokenType::TypeString   => "String",
        TokenType::TypeVoid     => "void",
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
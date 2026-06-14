use crate::ast::{BinOp, Expr, Program, Stmt, TypeAnnotation, UnOp};
use crate::token::{Token, TokenKind};

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl ParseError {
    fn new(message: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            message: message.into(),
            line,
            col,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ParseError line {}, col {}] {}",
            self.line, self.col, self.message
        )
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();
        while !self.at_end() {
            if self.check(&TokenKind::Semicolon) {
                self.advance();
                continue;
            }
            program.stmts.push(self.parse_stmt()?);
        }
        Ok(program)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let stmt = match self.peek_kind() {
            TokenKind::Make => self.parse_make()?,
            TokenKind::Show => self.parse_show()?,
            TokenKind::Return => self.parse_return()?,
            TokenKind::When => self.parse_when()?,
            TokenKind::During => self.parse_during()?,
            TokenKind::For => self.parse_for()?,
            TokenKind::Func => self.parse_func_def()?,
            _ => self.parse_expr_stmt()?,
        };

        if self.check(&TokenKind::Semicolon) {
            self.advance();
        }

        Ok(stmt)
    }

    fn parse_make(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Make)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Equal)?;
        let init = self.parse_expr()?;
        Ok(Stmt::Make { name, init })
    }

    fn parse_show(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Show)?;
        self.expect(TokenKind::LeftParen)?;
        let args = self.parse_expr_list(TokenKind::RightParen)?;
        self.expect(TokenKind::RightParen)?;
        Ok(Stmt::Show { args })
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Return)?;
        let value = if self.check(&TokenKind::Semicolon)
            || self.check(&TokenKind::RightBrace)
            || self.at_end()
        {
            None
        } else {
            Some(self.parse_expr()?)
        };
        Ok(Stmt::Return { value })
    }

    fn parse_when(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::When)?;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RightParen)?;
        let then_body = self.parse_block()?;
        let else_body = if self.check(&TokenKind::Otherwise) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Stmt::When {
            condition,
            then_body,
            else_body,
        })
    }

    fn parse_during(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::During)?;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RightParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::During { condition, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::For)?;
        self.expect(TokenKind::LeftParen)?;

        let init = if self.check(&TokenKind::Make) {
            let decl = self.parse_make()?;
            self.expect(TokenKind::Semicolon)?;
            Some(Box::new(decl))
        } else if self.check(&TokenKind::Semicolon) {
            self.advance();
            None
        } else {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semicolon)?;
            Some(Box::new(Stmt::ExprStmt(expr)))
        };

        let condition = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.expect(TokenKind::Semicolon)?;

        let update = if self.check(&TokenKind::RightParen) {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        self.expect(TokenKind::RightParen)?;

        let body = self.parse_block()?;
        Ok(Stmt::For {
            init,
            condition,
            update,
            body,
        })
    }

    fn parse_func_def(&mut self) -> Result<Stmt, ParseError> {
        self.expect(TokenKind::Func)?;

        let return_type = if self.peek_is_type() {
            Some(self.parse_type_annotation()?)
        } else {
            None
        };

        let name = self.expect_ident()?;
        self.expect(TokenKind::LeftParen)?;

        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            params.push(self.parse_parameter()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                params.push(self.parse_parameter()?);
            }
        }

        self.expect(TokenKind::RightParen)?;
        let body = self.parse_block()?;
        Ok(Stmt::FuncDef {
            name,
            return_type,
            params,
            body,
        })
    }

    fn parse_parameter(&mut self) -> Result<String, ParseError> {
        let name = self.expect_ident()?;
        if self.check(&TokenKind::Colon) {
            self.advance();
            self.parse_type_annotation()?;
        }
        Ok(name)
    }

    fn parse_type_annotation(&mut self) -> Result<TypeAnnotation, ParseError> {
        let annotation = match self.peek_kind() {
            TokenKind::TypeInt => TypeAnnotation::Int,
            TokenKind::TypeFloat => TypeAnnotation::Float,
            TokenKind::TypeBool => TypeAnnotation::Bool,
            TokenKind::TypeString => TypeAnnotation::Str,
            TokenKind::TypeVoid => TypeAnnotation::Void,
            _ => {
                let tok = self.current_token();
                return Err(ParseError::new(
                    format!("expected a type but found '{}'", tok.lexeme),
                    tok.line,
                    tok.col,
                ));
            }
        };
        self.advance();
        Ok(annotation)
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        Ok(Stmt::ExprStmt(self.parse_expr()?))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.expect(TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.at_end() {
            if self.check(&TokenKind::Semicolon) {
                self.advance();
                continue;
            }
            stmts.push(self.parse_stmt()?);
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(stmts)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_equality_and_comparison()?;

        if self.check(&TokenKind::Equal) {
            self.advance();
            let rhs = self.parse_assignment()?;
            match lhs {
                Expr::Identifier(name) => Ok(Expr::Assignment {
                    name,
                    value: Box::new(rhs),
                }),
                _ => {
                    let tok = self.current_token();
                    Err(ParseError::new(
                        "invalid assignment target",
                        tok.line,
                        tok.col,
                    ))
                }
            }
        } else {
            Ok(lhs)
        }
    }

    fn parse_equality_and_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::DoubleEqual => BinOp::Eq,
                TokenKind::NotEqual => BinOp::NotEq,
                TokenKind::Less => BinOp::Lt,
                TokenKind::LessEqual => BinOp::LtEq,
                TokenKind::Greater => BinOp::Gt,
                TokenKind::GreaterEqual => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let op = match self.peek_kind() {
            TokenKind::Minus => Some(UnOp::Neg),
            TokenKind::Not => Some(UnOp::Not),
            _ => None,
        };

        if let Some(op) = op {
            self.advance();
            let operand = self.parse_unary()?;
            Ok(Expr::UnaryOp {
                op,
                operand: Box::new(operand),
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek_kind() {
            TokenKind::Integer(_) => {
                let tok = self.advance().unwrap();
                match tok.kind {
                    TokenKind::Integer(n) => Ok(Expr::Integer(n)),
                    _ => unreachable!(),
                }
            }
            TokenKind::Float(_) => {
                let tok = self.advance().unwrap();
                match tok.kind {
                    TokenKind::Float(f) => Ok(Expr::Float(f)),
                    _ => unreachable!(),
                }
            }
            TokenKind::Boolean(_) => {
                let tok = self.advance().unwrap();
                match tok.kind {
                    TokenKind::Boolean(b) => Ok(Expr::Boolean(b)),
                    _ => unreachable!(),
                }
            }
            TokenKind::Nil => {
                self.advance();
                Ok(Expr::Nil)
            }
            TokenKind::StringLit(_) => {
                let tok = self.advance().unwrap();
                match tok.kind {
                    TokenKind::StringLit(s) => Ok(Expr::StringLit(s)),
                    _ => unreachable!(),
                }
            }
            TokenKind::Identifier(_) => {
                let tok = self.advance().unwrap();
                let name = match tok.kind {
                    TokenKind::Identifier(n) => n,
                    _ => unreachable!(),
                };
                if self.check(&TokenKind::LeftParen) {
                    self.advance();
                    let args = self.parse_expr_list(TokenKind::RightParen)?;
                    self.expect(TokenKind::RightParen)?;
                    Ok(Expr::Call { callee: name, args })
                } else {
                    Ok(Expr::Identifier(name))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(TokenKind::RightParen)?;
                Ok(Expr::Grouped(Box::new(inner)))
            }
            TokenKind::Unknown(ch) => {
                let tok = self.current_token();
                Err(ParseError::new(
                    format!("unknown character '{}'", ch),
                    tok.line,
                    tok.col,
                ))
            }
            _ => {
                let tok = self.current_token();
                Err(ParseError::new(
                    format!(
                        "unexpected '{}' — expected a literal, identifier, or '('",
                        tok.lexeme
                    ),
                    tok.line,
                    tok.col,
                ))
            }
        }
    }

    fn parse_expr_list(&mut self, terminator: TokenKind) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = Vec::new();
        if !self.check(&terminator) {
            exprs.push(self.parse_expr()?);
            while self.check(&TokenKind::Comma) {
                self.advance();
                exprs.push(self.parse_expr()?);
            }
        }
        Ok(exprs)
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens
            .get(self.cursor)
            .map(|t| t.kind.clone())
            .unwrap_or(TokenKind::Eof)
    }

    fn peek_is_type(&self) -> bool {
        matches!(
            self.peek_kind(),
            TokenKind::TypeInt
                | TokenKind::TypeFloat
                | TokenKind::TypeBool
                | TokenKind::TypeString
                | TokenKind::TypeVoid
        )
    }

    fn check(&self, expected: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek_kind()) == std::mem::discriminant(expected)
    }

    fn advance(&mut self) -> Option<Token> {
        if self.cursor < self.tokens.len() {
            let dummy = Token::new(TokenKind::Eof, "\0".into(), 0, 0);
            let consumed = std::mem::replace(&mut self.tokens[self.cursor], dummy);
            self.cursor += 1;
            Some(consumed)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        if self.check(&expected) {
            Ok(self.advance().unwrap())
        } else {
            let tok = self.current_token();
            Err(ParseError::new(
                format!(
                    "expected '{}' but found '{}'",
                    expected.display(),
                    tok.lexeme
                ),
                tok.line,
                tok.col,
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        if matches!(self.peek_kind(), TokenKind::Identifier(_)) {
            let tok = self.advance().unwrap();
            match tok.kind {
                TokenKind::Identifier(name) => Ok(name),
                _ => unreachable!(),
            }
        } else {
            let tok = self.current_token();
            Err(ParseError::new(
                format!("expected identifier but found '{}'", tok.lexeme),
                tok.line,
                tok.col,
            ))
        }
    }

    fn current_token(&self) -> &Token {
        self.tokens
            .get(self.cursor)
            .or_else(|| self.tokens.last())
            .expect("token list is never empty")
    }

    fn at_end(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }
}

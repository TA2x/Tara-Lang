use crate::token::{Token, TokenKind};

pub struct Lexer<'src> {
    #[allow(dead_code)]
    source: &'src str,
    chars: Vec<char>,
    cursor: usize,
    line: usize,
    col: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            cursor: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            self.skip_whitespace();

            let tok_line = self.line;
            let tok_col = self.col;

            let kind = match self.peek() {
                None => TokenKind::Eof,
                Some('#') => {
                    self.skip_comment();
                    continue;
                }
                Some('"') => self.scan_string(),

                Some('+') => {
                    self.advance();
                    TokenKind::Plus
                }
                Some('-') => {
                    self.advance();
                    TokenKind::Minus
                }
                Some('*') => {
                    self.advance();
                    TokenKind::Star
                }
                Some('/') => {
                    self.advance();
                    TokenKind::Slash
                }
                Some('%') => {
                    self.advance();
                    TokenKind::Percent
                }
                Some('&') => {
                    self.advance();
                    if self.peek() == Some('&') {
                        self.advance();
                        TokenKind::AndAnd
                    } else {
                        TokenKind::Unknown('&')
                    }
                }
                Some('|') => {
                    self.advance();
                    if self.peek() == Some('|') {
                        self.advance();
                        TokenKind::OrOr
                    } else {
                        TokenKind::Unknown('|')
                    }
                }
                Some('(') => {
                    self.advance();
                    TokenKind::LeftParen
                }
                Some(')') => {
                    self.advance();
                    TokenKind::RightParen
                }
                Some('{') => {
                    self.advance();
                    TokenKind::LeftBrace
                }
                Some('}') => {
                    self.advance();
                    TokenKind::RightBrace
                }
                Some(';') => {
                    self.advance();
                    TokenKind::Semicolon
                }
                Some(':') => {
                    self.advance();
                    TokenKind::Colon
                }
                Some(',') => {
                    self.advance();
                    TokenKind::Comma
                }
                Some('=') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::DoubleEqual
                    } else {
                        TokenKind::Equal
                    }
                }
                Some('!') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::NotEqual
                    } else {
                        TokenKind::Not
                    }
                }
                Some('<') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    }
                }
                Some('>') => {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::GreaterEqual
                    } else {
                        TokenKind::Greater
                    }
                }

                Some(ch) if ch.is_ascii_digit() => self.scan_number(),
                Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => self.scan_word(),
                Some(ch) => {
                    self.advance();
                    TokenKind::Unknown(ch)
                }
            };

            let lexeme = kind.display();
            return Token::new(kind, lexeme, tok_line, tok_col);
        }
    }

    fn scan_number(&mut self) -> TokenKind {
        let start = self.cursor;

        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }

        let is_float =
            self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit());

        if is_float {
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
            let text: String = self.chars[start..self.cursor].iter().collect();
            TokenKind::Float(text.parse().unwrap_or(0.0))
        } else {
            let text: String = self.chars[start..self.cursor].iter().collect();
            TokenKind::Integer(text.parse().unwrap_or(0))
        }
    }

    fn scan_word(&mut self) -> TokenKind {
        let start = self.cursor;

        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.advance();
        }

        let word: String = self.chars[start..self.cursor].iter().collect();

        match word.as_str() {
            "make" => TokenKind::Make,
            "show" => TokenKind::Show,
            "if" | "when" => TokenKind::When,
            "else" | "otherwise" => TokenKind::Otherwise,
            "while" | "during" => TokenKind::During,
            "for" => TokenKind::For,
            "return" => TokenKind::Return,
            "func" => TokenKind::Func,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "true" => TokenKind::Boolean(true),
            "false" => TokenKind::Boolean(false),
            "nil" => TokenKind::Nil,
            "int" => TokenKind::TypeInt,
            "float" => TokenKind::TypeFloat,
            "bool" => TokenKind::TypeBool,
            "String" => TokenKind::TypeString,
            "void" => TokenKind::TypeVoid,
            _ => TokenKind::Identifier(word),
        }
    }

    fn scan_string(&mut self) -> TokenKind {
        self.advance(); // opening "
        let mut buf = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return TokenKind::StringLit(buf);
                }
                '\\' => {
                    self.advance();
                    match self.peek() {
                        Some('n') => {
                            self.advance();
                            buf.push('\n');
                        }
                        Some('t') => {
                            self.advance();
                            buf.push('\t');
                        }
                        Some('r') => {
                            self.advance();
                            buf.push('\r');
                        }
                        Some('\\') => {
                            self.advance();
                            buf.push('\\');
                        }
                        Some('"') => {
                            self.advance();
                            buf.push('"');
                        }
                        Some(unk) => {
                            buf.push('\\');
                            buf.push(unk);
                            self.advance();
                        }
                        None => break,
                    }
                }
                _ => {
                    buf.push(ch);
                    self.advance();
                }
            }
        }

        TokenKind::StringLit(buf)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.cursor).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.cursor + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.cursor).copied()?;
        self.cursor += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        while self.peek().is_some_and(|c| c != '\n') {
            self.advance();
        }
    }
}

use crate::token::{Token, TokenType};

pub struct Lexer<'a> {
    source: &'a str, //full source code as a string **borrowed**
    char_list: Vec<char>, //list of characters from the source code in vector
    cursor: usize, //index of the next character to read
    line: usize, //current line number (increment on '\n')
    column: usize, //current column number (resets on '\n')
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            char_list: source.chars().collect(),
            cursor: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token_line = self.current_line;
        let token_column = self.current_column;

        let token_type = match self.peek(){
            None => TokenType::EOF,

            Some('/') => {
                self.advance();
                if self.peek() == Some('/') {
                    self.skip_line_comment();
                    return self.next_token(); // Skip comment and get next token
                }
                TokenType::Slash
            }

            Some('+') => {
                self.advance();
                TokenType::Plus
            }
            Some('-') => {
                self.advance();
                TokenType::Minus
            }
            Some('*') => {
                self.advance();
                TokenType::Star
            }
            Some('(') => {
                self.advance();
                TokenType::LeftParen
            }
            Some(')') => {
                self.advance();
                TokenType::RightParen
            }
            Some('{') => {
                self.advance();
                TokenType::LeftBrace
            }
            Some('}') => {
                self.advance();
                TokenType::RightBrace
            }
            Some(';') => {
                self.advance();
                TokenType::Semicolon
            }
            Some(':') => {
                self.advance();
                TokenType::Colon
            }
            Some(',') => {
                self.advance();
                TokenType::Comma
            }
            Some('=') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::DoubleEqual
                } else {
                    TokenType::Equal
                }
            }
            Some('!') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Not
                }
            }
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            Some('>') => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            Some(ch) if is_digit_asm(ch) => self.scan_number(),
            Some(ch) if is_alpha_asm(ch) => self.scan_identifier_or_keyword(),
            Some(ch) => {
                self.advance();
                TokenType::Unknown(ch)
            }
        };

        let lexeme_start = self.cursor.saturating_sub(
            self.char_list[..self.cursor]
            .iter()
            .collect::<String>()
            .len()
        );
        let lexeme = type_to_lexeme(&token_type, &self.char_list, self.cursor);

        Token::new(token_type, lexeme, token_line, token_column)
    }

    fn scan_number(&mut self) -> TokenType {
        let start = self.cursor;
        
        while self.peek().map_or(false, is_digit_asm){
            self.advance();
        }

        let is_float = self.peek() == Some('.') && self.peek_next().map_or(false, is_digit_asm);

        if is_float {
            self.advance(); // consume '.'
            while self.peek().map_or(false, is_digit_asm){
                self.advance();
            }
            let text: String = self.char_list[start..self.cursor].iter().collect();
            let value: f64 = text.parse().unwrap_or(0.0);
            TokenType::Float(value)
        } else {
            let text: String = self.char_list[start..self.cursor].iter().collect();
            let value: i64 = text.parse().unwrap_or(0);
            TokenType::Integer(value)
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> TokenType {
        let start = self.cursor;
        while self.peek().map_or(false, |ch| is_alpha_asm(ch) || is_digit_asm(ch) || ch == '_') {
            self.advance();
        }

        let word: String = self.char_list[start..self.cursor].iter().collect();

        match word.as_str() {
            "let" => TokenType::Let,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "func" => TokenType::Func,
            _ => TokenType::Identifier(word),
        }
    }

    fn peek(&self) -> Option<char> {
        self.char_list.get(self.cursor).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.char_list.get(self.cursor + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let current_char = self.char_list.get(self.cursor).copied();
        if let Some(ch) = current_char {
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        current_char
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }
}

fn token_type_to_lexeme(token_type: &TokenType, char_list: &[char], cursor: usize) -> String {
    match token_type {
        TokenKind::Integer(n)    => n.to_string(),
        TokenKind::Float(f)      => f.to_string(),
        TokenKind::Identifier(s) => s.clone(),
        TokenKind::Let           => "let".to_string(),
        TokenKind::While         => "while".to_string(),
        TokenKind::For           => "for".to_string(),
        TokenKind::Func          => "fn".to_string(),
        TokenKind::If            => "if".to_string(),
        TokenKind::Else          => "else".to_string(),
        TokenKind::Return        => "return".to_string(),
        TokenKind::Plus          => "+".to_string(),
        TokenKind::Minus         => "-".to_string(),
        TokenKind::Star          => "*".to_string(),
        TokenKind::Slash         => "/".to_string(),
        TokenKind::Equal         => "=".to_string(),
        TokenKind::DoubleEqual   => "==".to_string(),
        TokenKind::Not           => "!".to_string(),
        TokenKind::NotEqual      => "!=".to_string(),
        TokenKind::Less          => "<".to_string(),
        TokenKind::Greater       => ">".to_string(),
        TokenKind::LeftParen     => "(".to_string(),
        TokenKind::RightParen    => ")".to_string(),
        TokenKind::LeftBrace     => "{".to_string(),
        TokenKind::RightBrace    => "}".to_string(),
        TokenKind::Semicolon     => ";".to_string(),
        TokenKind::Colon         => ":".to_string(),
        TokenKind::Comma         => ",".to_string(),
        TokenKind::EOF           => "\0".to_string(),
        TokenKind::Unknown(c)    => c.to_string(),
    }
}
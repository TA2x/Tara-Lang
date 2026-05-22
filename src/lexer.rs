use crate::{asm_helpers::{is_digit_asm, is_alpha_asm}, token::{Token, TokenType}};

pub struct Lexer<'a> {
    #[allow(dead_code)]
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

        let token_line = self.line;
        let token_column = self.column;

        let token_type = match self.peek(){
            None => TokenType::EOF,

            Some('"') => self.scan_string(),

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

        let _lexeme_start = self.cursor.saturating_sub(
            self.char_list[..self.cursor]
            .iter()
            .collect::<String>()
            .len()
        );
        let lexeme = token_type_to_lexeme(&token_type, &self.char_list, self.cursor);

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

    fn scan_string(&mut self) -> TokenType {
        self.advance(); // consume opening quote
        let start = self.cursor;
        
        while let Some(ch) = self.peek() {
            if ch == '"' {
                let string_content: String = self.char_list[start..self.cursor].iter().collect();
                self.advance(); // consume closing quote
                return TokenType::String(string_content);
            }
            self.advance();
        }
        
        // Unterminated string - return what we have as a string
        let string_content: String = self.char_list[start..self.cursor].iter().collect();
        TokenType::String(string_content)
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

fn token_type_to_lexeme(token_type: &TokenType, _char_list: &[char], _cursor: usize) -> String {
    match token_type {
        TokenType::Integer(n)    => n.to_string(),
        TokenType::Float(f)      => f.to_string(),
        TokenType::String(s)     => format!("\"{}\"", s),
        TokenType::Identifier(s) => s.clone(),
        TokenType::Let           => "let".to_string(),
        TokenType::While         => "while".to_string(),
        TokenType::For           => "for".to_string(),
        TokenType::Func          => "fn".to_string(),
        TokenType::If            => "if".to_string(),
        TokenType::Else          => "else".to_string(),
        TokenType::Return        => "return".to_string(),
        TokenType::Plus          => "+".to_string(),
        TokenType::Minus         => "-".to_string(),
        TokenType::Star          => "*".to_string(),
        TokenType::Slash         => "/".to_string(),
        TokenType::Equal         => "=".to_string(),
        TokenType::DoubleEqual   => "==".to_string(),
        TokenType::Not           => "!".to_string(),
        TokenType::NotEqual      => "!=".to_string(),
        TokenType::Less          => "<".to_string(),
        TokenType::Greater       => ">".to_string(),
        TokenType::LeftParen     => "(".to_string(),
        TokenType::RightParen    => ")".to_string(),
        TokenType::LeftBrace     => "{".to_string(),
        TokenType::RightBrace    => "}".to_string(),
        TokenType::Semicolon     => ";".to_string(),
        TokenType::Colon         => ":".to_string(),
        TokenType::Comma         => ",".to_string(),
        TokenType::LessEqual     => "<=".to_string(),
        TokenType::GreaterEqual  => ">=".to_string(),
        TokenType::EOF           => "\0".to_string(),
        TokenType::Unknown(c)    => c.to_string(),
    }
}
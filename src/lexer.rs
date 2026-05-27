use crate::token::{Token, TokenType::{self, Unknown}};
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

            Some('#') => {
                self.skip_line_comment();
                return self.next_token();
            }
            
            Some('/') => {
                self.advance();
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
            Some(ch) if ch.is_ascii_digit() => self.scan_number(),
            Some(ch) if ch.is_ascii_alphabetic() => self.scan_identifier_or_keyword(),
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
        
        while self.peek().map_or(false, |ch| ch.is_ascii_digit()){
            self.advance();
        }

        let is_float = self.peek() == Some('.') && self.peek_next().map_or(false, |ch| ch.is_ascii_digit());

        if is_float {
            self.advance(); // consume '.'
            while self.peek().map_or(false, |ch| ch.is_ascii_digit()){
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
        while self.peek().map_or(false, |ch| ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_') {
            self.advance();
        }

        let word: String = self.char_list[start..self.cursor].iter().collect();

        match word.as_str() {
            "make" => TokenType::Make,
            "show" => TokenType::Show,
            "when" => TokenType::When,
            "otherwise" => TokenType::Otherwise,
            "during" => TokenType::During,
            "for" => TokenType::For,
            "return" => TokenType::Return,
            "func" => TokenType::Func,
            "true" => TokenType::Boolean(true),
            "false" => TokenType::Boolean(false),
            _ => TokenType::Identifier(word),
        }
    }

    fn scan_string(&mut self) -> TokenType {
        self.advance(); // consume opening quote
        
        let mut string_content = String::new();
        
        while let Some(ch) = self.peek() {
            if ch == '"' {
                self.advance(); // consume closing quote
                return TokenType::String(string_content);
            }

            if ch == '\\' {
                self.advance();
                match self.peek() {
                    Some('n') => { self.advance(); string_content.push('\n');}
                    Some('t') => { self.advance(); string_content.push('\t'); }
                    Some('r') => { self.advance(); string_content.push('\r'); }
                    Some('\\') => { self.advance(); string_content.push('\\'); }
                    Some('"') => { self.advance(); string_content.push('"');  }

                    Some(unknown) => {
                        string_content.push('\\');
                        string_content.push(unknown);
                        self.advance();
                    }
                    None => break, // end of file after backslash
                }
            } else {
                string_content.push(ch);
                self.advance();
            }
        }
        
        // Unterminated string - return what we have as a string
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
        TokenType::Boolean(b)    => b.to_string(),
        TokenType::String(s)     => format!("\"{}\"", s),
        TokenType::Identifier(s) => s.clone(),
        TokenType::Make          => "make".to_string(),
        TokenType::Show          => "show".to_string(),
        TokenType::When          => "when".to_string(),
        TokenType::Otherwise     => "otherwise".to_string(),
        TokenType::During        => "during".to_string(),
        TokenType::For           => "for".to_string(),
        TokenType::Return        => "return".to_string(),
        TokenType::Func          => "func".to_string(),
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
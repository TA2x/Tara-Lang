use crate::token::{Token, TokenType};

pub struct Lexer<'a> {
    #[allow(dead_code)]
    source: &'a str,
    char_list: Vec<char>,
    cursor: usize,
    line: usize,
    column: usize,
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

        let token_line   = self.line;
        let token_column = self.column;

        let token_type = match self.peek() {
            None      => TokenType::EOF,
            Some('"') => self.scan_string(),

            Some('#') => {
                self.skip_line_comment();
                return self.next_token();
            }

            Some('/') => { self.advance(); TokenType::Slash }
            Some('+') => { self.advance(); TokenType::Plus }
            Some('-') => { self.advance(); TokenType::Minus }
            Some('*') => { self.advance(); TokenType::Star }
            Some('(') => { self.advance(); TokenType::LeftParen }
            Some(')') => { self.advance(); TokenType::RightParen }
            Some('{') => { self.advance(); TokenType::LeftBrace }
            Some('}') => { self.advance(); TokenType::RightBrace }
            Some(';') => { self.advance(); TokenType::Semicolon }
            Some(':') => { self.advance(); TokenType::Colon }
            Some(',') => { self.advance(); TokenType::Comma }

            Some('=') => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenType::DoubleEqual }
                else                        { TokenType::Equal }
            }
            Some('!') => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenType::NotEqual }
                else                        { TokenType::Not }
            }
            Some('<') => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenType::LessEqual }
                else                        { TokenType::Less }
            }
            Some('>') => {
                self.advance();
                if self.peek() == Some('=') { self.advance(); TokenType::GreaterEqual }
                else                        { TokenType::Greater }
            }

            Some(ch) if ch.is_ascii_digit()                    => self.scan_number(),
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_'  => self.scan_identifier_or_keyword(),
            Some(ch) => { self.advance(); TokenType::Unknown(ch) }
        };

        let lexeme = token_type_to_lexeme(&token_type);
        Token::new(token_type, lexeme, token_line, token_column)
    }

    fn scan_number(&mut self) -> TokenType {
        let start = self.cursor;

        while self.peek().map_or(false, |ch| ch.is_ascii_digit()) {
            self.advance();
        }

        let is_float = self.peek() == Some('.')
            && self.peek_next().map_or(false, |ch| ch.is_ascii_digit());

        if is_float {
            self.advance();
            while self.peek().map_or(false, |ch| ch.is_ascii_digit()) {
                self.advance();
            }
            let text: String = self.char_list[start..self.cursor].iter().collect();
            TokenType::Float(text.parse().unwrap_or(0.0))
        } else {
            let text: String = self.char_list[start..self.cursor].iter().collect();
            TokenType::Integer(text.parse().unwrap_or(0))
        }
    }

    fn scan_identifier_or_keyword(&mut self) -> TokenType {
        let start = self.cursor;
        while self.peek().map_or(false, |ch| ch.is_ascii_alphabetic() || ch.is_ascii_digit() || ch == '_') {
            self.advance();
        }

        let word: String = self.char_list[start..self.cursor].iter().collect();

        match word.as_str() {
            "make"      => TokenType::Make,
            "show"      => TokenType::Show,
            "when"      => TokenType::When,
            "otherwise" => TokenType::Otherwise,
            "during"    => TokenType::During,
            "for"       => TokenType::For,
            "return"    => TokenType::Return,
            "func"      => TokenType::Func,
            "true"      => TokenType::Boolean(true),
            "false"     => TokenType::Boolean(false),
            "int"       => TokenType::TypeInt,
            "float"     => TokenType::TypeFloat,
            "bool"      => TokenType::TypeBool,
            "String"    => TokenType::TypeString,
            "void"      => TokenType::TypeVoid,
            _           => TokenType::Identifier(word),
        }
    }

    fn scan_string(&mut self) -> TokenType {
        self.advance();

        let mut content = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    return TokenType::String(content);
                }
                '\\' => {
                    self.advance();
                    match self.peek() {
                        Some('n')  => { self.advance(); content.push('\n'); }
                        Some('t')  => { self.advance(); content.push('\t'); }
                        Some('r')  => { self.advance(); content.push('\r'); }
                        Some('\\') => { self.advance(); content.push('\\'); }
                        Some('"')  => { self.advance(); content.push('"');  }
                        Some(unknown) => { content.push('\\'); content.push(unknown); self.advance(); }
                        None => break,
                    }
                }
                _ => {
                    content.push(ch);
                    self.advance();
                }
            }
        }

        TokenType::String(content)
    }

    fn peek(&self) -> Option<char> {
        self.char_list.get(self.cursor).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.char_list.get(self.cursor + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.char_list.get(self.cursor).copied();
        if let Some(c) = ch {
            self.cursor += 1;
            if c == '\n' {
                self.line   += 1;
                self.column  = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while self.peek().map_or(false, |ch| ch.is_ascii_whitespace()) {
            self.advance();
        }
    }

    fn skip_line_comment(&mut self) {
        while self.peek().map_or(false, |ch| ch != '\n') {
            self.advance();
        }
    }
}

fn token_type_to_lexeme(token_type: &TokenType) -> String {
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
        TokenType::TypeInt       => "int".to_string(),
        TokenType::TypeFloat     => "float".to_string(),
        TokenType::TypeBool      => "bool".to_string(),
        TokenType::TypeString    => "String".to_string(),
        TokenType::TypeVoid      => "void".to_string(),
        TokenType::Plus          => "+".to_string(),
        TokenType::Minus         => "-".to_string(),
        TokenType::Star          => "*".to_string(),
        TokenType::Slash         => "/".to_string(),
        TokenType::Equal         => "=".to_string(),
        TokenType::DoubleEqual   => "==".to_string(),
        TokenType::Not           => "!".to_string(),
        TokenType::NotEqual      => "!=".to_string(),
        TokenType::Less          => "<".to_string(),
        TokenType::LessEqual     => "<=".to_string(),
        TokenType::Greater       => ">".to_string(),
        TokenType::GreaterEqual  => ">=".to_string(),
        TokenType::LeftParen     => "(".to_string(),
        TokenType::RightParen    => ")".to_string(),
        TokenType::LeftBrace     => "{".to_string(),
        TokenType::RightBrace    => "}".to_string(),
        TokenType::Semicolon     => ";".to_string(),
        TokenType::Colon         => ":".to_string(),
        TokenType::Comma         => ",".to_string(),
        TokenType::EOF           => "\0".to_string(),
        TokenType::Unknown(c)    => c.to_string(),
    }
}
mod token;
mod lexer;
mod asm_helpers;
mod ast;
mod parser;


use lexer::Lexer;
use token::TokenType;

fn main(){
    let source_code = r#"
        let x = 42;
        let y = x + 3.14;
        if (y > 50) {
            return "big";
        } else {
            return "small";
        }
    "#;

    let mut lexer = Lexer::new(source_code);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        let is_eof = token.token_type == TokenType::EOF;
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program().unwrap();
    println!("{:#?}", program);
}
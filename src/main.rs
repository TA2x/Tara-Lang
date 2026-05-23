mod token;
mod lexer;
mod asm_helpers;
mod ast;
mod parser;


use lexer::Lexer;
use token::TokenType;

fn main(){
    let source_code = r#"
        make x = 10
        make y = x + 5

        show("Hello Tara!")
        show(x + y)

        when (x > 5) {
            show("x is big")
        } otherwise {
            show("x is small")
        }

        make i = 0
        during (i < 3) {
            show(i)
            make i = i + 1
        }

        for (make j = 0; j < 5; j = j + 1) {
            show(j)
        }

        make total = (
            1 + 2 +
            3 + 4
        )
        show(total)
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
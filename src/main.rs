mod token;
mod lexer;
mod asm_helpers;

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
    let mut token_count = 0;

    loop {
        let token = lexer.next_token();

        println!(
            "[line {:2}, col {:2}]  {:?}",
            token.line,
            token.column,
            token.token_type
        );

        token_count += 1;

        if token.token_type == TokenType::EOF {
            break;
        }
    }

    println!("\n--- {} tokens produced ---", token_count);
}

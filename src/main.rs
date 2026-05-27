mod token;
mod lexer;
mod ast;
mod parser;
mod interpreter;

use std::env;
use std::fs;
use std::process;
use lexer::Lexer;
use token::TokenType;
use parser::Parser;
use interpreter::Interpreter;

fn main(){
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <file.tara>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];

    // Check if file has .tara extension
    if !filename.ends_with(".tara") {
        eprintln!("Error: File must have .tara extension");
        process::exit(1);
    }

    // Read the source code from file
    let source_code = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    // Tokenize
    let mut lexer = Lexer::new(&source_code);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        let is_eof = token.token_type == TokenType::EOF;
        tokens.push(token);
        if is_eof {
            break;
        }
    }

    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(prog) => prog,
        Err(err) => {
            eprintln!("Parse error: {}", err);
            process::exit(1);
        }
    };

    // Interpret
    let mut interpreter = Interpreter::new();
    interpreter.interpret(&program);
}
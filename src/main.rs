mod ast;
mod interpreter;
mod lexer;
mod parser;
mod token;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::{env, fs, process};
use token::TokenKind;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <file.tara>", args[0]);
        process::exit(1);
    }

    let path = &args[1];

    if !path.ends_with(".tara") {
        eprintln!("Error: file must have a .tara extension");
        process::exit(1);
    }

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error reading '{}': {}", path, err);
            process::exit(1);
        }
    };

    let tokens = {
        let mut lexer = Lexer::new(&source);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            let eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if eof {
                break;
            }
        }
        tokens
    };

    let program = match Parser::new(tokens).parse_program() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

    if let Err(err) = Interpreter::new().run(&program) {
        eprintln!("[RuntimeError] {}", err);
        process::exit(1);
    }
}

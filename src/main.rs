use std::{
    env,
    io::{Write, stdin, stdout},
    process::exit,
    sync::Mutex,
};

use interpreter::Interpreter;
use parser::Parser;
mod Scanner;
mod Tokentype;
mod environment;
mod expr;
mod interpreter;
mod parser;
mod stmt;
static ERROR: Mutex<bool> = Mutex::new(false);
fn main() {
    let arg: Vec<String> = env::args().collect();
    match arg.len().cmp(&2) {
        std::cmp::Ordering::Less => run_prompt(),
        std::cmp::Ordering::Greater => println!("using jlox script []"),
        std::cmp::Ordering::Equal => run_file(&arg[1]),
    }
}
pub fn run_file(source: &String) {
    let source = std::fs::read_to_string(source).unwrap();
    let mut interpreter = Interpreter::new();
    run(source, &mut interpreter);
    if *ERROR.lock().unwrap() {
        exit(64);
    }
}

pub fn run_prompt() {
    let mut interpreter = Interpreter::new();
    loop {
        print!(">>");
        stdout().flush().unwrap();

        let mut source = String::new();
        stdin().read_line(&mut source).unwrap();
        run(source, &mut interpreter);
        let mut lock = ERROR.lock().unwrap();
        *lock = false;
    }
}

pub fn run(source: String, interpreter: &mut Interpreter) {
    let mut scanner = Scanner::Scanner::new(source);
    scanner.scan_tokens();
    let mut parser = Parser::new(scanner.tokens);
    match parser.parse_stmt() {
        Ok(a) => {
            match interpreter.interpret(a) {
                Ok(_) => (),
                Err(e) => println!("{}", e),
            };
        }
        Err(e) => println!("{}", e),
    };
}

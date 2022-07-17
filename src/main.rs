use craftinginterpreters::{LoxError, Scanner, Token};
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    process,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => print!("Usage: rlox [script]"),
    }
}

fn run_file(path: impl AsRef<Path>) {
    let path_string = path.as_ref().to_string_lossy().to_string();
    let code = match fs::read_to_string(path) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("Unable to read file {}: {}", path_string, error);
            process::exit(74);
        }
    };
    if let Err(error) = run(code) {
        error_report(error);
        std::process::exit(65);
    }
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().expect("Unable to flush stdout");
        let mut line = String::new();
        let n = io::stdin()
            .read_line(&mut line)
            .expect("Unable to read line from prompt");
        if n == 0 {
            break;
        }
        if let Err(error) = run(line) {
            error_report(error);
            std::process::exit(65);
        };
    }
}

fn run(source: String) -> Result<(), LoxError> {
    let scanner = Scanner::new(&source);
    let tokens: Vec<Token> = scanner.scan_tokens()?;

    // for now, just print the tokens
    for token in tokens {
        println!("{token:?}");
    }
    Ok(())
}

//TODO: add error type name into error message
fn error_report(error: LoxError) {
    match error {
        LoxError::ParseError { line, message } => {
            eprintln!("[line {line}] Error: {message}")
        }
    }
}

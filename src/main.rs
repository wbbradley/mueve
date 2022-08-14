use std::env;
use std::fs;

use crate::lexer::Lexer;

mod error;
mod identifier;
mod lexer;
mod location;
mod parser;
mod token;
use crate::parser::{parse_decl, parse_many};

fn main() {
    let exit_code = run_compiler();
    std::process::exit(exit_code);
}

fn run_compiler() -> i32 {
    match run_real_compiler() {
        true => 0,
        false => 1,
    }
}

fn run_real_compiler() -> bool {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("mueve [filename.mv]");
        return false;
    }

    let filename: String = args[1].clone();
    let filename_slice = &filename[..];
    match fs::read_to_string(filename.clone()) {
        Ok(input) => {
            let input_slice: &str = &input[..];
            println!("parsing '{}'...", filename);
            compile(filename_slice, input_slice)
        }
        Err(err) => {
            eprintln!("Failed to read input ({filename})!\n{err}");
            false
        }
    }
}

fn compile<'a>(filename: &'a str, input: &'a str) -> bool {
    let lexer = Lexer::new(filename, input);
    match parse_many(parse_decl, lexer.advance()) {
        Ok((decls, _)) => {
            println!("Parsed {:?}", decls);
            true
        }
        Err(err) => {
            eprintln!("{}", err);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_some() {}
}

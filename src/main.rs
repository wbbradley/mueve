use paco::{
    all_of_input, concat, exact_string, lift, many, sequence, skip_whitespace, string_while,
    strip_whitespace, ParseState, Parser, Progress,
};

fn is_ident_starter(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn keyword<'a, T>(kw: T) -> impl Parser<char, String>
where
    T: Into<String>,
{
    skip_whitespace(exact_string(kw.into()))
}

#[derive(Debug)]
pub struct FnDecl {
    name: String,
}

fn ast_func(terms: Vec<String>) -> FnDecl {
    FnDecl {
        name: terms[1].clone(),
    }
}

fn main() {
    let ps = ParseState::new("fn __hello   \t fn   exit");

    let identifier = skip_whitespace(lift(
        concat,
        sequence!(string_while(is_ident_starter), string_while(is_ident_char)),
    ));

    // TODO: implement variadic lift.
    let func_def = lift(ast_func, sequence!(keyword("fn"), identifier));

    let ast_parser = all_of_input(strip_whitespace(many(func_def)));

    match ast_parser(ps) {
        Progress::Parsed(_, value) => {
            println!("{:?}", value);
        }
        Progress::Failed => {
            println!("Did not parse.")
        }
    }
}

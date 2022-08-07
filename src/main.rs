use paco::{
    all_of_input, bracket, choice, concat, exact_string, lift, many, sequence, skip_whitespace,
    string_while, strip_whitespace, ParseState, Parser, Progress,
};

fn is_ident_starter(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_var_ident_starter(ch: char) -> bool {
    (ch.is_ascii_alphabetic() && ch.is_lowercase()) || ch == '_'
}

fn is_ident_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn token<'a, T>(kw: T) -> impl Parser<char, String>
where
    T: Into<String>,
{
    skip_whitespace(exact_string(kw.into()))
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FnDecl {
    name: String,
}

#[allow(dead_code)]
enum Predicate {
    Irrefutable(Id),
    Integer(i64),
    String(String),
    Tuple { dims: Vec<Box<Predicate>> },
}

#[allow(dead_code)]
pub struct PatternExpr {
    predicate: Predicate,
    expr: Expr,
}

#[allow(dead_code)]
pub struct Id {
    name: String,
}

#[allow(dead_code)]
pub enum Expr {
    Lambda {
        param_names: Vec<Id>,
        body: Box<Expr>,
    },
    LiteralInteger {
        value: i64,
    },
    LiteralString {
        value: String,
    },
    Id {
        name: String,
    },
    Match {
        subject: Box<Expr>,
        pattern_exprs: Vec<PatternExpr>,
    },
    Callsite {
        function: Box<Expr>,
        arguments: Vec<Box<Expr>>,
    },
    TupleCtor {
        dims: Vec<Box<Expr>>,
    },
}

#[allow(dead_code)]
fn mkcallsite(_exprs: Vec<Expr>) -> Expr {
    Expr::Lambda {
        param_names: Vec::new(),
        body: Box::new(Expr::Id {
            name: "foo".to_string(),
        }),
    }
}

#[allow(dead_code)]
pub struct Decl {
    name: Id,
    pattern: Vec<Predicate>,
    body: Expr,
}

fn lift4ac<'a, A, B, C, D, V, F, PA, PB, PC, PD>(
    f: F,
    pa: PA,
    pb: PB,
    pc: PC,
    pd: PD,
) -> impl Parser<char, V>
where
    A: 'a,
    B: 'a,
    C: 'a,
    D: 'a,
    V: 'a,
    F: 'a + Fn(A, C) -> V,
    PA: Parser<char, A>,
    PB: Parser<char, B>,
    PC: Parser<char, C>,
    PD: Parser<char, D>,
{
    move |ps: ParseState<char>| -> Progress<char, V> {
        return match pa(ps) {
            Progress::Failed => Progress::Failed,
            Progress::Parsed(ps, a) => match pb(ps) {
                Progress::Failed => Progress::Failed,
                Progress::Parsed(ps, _) => match pc(ps) {
                    Progress::Failed => Progress::Failed,
                    Progress::Parsed(ps, c) => match pd(ps) {
                        Progress::Failed => Progress::Failed,
                        Progress::Parsed(ps, _) => return Progress::Parsed(ps, f(a, c)),
                    },
                },
            },
        };
    }
}

fn ast_func(predicate: Vec<String>, callsite: String) -> FnDecl {
    FnDecl {
        name: predicate.join(",") + " " + &callsite,
    }
}

fn main() {
    let ps = ParseState::new("predicate = callsite;\t\npredicate  =callsite ;");

    let ast_parser = get_ast_parser();

    match ast_parser(ps) {
        Progress::Parsed(_, value) => {
            println!("{:?}", value);
        }
        Progress::Failed => {
            println!("Did not parse.")
        }
    }
}

fn get_ast_parser() -> impl Parser<char, Vec<FnDecl>> {
    let identifier = skip_whitespace(lift(
        concat,
        sequence!(
            string_while(is_var_ident_starter),
            string_while(is_ident_char)
        ),
    ));
    let int = |s: String| -> i64 {
        let i = s.parse::<i64>().unwrap();
        i
    };
    /*
     *
    Irrefutable(Id),
    Integer(i64),
    String(String),
    Tuple { dims: Vec<Box<Predicate>> },
    */
    let tuple_predicate_parser = strip_whitespace(bracket(
        '(',
        ')',
        |ps: ParseState<char>| -> Progress<char, Vec<Box<Predicate>>> {
            Progress::Parsed(ps, Vec::new())
        },
    ));
    let predicate_parser = strip_whitespace(choice!(
        lift(Predicate::Irrefutable.Id, identifier),
        lift(Predicate::Integer, int),
    ));

    let callsite_parser = strip_whitespace(exact_string("callsite"));

    let def_parser = lift5abd(
        ast_func,
        identifier,
        predicate_parser,
        token("="),
        callsite_parser,
        token(";"),
    );

    return all_of_input(many(strip_whitespace(def_parser)));
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! test_parse_ok {
        ($input: expr, $parser: expr) => {
            let ps: ParseState<_> = ParseState::new($input);
            let language_parser = $parser;
            match language_parser(ps) {
                Progress::Parsed(_, _) => (),
                Progress::Failed => assert!(false),
            }
        };
    }
    macro_rules! test_parse_fail {
        ($input: expr, $parser: expr) => {
            let ps: ParseState<_> = ParseState::new($input);
            let language_parser = $parser;
            match language_parser(ps) {
                Progress::Parsed(_, _) => assert!(false),
                Progress::Failed => (),
            }
        };
    }

    #[test]
    fn parse_ast() {
        test_parse_ok!(
            "predicate = callsite;\t\npredicate  =callsite ;",
            get_ast_parser()
        );
        test_parse_fail!(
            "predicate  callsite;\t\npredicate  =callsite ;",
            get_ast_parser()
        );
    }
}

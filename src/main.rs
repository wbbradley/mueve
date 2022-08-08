mod lexer;
use lexer::Lexer;

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

fn main() {
    let input = "123454 14 \n pi.(} &";
    let lexer = Lexer::new("raw-text", &input);
    for token in lexer {
        println!("{:?}", token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_some() {}
}

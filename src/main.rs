use paco::{digits, ParseState, Progress};

fn main() {
    let ps = ParseState::new("Hello, world!");
    match digits(ps) {
        Progress::Parsed(_, num) => {
            println!("{}", num);
        }
        Progress::Failed => {
            println!("Did not parse.")
        }
    }
}

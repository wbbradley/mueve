use paco::{digits, ParseState, Progress};

fn main() {
    let ps = ParseState::new(&"14234".to_string());
    match digits(ps) {
        Progress::Parsed(_, num) => {
            println!("{}", num);
        }
        Progress::Failed => {
            println!("Did not parse.")
        }
    }
}

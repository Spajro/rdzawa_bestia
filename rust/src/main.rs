use std::io;
use shakmaty::Chess;
use crate::engine::RandomEngine;

mod engine;
mod uci;

fn main() {
    let mut input = String::new();
    let mut engine = RandomEngine {
        pos: Chess::default()
    };
    let stdin = io::stdin();
    loop {
        stdin.read_line(&mut input).expect("panic message");
        if input.ends_with('\n') {
            input.pop();
        }
        if input.ends_with('\r') {
            input.pop();
        }
        let result = uci::handle_uci(&input, &mut engine);
        if result.is_some() {
            println!("{}", result.unwrap());
        }
        input.clear()
    }
}

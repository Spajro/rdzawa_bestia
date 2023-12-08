use std::io;
use shakmaty::Chess;
use crate::minmax_engine::MinMaxEngine;
use crate::output::send;

mod engine;
mod uci;
mod output;
mod minmax_engine;
mod evaluation;
mod time_management;

fn main() {
    let mut input = String::new();
    let mut engine = MinMaxEngine {
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
            send(result.unwrap())
        }
        input.clear()
    }
}

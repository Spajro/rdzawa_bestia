use crate::minmax_engine::MinMaxEngine;
use crate::output::send;
use shakmaty::Chess;
use std::io;

mod engine;
mod evaluation;
mod minmax_engine;
mod output;
mod time_management;
mod uci;
mod quiesence;

fn main() {
    let mut input = String::new();
    let mut engine = MinMaxEngine::new(Chess::default());
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

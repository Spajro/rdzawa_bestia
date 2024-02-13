use crate::minmax_engine::MinMaxEngine;
use crate::io::output::send;
use shakmaty::Chess;
use crate::io::uci;

mod engine;
mod minmax_engine;

mod features;
mod io;

fn main() {
    let mut input = String::new();
    let mut engine = MinMaxEngine::new(Chess::default());
    let stdin = std::io::stdin();
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

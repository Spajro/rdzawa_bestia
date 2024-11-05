use chess::Board;
use chess::Color::White;

use crate::io::output::send;
use crate::io::uci;
use crate::io::uci::State;
use crate::minmax_engine::MinMaxEngine;

mod engine;
mod minmax_engine;

mod features;
mod io;
mod random_engine;

fn main() {
    let mut input = String::new();
    let mut state = State{
        engine: Box::new(MinMaxEngine::new(Board::default())),
        next_color: White,
    };
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut input).expect("panic message");
        if input.ends_with('\n') {
            input.pop();
        }
        if input.ends_with('\r') {
            input.pop();
        }
        let result = uci::handle_uci(&input, &mut state);
        if result.msg.is_some() {
            send(result.msg.unwrap())
        }
        input.clear()
    }
}

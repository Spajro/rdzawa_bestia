use crate::minmax_engine::MinMaxEngine;
use crate::io::output::send;
use chess::Board;
use chess::Color::White;
use crate::io::uci;

mod engine;
mod minmax_engine;

mod features;
mod io;
mod random_engine;

fn main() {
    let mut input = String::new();
    let mut engine = MinMaxEngine::new(Board::default());
    let mut next_color = White;
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut input).expect("panic message");
        if input.ends_with('\n') {
            input.pop();
        }
        if input.ends_with('\r') {
            input.pop();
        }
        let result = uci::handle_uci(&input, &mut engine, next_color);
        if result.msg.is_some() {
            send(result.msg.unwrap())
        }
        next_color = result.next_color;
        input.clear()
    }
}

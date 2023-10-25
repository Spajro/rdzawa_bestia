use shakmaty::Move;
use uci::move_to_uci;
use crate::uci;

pub fn send(string: String) {
    println!("{}", string);
}

pub fn send_move(mv: Move) {
    send(move_to_uci(mv));
}
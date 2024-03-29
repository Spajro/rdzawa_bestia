use chess::ChessMove;
use uci::move_to_uci;
use crate::uci;

pub fn send(string: String) {
    println!("{}", string);
}

pub fn send_move(mv: ChessMove) {
    send(move_to_uci(mv));
}

pub fn send_info(info: String) {
    send(String::from("info ") + &*info)
}
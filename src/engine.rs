use chess::ChessMove;
use crate::io::uci::Fen;

pub trait Engine {
    fn start(&mut self, time: u64);
    fn stop(&mut self);
    fn update(&mut self, fen: Fen, moves: Vec<ChessMove>);
    fn restart(&mut self);
}
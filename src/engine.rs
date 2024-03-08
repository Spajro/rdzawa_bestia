use chess::ChessMove;
use crate::io::uci::Position;

pub trait Engine {
    fn start(&mut self, time: u64);
    fn stop(&mut self);
    fn update(&mut self, fen: Position, moves: Vec<ChessMove>);
    fn restart(&mut self);
}
use chess::{Board, ChessMove};

pub trait Engine {
    fn start(&mut self, time: u64);
    fn stop(&mut self);
    fn update(&mut self, mv: ChessMove);
    fn restart(&mut self);
    fn get_status(&self) -> Board;
}
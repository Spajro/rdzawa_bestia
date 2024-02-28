use chess::ChessMove;

pub trait Engine {
    fn start(&mut self, time: u64);
    fn stop(&mut self);
    fn update(&mut self, mv: ChessMove);
    fn restart(&mut self);
}
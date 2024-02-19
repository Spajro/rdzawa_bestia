use crate::io::output::send_move;
use chess::{Board, ChessMove, MoveGen};
use rand::seq::SliceRandom;

pub trait Engine {
    fn start(&mut self, time: u64);
    fn stop(&mut self);
    fn update(&mut self, mv: ChessMove);
    fn restart(&mut self);
    fn get_status(&self) -> Board;
}

pub struct RandomEngine {
    pub pos: Board,
}

impl Engine for RandomEngine {
    fn start(&mut self, _time: u64) {
        send_move(self.next_move())
    }

    fn stop(&mut self) {
        send_move(self.next_move())
    }

    fn update(&mut self, mv: ChessMove) {
        self.pos = self.pos.make_move_new(mv);
    }

    fn restart(&mut self) {
        self.pos = Board::default();
    }

    fn get_status(&self) -> Board {
        self.pos.clone()
    }
}

impl RandomEngine {
    fn next_move(&mut self) -> ChessMove {
        let moves = MoveGen::new_legal(&self.pos)
            .into_iter()
            .collect::<Vec<ChessMove>>();

        let mv = moves.choose(&mut rand::thread_rng()).unwrap();
        self.pos = self.pos.make_move_new(*mv);
        mv.clone()
    }
}
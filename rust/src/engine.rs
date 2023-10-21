use shakmaty::{Chess, Move, Position};
use rand::seq::SliceRandom;

pub trait Engine {
    fn start(&self);
    fn stop(&mut self) -> Move;
    fn update(&mut self, mv: Move);
    fn restart(&mut self);
    fn get_status(&self) -> Chess;
}

pub struct RandomEngine {
    pub pos: Chess,
}

impl Engine for RandomEngine {
    fn start(&self) {}

    fn stop(&mut self) -> Move {
        let moves = self.pos
            .legal_moves();
        let mv = moves
            .choose(&mut rand::thread_rng())
            .unwrap();
        self.pos.play_unchecked(mv);
        mv.clone()
    }

    fn update(&mut self, mv: Move) {
        self.pos.play_unchecked(&mv);
    }

    fn restart(&mut self) {
        self.pos = Chess::default();
    }

    fn get_status(&self) -> Chess {
        self.pos.clone()
    }
}

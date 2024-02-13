use shakmaty::{Chess, Move, Position};
use rand::seq::SliceRandom;
use crate::io::output::send_move;

pub trait Engine {
    fn start(&mut self, time: u64);
    fn stop(&mut self);
    fn update(&mut self, mv: Move);
    fn restart(&mut self);
    fn get_status(&self) -> Chess;
}

pub struct RandomEngine {
    pub pos: Chess,
}

impl Engine for RandomEngine {
    fn start(&mut self, _time: u64) {
        send_move(self.next_move())
    }

    fn stop(&mut self) {
        send_move(self.next_move())
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

impl RandomEngine {
    fn next_move(&mut self) -> Move {
        let moves = self.pos
            .legal_moves();
        let mv = moves
            .choose(&mut rand::thread_rng())
            .unwrap();
        self.pos.play_unchecked(mv);
        mv.clone()
    }
}

use std::f32::consts::SQRT_2;

use arrayvec::ArrayVec;
use chess::{ChessMove, Square};

#[derive(Clone)]
pub struct KillerMoves<const MAX_SIZE: usize> {
    pub(crate) moves: ArrayVec<ChessMove, MAX_SIZE>,
    pub(crate) size: usize,
}

impl<const MAX_SIZE: usize> KillerMoves<MAX_SIZE> {
    pub fn new() -> Self {
        let mut moves = ArrayVec::<_, { MAX_SIZE }>::new();

        (0..MAX_SIZE)
            .into_iter()
            .map(|_| ChessMove::new(Square::A1, Square::A1, None))
            .for_each(|put| moves.push(put));

        KillerMoves::<MAX_SIZE> {
            moves: moves,
            size: 0,
        }
    }
    pub fn add(&mut self, mv: ChessMove) {
        self.moves.rotate_right(1);
        self.size = std::cmp::min(self.size + 1, MAX_SIZE);
        self.moves[0] = mv;
    }
}

use arrayvec::ArrayVec;
use shakmaty::Move;

#[derive(Clone)]
pub struct KillerMoves<const MAX_SIZE: usize> {
    pub(crate) moves: ArrayVec<Move, MAX_SIZE>,
    pub(crate) size: usize,
}

impl<const MAX_SIZE: usize> KillerMoves<MAX_SIZE> {
    pub fn new() -> Self {
        let mut moves = ArrayVec::<_, { MAX_SIZE }>::new();

        (0..MAX_SIZE).into_iter().map(|_| Move::Put {
            role: shakmaty::Role::Pawn,
            to: shakmaty::Square::E5,
        }).for_each(|put| moves.push(put));

        KillerMoves::<MAX_SIZE> {
            moves: moves,
            size: 0,
        }
    }
    pub fn add(&mut self, mv: Move) {
        self.moves.rotate_right(1);
        self.size = std::cmp::min(self.size + 1, MAX_SIZE);
        self.moves[0] = mv;
    }
}
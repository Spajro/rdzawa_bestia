use shakmaty::{Chess, Position};

pub fn eval(chess: &Chess) -> f32 {
    let board = chess.board();
    let white_value = board.white().intersect(board.pawns()).count() +
        3 * board.white().intersect(board.knights()).count() +
        3 * board.white().intersect(board.bishops()).count() +
        5 * board.white().intersect(board.rooks()).count() +
        9 * board.white().intersect(board.queens()).count();
    let black_value = board.black().intersect(board.pawns()).count() +
        3 * board.black().intersect(board.knights()).count() +
        3 * board.black().intersect(board.bishops()).count() +
        5 * board.black().intersect(board.rooks()).count() +
        9 * board.black().intersect(board.queens()).count();
    (white_value - black_value) as f32
}


#[cfg(test)]
mod eval_tests {
    use shakmaty::{Chess, Move, Position, Role, Square};
    use crate::evaluation::eval;

    #[test]
    fn start_board_test() {
        let chess = Chess::new();
        assert_eq!(0.0, eval(&chess))
    }

    #[test]
    fn board_after_taking_pawn_test() {
        let chess0 = Chess::new();
        let chess1 = chess0.play(&Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        }).unwrap();
        let chess2 = chess1.play(&Move::Normal {
            role: Role::Pawn,
            from: Square::D7,
            capture: None,
            to: Square::D5,
            promotion: None,
        }).unwrap();
        let chess3 = chess2.play(&Move::Normal {
            role: Role::Pawn,
            from: Square::E4,
            capture: Option::from(Role::Pawn),
            to: Square::D5,
            promotion: None,
        }).unwrap();
        assert_eq!(1.0, eval(&chess3))
    }

    #[test]
    fn board_after_taking_2_pawns_test() {
        let chess0 = Chess::new();
        let chess1 = chess0.play(&Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        }).unwrap();
        let chess2 = chess1.play(&Move::Normal {
            role: Role::Pawn,
            from: Square::D7,
            capture: None,
            to: Square::D5,
            promotion: None,
        }).unwrap();
        let chess3 = chess2.play(&Move::Normal {
            role: Role::Pawn,
            from: Square::E4,
            capture: Option::from(Role::Pawn),
            to: Square::D5,
            promotion: None,
        }).unwrap();
        let chess4 = chess3.play(&Move::Normal {
            role: Role::Queen,
            from: Square::D8,
            capture: Option::from(Role::Pawn),
            to: Square::D5,
            promotion: None,
        }).unwrap();
        assert_eq!(0.0, eval(&chess4))
    }
}
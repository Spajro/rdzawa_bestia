use shakmaty::{Chess, Position, Square, File, Rank, Board, Color};

pub const KING_SQUARE_TABLE: [i32; 64] =
    [
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -60, -60, -60, -60, -60, -60, -60, -60,
        3, 1, -5, -15, -15, -5, 1, 3,
        5, 20, 10, 0, 5, 0, 40, 20,
    ];

pub const QUEEN_SQUARE_TABLE: [i32; 64] =
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

pub const ROOK_SQUARE_TABLE: [i32; 64] =
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

pub const BISHOP_SQUARE_TABLE: [i32; 64] =
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

pub const KNIGHT_SQUARE_TABLE: [i32; 64] =
    [
        -60, -60, -30, -30, -30, -30, -60, -60,
        -40, -40, -20, -20, -20, -20, -40, -40,
        -20, 60, 40, 60, 60, 100, 50, 20,
        -5, 10, 10, 40, 20, 20, 10, 10,
        -10, 5, 10, 10, 20, 10, 20, -8,
        -15, -10, 12, 10, 10, 12, -10, -15,
        -20, -50, -10, 0, 0, -10, -50, -20,
        -100, -20, -30, -40, -40, -30, -20, -100,
    ];

pub const PAWN_SQUARE_TABLE: [i32; 64] =
    [
        0, 0, 0, 0, 0, 0, 0, 0,
        20, 25, 30, 35, 35, 30, 25, 20,
        15, 20, 25, 30, 30, 25, 20, 15,
        10, 15, 20, 25, 25, 20, 15, 10,
        5, 10, 15, 30, 30, 15, 10, 5,
        3, 5, 10, 15, 15, 10, 5, 3,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];


pub fn get_sq_val(sq: Square, square_table: [i32; 64], color: Color) -> i32 {
    let index = sq.file().distance(File::H) + 8 * sq.rank().distance(match color {
        Color::White => Rank::Eighth,
        Color::Black => Rank::First
    });
    square_table[index as usize]
}

pub fn get_position_cumulative_value(board: &Board, color: Color) -> f32 {
    let color_board = board.by_color(color);
    let king_pos_val = color_board.intersect(board.kings())
        .into_iter()
        .map(|sq| get_sq_val(sq, KING_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let queen_pos_val = color_board.intersect(board.queens())
        .into_iter()
        .map(|sq| get_sq_val(sq, QUEEN_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let rooks_pos_val = color_board.intersect(board.rooks())
        .into_iter()
        .map(|sq| get_sq_val(sq, ROOK_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let bishops_pos_val = color_board.intersect(board.bishops())
        .into_iter()
        .map(|sq| get_sq_val(sq, BISHOP_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let knights_pos_val = color_board.intersect(board.knights())
        .into_iter()
        .map(|sq| get_sq_val(sq, KNIGHT_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let pawns_pos_val = color_board.intersect(board.pawns())
        .into_iter()
        .map(|sq| get_sq_val(sq, PAWN_SQUARE_TABLE, color))
        .sum::<i32>() as f32;

    (king_pos_val + queen_pos_val + rooks_pos_val + bishops_pos_val + knights_pos_val + pawns_pos_val) / 100.0
}

pub fn eval(chess: &Chess) -> f32 {
    let board = chess.board();
    if chess.is_game_over() {
        return if chess.is_checkmate() {
            if chess.turn().is_white() {
                -1e9
            } else {
                1e9
            }
        } else {
            0.0
        };
    }

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
    white_value as f32 - black_value as f32 + get_position_cumulative_value(board, Color::White) - get_position_cumulative_value(board, Color::Black)
}


#[cfg(test)]
mod eval_tests {
    use shakmaty::{Chess, Move, Position, Role, Square};
    use crate::evaluation::{eval};

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
        assert_eq!(1.25, eval(&chess3))
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
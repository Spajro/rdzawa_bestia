use chess::{BitBoard, Board, BoardStatus, Color, File, Piece, Rank, Square, EMPTY};

use crate::io::output::send_info;

#[rustfmt::skip]
pub const KING_SQUARE_TABLE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30, 
    -30,-40,-40,-50,-50,-40,-40,-30, 
    -30,-40,-40,-50,-50,-40,-40,-30, 
    -30,-40,-40,-50,-50,-40,-40,-30, 
    -20,-30,-30,-40,-40,-30,-30,-20, 
    -10,-20,-20,-20,-20,-20,-20,-10, 
     20, 20,  0,  0,  0,  0, 20, 20, 
     20, 30, 10,  0,  0, 10, 30, 20,
];

#[rustfmt::skip]
pub const QUEEN_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
pub const ROOK_SQUARE_TABLE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
pub const BISHOP_SQUARE_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
pub const KNIGHT_SQUARE_TABLE: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  10,  15,  15,  10,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
pub const PAWN_SQUARE_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0, 
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

fn file_distance(a: File, b: File) -> i32 {
    i32::abs(a.to_index() as i32 - b.to_index() as i32)
}

fn rank_distance(a: Rank, b: Rank) -> i32 {
    i32::abs(a.to_index() as i32 - b.to_index() as i32)
}

pub fn get_sq_val(sq: Square, square_table: [i32; 64], color: Color) -> i32 {
    let index = file_distance(sq.get_file(), File::H)
        + 8 * rank_distance(
            sq.get_rank(),
            match color {
                Color::White => Rank::Eighth,
                Color::Black => Rank::First,
            },
        );
    square_table[index as usize]
}

fn get_pieces_value(board: &Board, board_side: &BitBoard) -> u32 {
    100 * (board.pieces(Piece::Pawn) & board_side).popcnt()
        + 320 * (board.pieces(Piece::Knight) & board_side).popcnt()
        + 330 * (board.pieces(Piece::Bishop) & board_side).popcnt()
        + 500 * (board.pieces(Piece::Rook) & board_side).popcnt()
        + 900 * (board.pieces(Piece::Queen) & board_side).popcnt()
}

pub fn get_position_cumulative_value(board: &Board, color: Color) -> f32 {
    let color_board = board.color_combined(color);
    let king_pos_val = (color_board & board.pieces(Piece::King))
        .into_iter()
        .map(|sq| get_sq_val(sq, KING_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let queen_pos_val = (color_board & board.pieces(Piece::Queen))
        .into_iter()
        .map(|sq| get_sq_val(sq, QUEEN_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let rooks_pos_val = (color_board & board.pieces(Piece::Rook))
        .into_iter()
        .map(|sq| get_sq_val(sq, ROOK_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let bishops_pos_val = (color_board & board.pieces(Piece::Bishop))
        .into_iter()
        .map(|sq| get_sq_val(sq, BISHOP_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let knights_pos_val = (color_board & board.pieces(Piece::Knight))
        .into_iter()
        .map(|sq| get_sq_val(sq, KNIGHT_SQUARE_TABLE, color))
        .sum::<i32>() as f32;
    let pawns_pos_val = (color_board & board.pieces(Piece::Pawn))
        .into_iter()
        .map(|sq| get_sq_val(sq, PAWN_SQUARE_TABLE, color))
        .sum::<i32>() as f32;

    king_pos_val + queen_pos_val + rooks_pos_val + bishops_pos_val + knights_pos_val + pawns_pos_val
}

#[inline]
pub fn status(board: &Board, any_legal_move: bool) -> BoardStatus {
    match any_legal_move {
        false => {
            if *board.checkers() == EMPTY {
                BoardStatus::Stalemate
            } else {
                BoardStatus::Checkmate
            }
        }
        true => BoardStatus::Ongoing,
    }
}

pub fn eval(board: &Board, any_legal_move: bool) -> f32 {
    // match board.status() {
    match status(board, any_legal_move) {
        BoardStatus::Checkmate => {
            if board.side_to_move() == Color::White {
                -1e9 as f32
            } else {
                1e9 as f32
            }
        }

        BoardStatus::Stalemate => 0.0,

        BoardStatus::Ongoing => {
            let white_value = get_pieces_value(board, board.color_combined(Color::White));
            let black_value = get_pieces_value(board, board.color_combined(Color::Black));
            white_value as f32 - black_value as f32
                + get_position_cumulative_value(board, Color::White)
                - get_position_cumulative_value(board, Color::Black)
        }
    }
}

#[cfg(test)]
mod eval_tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn mate_in_two() {
        // https://www.chess.com/forum/view/more-puzzles/hardest-mate-in-1-puzzles
        let board =
            Board::from_str("r1b2b1r/pp3Qp1/2nkn2p/3ppP1p/P1p5/1NP1NB2/1PP1PPR1/1K1R3q w - - 0 1")
                .unwrap();
        // println!("board: {:?}", board);
        assert_eq!(eval(&board, true), -400.0)
    }
}

// #[cfg(test)]
// mod eval_tests {
//     use crate::evaluation::eval;
//     use shakmaty::{Chess, Move, Position, Role, Square};

//     #[test]
//     fn start_board_test() {
//         let chess = Chess::new();
//         assert_eq!(0.0, eval(&chess, false))
//     }

//     #[test]
//     fn board_after_taking_pawn_test() {
//         let chess0 = Chess::new();
//         let chess1 = chess0
//             .play(&Move::Normal {
//                 role: Role::Pawn,
//                 from: Square::E2,
//                 capture: None,
//                 to: Square::E4,
//                 promotion: None,
//             })
//             .unwrap();
//         let chess2 = chess1
//             .play(&Move::Normal {
//                 role: Role::Pawn,
//                 from: Square::D7,
//                 capture: None,
//                 to: Square::D5,
//                 promotion: None,
//             })
//             .unwrap();
//         let chess3 = chess2
//             .play(&Move::Normal {
//                 role: Role::Pawn,
//                 from: Square::E4,
//                 capture: Option::from(Role::Pawn),
//                 to: Square::D5,
//                 promotion: None,
//             })
//             .unwrap();
//         assert_eq!(100.25, eval(&chess3, false))
//     }

//     #[test]
//     fn board_after_taking_2_pawns_test() {
//         let chess0 = Chess::new();
//         let chess1 = chess0
//             .play(&Move::Normal {
//                 role: Role::Pawn,
//                 from: Square::E2,
//                 capture: None,
//                 to: Square::E4,
//                 promotion: None,
//             })
//             .unwrap();
//         let chess2 = chess1
//             .play(&Move::Normal {
//                 role: Role::Pawn,
//                 from: Square::D7,
//                 capture: None,
//                 to: Square::D5,
//                 promotion: None,
//             })
//             .unwrap();
//         let chess3 = chess2
//             .play(&Move::Normal {
//                 role: Role::Pawn,
//                 from: Square::E4,
//                 capture: Option::from(Role::Pawn),
//                 to: Square::D5,
//                 promotion: None,
//             })
//             .unwrap();
//         let chess4 = chess3
//             .play(&Move::Normal {
//                 role: Role::Queen,
//                 from: Square::D8,
//                 capture: Option::from(Role::Pawn),
//                 to: Square::D5,
//                 promotion: None,
//             })
//             .unwrap();
//         assert_eq!(0.0, eval(&chess4, false))
//     }
// }

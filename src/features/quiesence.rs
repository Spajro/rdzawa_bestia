use crate::features::evaluation::{eval, status};
use crate::minmax_engine::{MinMaxEngine, Result};
use arrayvec::ArrayVec;
use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen, Piece, EMPTY};
use rand::seq::SliceRandom;
use std::time::Instant;

use super::evaluation::is_insufficient_material;

pub fn quiescence(
    mut engine: &mut MinMaxEngine,
    pos: Board,
    qdepth: usize,
    mut alpha: f32,
    beta: f32,
    end_time: Instant,
) -> Result {
    if (engine.evaluations_cnt & 511) == 0 && end_time <= Instant::now() {
        return Result {
            score: alpha,
            chosen_move: None,
            computed: false,
        };
    }

    let mut moves_generator = MoveGen::new_legal(&pos);
    let any_legal_move = moves_generator.size_hint().0 > 0;
    if *pos.checkers() == EMPTY {
        // TODO: no en-passant check here
        moves_generator.set_iterator_mask(*pos.color_combined(!pos.side_to_move()));
    }

    engine.evaluations_cnt += 1;
    let insufficient_material = is_insufficient_material(&pos);

    let board_status = status(&pos, any_legal_move, insufficient_material);

    let stand_pat = if pos.side_to_move() == Color::White {
        eval(&pos, board_status)
    } else {
        -eval(&pos, board_status)
    };

    if stand_pat >= beta {
        return Result {
            score: stand_pat,
            chosen_move: None,
            computed: true,
        };
    }

    if alpha < stand_pat {
        alpha = stand_pat;
    }

    if board_status != BoardStatus::Ongoing || qdepth == 0 {
        return Result {
            score: stand_pat,
            chosen_move: None,
            computed: true,
        };
    }

    let mut move_order = moves_generator
        .into_iter()
        .map(|mv| {
            return match pos.piece_on(mv.get_source()) {
                Some(Piece::Pawn) => (1, mv),
                Some(Piece::Knight) => (3, mv),
                Some(Piece::Bishop) => (4, mv),
                Some(Piece::Rook) => (5, mv),
                Some(Piece::Queen) => (9, mv),
                Some(Piece::King) => (10, mv),
                None => (0, mv),
            };
        })
        .collect::<Vec<(i16, ChessMove)>>();

    if move_order.is_empty() {
        return Result {
            score: stand_pat,
            chosen_move: None,
            computed: true,
        };
    }

    move_order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut new_pos = pos.clone();
    for (_, next_move) in move_order {
        pos.make_move(next_move, &mut new_pos);

        let mut result: Result =
            quiescence(&mut engine, new_pos, qdepth - 1, -beta, -alpha, end_time);
        result.score = -result.score;

        if result.computed == false {
            return Result {
                score: alpha,
                chosen_move: None,
                computed: false,
            };
        }

        if result.score >= beta {
            return Result {
                score: beta,
                chosen_move: None,
                computed: true,
            };
        }

        if result.score > alpha {
            alpha = result.score;
        }
    }

    return Result {
        score: alpha,
        chosen_move: None,
        computed: true,
    };
}

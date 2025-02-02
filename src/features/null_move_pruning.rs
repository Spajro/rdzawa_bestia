use crate::minmax_engine::{MinMaxEngine, Result};
use chess::{Board, ChessMove, Piece, EMPTY};
use std::time::Instant;

const NULL_MOVE_DEPTH_REDUCTION: usize = 3;

pub struct NullMoveResult {
    pub(crate) prunned: bool,
    pub(crate) chosen_move: Option<ChessMove>,
}

pub fn null_move(
    engine: &mut MinMaxEngine,
    pos: Board,
    depth: usize,
    qdepth: usize,
    total_depth: usize,
    beta: i32,
    end_time: Instant,
    is_last_null_move: bool,
) -> NullMoveResult {
    if !can_apply_null_move(depth, total_depth, is_last_null_move, pos) {
        return NullMoveResult {
            prunned: false,
            chosen_move: None,
        };
    }

    let new_pos = pos.null_move();
    
    let mut result: Result = engine.negamax(
        new_pos.unwrap(),
        depth - NULL_MOVE_DEPTH_REDUCTION,
        qdepth,
        total_depth + 1,
        -beta,
        -(beta - 1),
        end_time,
        true,
    );
    result.score = -result.score;

    if result.computed == false {
        return NullMoveResult {
            prunned: false,
            chosen_move: None,
        };
    }

    return NullMoveResult {
        prunned: result.score >= beta,
        chosen_move: result.chosen_move,
    };
}

pub fn can_apply_null_move(
    depth: usize,
    total_depth: usize,
    is_last_null_move: bool,
    pos: Board,
) -> bool {
    if is_last_null_move {
        return false;
    }
    // TODO: later add check for null window
    if depth <= NULL_MOVE_DEPTH_REDUCTION || total_depth == 0 {
        return false;
    }
    // cannot do null move if there is a check
    if *pos.checkers() != EMPTY {
        return false;
    }
    // cannot do null move if there is not enough material
    let has_non_pawn_material = (pos.color_combined(pos.side_to_move())
        ^ pos.pieces(Piece::King)
        ^ pos.pieces(Piece::Pawn))
    .popcnt() > 0;
    if !has_non_pawn_material {
        return false;
    }
    return true;
}

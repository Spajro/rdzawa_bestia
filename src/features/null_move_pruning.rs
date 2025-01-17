use crate::minmax_engine::{MinMaxEngine, Result};
use chess::{Board, Piece};
use std::time::Instant;

const NULL_MOVE_DEPTH_REDUCTION: usize = 3;

pub fn null_move(
    engine: &mut MinMaxEngine,
    pos: Board,
    depth: usize,
    qdepth: usize,
    total_depth: usize,
    alpha: f32,
    beta: f32,
    end_time: Instant,
) -> bool {
    if (engine.evaluations_cnt & 511) == 0 && end_time <= Instant::now()
        || !can_apply_null_move(depth, pos)
    {
        return false;
    }

    let new_pos = pos.null_move();

    let mut result: Result = engine.negamax(
        new_pos.unwrap(),
        depth - 1,
        qdepth,
        total_depth + 1,
        -beta,
        -alpha,
        end_time,
    );
    result.score = -result.score;

    if result.computed == false {
        return false;
    }

    return result.score >= beta;
}

pub fn can_apply_null_move(depth: usize, pos: Board) -> bool {
    let non_pawns_on_board = (pos.color_combined(pos.side_to_move())
        ^ pos.pieces(Piece::King)
        ^ pos.pieces(Piece::Pawn))
    .popcnt();
    return depth > NULL_MOVE_DEPTH_REDUCTION && non_pawns_on_board > 0 && pos.checkers().popcnt() == 0;
}

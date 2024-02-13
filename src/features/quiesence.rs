use std::time::Instant;
use arrayvec::ArrayVec;
use shakmaty::{Chess, Move, MoveList, Position, Role};
use crate::features::evaluation::eval;
use crate::minmax_engine::{MinMaxEngine, Result};

pub fn quiescence(
    mut engine: &mut MinMaxEngine,
    pos: Chess,
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

    let mut legal_moves: MoveList = pos.legal_moves();
    engine.evaluations_cnt += 1;
    let stand_pat = if pos.turn().is_white() {
        eval(&pos, &legal_moves, false)
    } else {
        -eval(&pos, &legal_moves, false)
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

    // if pos.outcome().is_some() || qdepth == 0 {
    if pos.is_variant_end()
        || legal_moves.is_empty()
        || pos.is_insufficient_material()
        || qdepth == 0
    {
        return Result {
            score: stand_pat,
            chosen_move: None,
            computed: true,
        };
    }

    if !pos.is_check() {
        legal_moves = legal_moves
            .into_iter()
            .filter(|mv| mv.is_capture())
            .collect::<ArrayVec<Move, 256>>();

        if legal_moves.is_empty() {
            return Result {
                score: stand_pat,
                chosen_move: None,
                computed: true,
            };
        }
    }

    let mut move_order: Vec<(i16, &Move)> = legal_moves
        .iter()
        .map(|mv| {
            let role = mv.role();
            return match role {
                Role::Pawn => (1, mv),
                Role::Knight => (3, mv),
                Role::Bishop => (4, mv),
                Role::Rook => (5, mv),
                Role::Queen => (9, mv),
                Role::King => (10, mv),
            };
        })
        .collect::<Vec<(i16, &Move)>>();

    move_order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    for (_, next_move) in move_order {
        let mut new_pos = pos.clone();
        new_pos.play_unchecked(&next_move);

        let mut result: Result = quiescence(&mut engine, new_pos, qdepth - 1, -beta, -alpha, end_time);
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
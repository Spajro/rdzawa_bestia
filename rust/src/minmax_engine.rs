use std::ops::Add;
use std::time::{Duration, Instant};
use shakmaty::{Chess, Move, Position, MoveList};
use output::send_move;
use crate::engine::Engine;
use crate::evaluation::eval;
use crate::output;
use crate::time_management::default_time_manager;

struct Result {
    score: f32,
    chosen_move: Option<Move>,
    computed: bool,
}

pub struct MinMaxEngine {
    pub pos: Chess,
}

impl Engine for MinMaxEngine {
    fn start(&mut self, time: u64) {
        send_move(self.find_best_move(time))
    }

    fn stop(&mut self) {
        send_move(self.find_best_move(0))
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

impl MinMaxEngine {
    fn negamax(&self, pos: Chess, depth: i32, mut alpha: f32, beta: f32, end_time: Instant) -> Result {
        if end_time <= Instant::now() {
            return Result {
                score: alpha,
                chosen_move: None,
                computed: false,
            };
        }
        if pos.outcome().is_some() || depth == 0 {
            let evl = if pos.turn().is_white() {
                eval(&pos)
            } else {
                -eval(&pos)
            };
            return Result { score: evl, chosen_move: None, computed: true };
        }

        let legal_moves: MoveList = pos.legal_moves();
        let mut best_move = legal_moves[0].clone();
        for next_move in legal_moves {
            let mut new_pos = pos.clone();
            new_pos.play_unchecked(&next_move);

            let mut result: Result = self.negamax(new_pos, depth - 1, -beta, -alpha, end_time);
            result.score = -result.score;
            if result.computed == false {
                return Result { score: alpha, chosen_move: Some(best_move), computed: false };
            }

            if result.score >= beta {
                return Result { score: beta, chosen_move: Some(best_move), computed: true };
            }

            if result.score > alpha {
                alpha = result.score;
                best_move = next_move;
            }
        }

        return Result { score: alpha, chosen_move: Some(best_move), computed: true };
    }

    fn find_best_move(&mut self, time: u64) -> Move {
        let mut depth = 1;
        let max_depth = 30;

        // let mut best_score = -1e9;
        let mut best_move: Option<Move> = None;
        let end_time = Instant::now().add(Duration::from_millis(default_time_manager(time)));
        while depth <= max_depth {
            output::send_info(String::from("Depth:") + &*depth.to_string());
            let result = self.negamax(self.pos.clone(), depth, -1000000000.0, 1000000000.0, end_time);
            output::send_info(String::from("Score ") + &*result.score.to_string());
            if result.computed == false {
                depth -= 1;
                break;
            }
            best_move = result.chosen_move;
            // best_score = result.score;
            depth += 1;
        }
        output::send_info(String::from("Final depth:") + &*depth.to_string());
        let chosen = best_move.unwrap();
        self.pos.play_unchecked(&chosen);
        chosen.clone()
    }
}

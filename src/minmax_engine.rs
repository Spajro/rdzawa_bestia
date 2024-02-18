use crate::engine::Engine;
use crate::features::evaluation::eval;
use crate::features::killer_moves::KillerMoves;
use crate::features::opening_book::OpeningBook;
use crate::features::quiesence::quiescence;
use crate::features::time_management::default_time_manager;
use crate::io::output::send_info;
use crate::io::output::send_move;
use arrayvec::ArrayVec;
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
// use shakmaty::{CastlingMode, Chess, Move, MoveList, Position};
use std::ops::Add;
use std::time::{Duration, Instant};

pub struct Result {
    pub(crate) score: f32,
    pub(crate) chosen_move: Option<ChessMove>,
    pub(crate) computed: bool,
}

pub struct MinMaxEngine {
    pub pos: Board,
    pub killer_moves: ArrayVec<KillerMoves<{ Self::KILLER_MOVES_SIZE }>, { Self::MAX_DEPTH }>,
    pub evaluations_cnt: i32,
    pub book: OpeningBook,
}

impl Engine for MinMaxEngine {
    fn start(&mut self, time: u64) {
        // send_move(self.find_best_move(time))
    }

    fn stop(&mut self) {
        // send_move(self.find_best_move(0))
    }

    fn update(&mut self, mv: ChessMove) {
        // let mov = mv.to_uci(CastlingMode::Standard).to_string();
        // self.book = self.book.clone().update(mov);
        self.pos = self.pos.make_move_new(mv);
    }

    fn restart(&mut self) {
        self.pos = Board::default();
    }

    fn get_status(&self) -> Board {
        self.pos.clone()
    }
}

impl MinMaxEngine {
    const MAX_DEPTH: usize = 30;
    const KILLER_MOVES_SIZE: usize = 2;
    pub fn new(pos: Board) -> Self {
        let mut km = ArrayVec::<_, { Self::MAX_DEPTH }>::new();
        for _ in 0..Self::MAX_DEPTH {
            km.push(KillerMoves::<{ Self::KILLER_MOVES_SIZE }>::new());
        }
        MinMaxEngine {
            pos: pos,
            killer_moves: km,
            evaluations_cnt: 0,
            book: OpeningBook::new(),
        }
    }

    fn negamax(
        &mut self,
        pos: Board,
        depth: usize,
        qdepth: usize,
        mut alpha: f32,
        beta: f32,
        end_time: Instant,
    ) -> Result {
        if (self.evaluations_cnt & 511) == 0 && end_time <= Instant::now() {
            return Result {
                score: alpha,
                chosen_move: None,
                computed: false,
            };
        }

        let legal_moves = MoveGen::new_legal(&pos);
        if pos.status() != BoardStatus::Ongoing {
            self.evaluations_cnt += 1;

            let evl = if pos.side_to_move() == Color::White {
                eval(&pos, false)
            } else {
                -eval(&pos, false)
            };
            return Result {
                score: evl,
                chosen_move: None,
                computed: true,
            };
        }

        if depth == 0 {
            return quiescence(self, pos, qdepth, alpha, beta, end_time);
        }

        let km_min_value = 1e6;
        let km_size: f32 = Self::KILLER_MOVES_SIZE as f32;
        // move ordering (killer moves first)
        let mut move_order = legal_moves
            .into_iter()
            .map(|mv: ChessMove| {
                for i in 0..self.killer_moves[depth].size {
                    if mv == self.killer_moves[depth].moves[i] {
                        return (km_min_value + km_size - i as f32, mv);
                    }
                }
                (0.0, mv)
            })
            .collect::<Vec<(f32, ChessMove)>>();

        // reverse sort
        move_order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let mut best_move = move_order[0].1;

        // send_info("len: ".to_string() + move_ordering.len().to_string().as_str());
        let mut new_pos = Board::default();
        for (value, next_move) in move_order {
            pos.make_move(next_move, &mut new_pos);

            let mut result: Result =
                self.negamax(new_pos, depth - 1, qdepth, -beta, -alpha, end_time);
            result.score = -result.score;

            if result.computed == false {
                return Result {
                    score: alpha,
                    chosen_move: Some(best_move),
                    computed: false,
                };
            }

            if result.score >= beta {
                return Result {
                    score: beta,
                    chosen_move: Some(best_move),
                    computed: true,
                };
            }

            if result.score > alpha {
                alpha = result.score;
                best_move = next_move.clone();

                if value < km_min_value {
                    self.killer_moves[depth].add(next_move.clone());
                }
            }
        }

        return Result {
            score: alpha,
            chosen_move: Some(best_move),
            computed: true,
        };
    }

    fn find_best_move(&mut self, time: u64) -> ChessMove {
        // let book_result = self.book.clone().try_get_best(&self.pos);
        // self.book = book_result.book;
        // if book_result.mv.is_some() {
        //     let mov = book_result.mv.unwrap();
        //     self.pos.play_unchecked(&mov);
        //     return mov;
        // }

        let mut depth = 1;
        let mut estimation = 0.0;
        let delta = 30.0; // 0.3 of the pawn
        let pos_inf = 1e9;
        let neg_inf = -1e9;
        // let mut best_score = -1e9;
        let mut best_move: Option<ChessMove> = MoveGen::new_legal(&self.pos).next();
        let end_time = Instant::now().add(Duration::from_millis(default_time_manager(time)));

        while depth < Self::MAX_DEPTH {
            send_info(String::from("Depth:") + &*depth.to_string());
            let alpha: f32 = estimation - delta;
            let beta: f32 = estimation + delta;
            let qdepth = 2 * depth;

            let mut result;
            if depth < 3 {
                result = self.negamax(self.pos.clone(), depth, qdepth, neg_inf, pos_inf, end_time);
            } else {
                result = self.negamax(self.pos.clone(), depth, qdepth, alpha, beta, end_time);

                if result.score >= beta {
                    result = self.negamax(
                        self.pos.clone(),
                        depth,
                        qdepth,
                        result.score,
                        pos_inf,
                        end_time,
                    );
                } else if result.score <= alpha {
                    result = self.negamax(
                        self.pos.clone(),
                        depth,
                        qdepth,
                        neg_inf,
                        result.score,
                        end_time,
                    );
                }

                if result.score <= alpha || result.score >= beta {
                    result =
                        self.negamax(self.pos.clone(), depth, qdepth, neg_inf, pos_inf, end_time);
                }
            }

            send_info(String::from("Score ") + &*result.score.to_string());
            if result.computed == false {
                depth -= 1;
                break;
            }
            estimation = result.score;
            best_move = result.chosen_move;
            depth += 1;
        }
        send_info(String::from("Final depth:") + &*depth.to_string());
        let chosen_move = best_move.unwrap();
        self.pos = self.pos.make_move_new(chosen_move);
        // eval(&self.pos, true);
        chosen_move.clone()
    }
}

// cargo flamegraph --unit-test -- mod_minmax_tests::minmax_depth8_inital_position
#[cfg(test)]
mod mod_minmax_tests {
    use super::*;

    #[test]
    fn minmax_depth8_inital_position() {
        let mut engine = MinMaxEngine::new(Board::default());
        let end_time = Instant::now().add(Duration::from_secs(60 * 10));
        let depth = 8;
        let result = engine.negamax(Board::default(), depth, 2 * depth, -1e9, 1e9, end_time);
        send_info(String::from("Score ") + &*result.score.to_string());
        println!("Evaluation_cnt={}", engine.evaluations_cnt);
    }
}

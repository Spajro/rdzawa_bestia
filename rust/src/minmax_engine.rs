use crate::engine::Engine;
use crate::evaluation::eval;
use crate::output::{self, send, send_info};
use crate::time_management::default_time_manager;
use arrayvec::ArrayVec;
use output::send_move;
use shakmaty::{Chess, Move, MoveList, Position, Role};
use std::ops::Add;
use std::time::{Duration, Instant};

struct Result {
    score: f32,
    chosen_move: Option<Move>,
    computed: bool,
}

#[derive(Clone)]
pub struct KillerMoves<const MAX_SIZE: usize> {
    moves: ArrayVec<Move, MAX_SIZE>,
    size: usize,
}

impl<const MAX_SIZE: usize> KillerMoves<MAX_SIZE> {
    pub fn new() -> Self {
        let mut moves = ArrayVec::<_, { MAX_SIZE }>::new();
        for _ in 0..MAX_SIZE {
            // TODO jakoś ładniej to zrobić:
            moves.push(Move::Put {
                role: shakmaty::Role::Pawn,
                to: shakmaty::Square::E5,
            });
        }
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

pub struct MinMaxEngine {
    pub pos: Chess,
    pub killer_moves: ArrayVec<KillerMoves<{ Self::KILLER_MOVES_SIZE }>, { Self::MAX_DEPTH }>,
    pub evaluations_cnt: i32,
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
    const MAX_DEPTH: usize = 30;
    const KILLER_MOVES_SIZE: usize = 2;
    pub fn new(pos: Chess) -> Self {
        let mut km = ArrayVec::<_, { Self::MAX_DEPTH }>::new();
        for _ in 0..Self::MAX_DEPTH {
            km.push(KillerMoves::<{ Self::KILLER_MOVES_SIZE }>::new());
        }
        MinMaxEngine {
            pos: pos,
            killer_moves: km,
            evaluations_cnt: 0,
        }
    }

    fn quiescence(
        &mut self,
        pos: Chess,
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

        let mut legal_moves: MoveList = pos.legal_moves();
        
        // if pos.outcome().is_some() || qdepth == 0 {
        if pos.is_variant_end()
            || legal_moves.is_empty()
            || pos.is_insufficient_material()
            || qdepth == 0
        {
            self.evaluations_cnt += 1;
            let evl = if pos.turn().is_white() {
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

        if !pos.is_check() {
            legal_moves = legal_moves
                .into_iter()
                .filter(|mv| mv.is_capture())
                .collect::<ArrayVec<Move, 256>>();
            if legal_moves.is_empty() {
                self.evaluations_cnt += 1;
                let evl = if pos.turn().is_white() {
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
        }

        let mut move_order: Vec<(i16, &Move)> = legal_moves
            .iter()
            .map(|mv| {
                let role = mv.role();
                match role {
                    Role::Pawn => return (1, mv),
                    Role::Knight => return (3, mv),
                    Role::Bishop => return (4, mv),
                    Role::Rook => return (5, mv),
                    Role::Queen => return (9, mv),
                    Role::King => return (10, mv),
                }
            })
            .collect::<Vec<(i16, &Move)>>();

        move_order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut best_move: Move = move_order[0].1.clone();
        for (_, next_move) in move_order {
            let mut new_pos = pos.clone();
            new_pos.play_unchecked(&next_move);

            let mut result: Result = self.quiescence(new_pos, qdepth - 1, -beta, -alpha, end_time);
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
            }
        }

        return Result {
            score: alpha,
            chosen_move: Some(best_move),
            computed: true,
        };
    }

    fn negamax(
        &mut self,
        pos: Chess,
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

        let legal_moves: MoveList = pos.legal_moves();
        // if pos.outcome().is_some() || qdepth == 0 {
        if pos.is_variant_end() || legal_moves.is_empty() || pos.is_insufficient_material() {
            self.evaluations_cnt += 1;
            let evl = if pos.turn().is_white() {
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
            return self.quiescence(pos, qdepth, alpha, beta, end_time);
        }

        let km_min_value = 1e6;
        let km_size: f32 = Self::KILLER_MOVES_SIZE as f32;
        // move ordering (killer moves first)
        let mut move_order: Vec<(f32, &Move)> = legal_moves
            .iter()
            .map(|mv| {
                for i in 0..self.killer_moves[depth].size {
                    if mv == &self.killer_moves[depth].moves[i] {
                        return (km_min_value + km_size - i as f32, mv);
                    }
                }
                (0.0, mv)
            })
            .collect::<Vec<(f32, &Move)>>();

        // reverse sort
        move_order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let mut best_move: Move = legal_moves[0].clone();

        // send_info("len: ".to_string() + move_ordering.len().to_string().as_str());
        for (value, next_move) in move_order {
            let mut new_pos = pos.clone();
            new_pos.play_unchecked(&next_move);

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

    fn find_best_move(&mut self, time: u64) -> Move {
        let mut depth = 1;

        // let mut best_score = -1e9;
        let mut best_move: Option<Move> = None;
        let end_time = Instant::now().add(Duration::from_millis(default_time_manager(time)));
        while depth <= Self::MAX_DEPTH {
            output::send_info(String::from("Depth:") + &*depth.to_string());
            let result = self.negamax(
                self.pos.clone(),
                depth,
                depth,
                -1000000000.0,
                1000000000.0,
                end_time,
            );
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
        eval(&self.pos, true);
        chosen.clone()
    }
}

// cargo flamegraph --unit-test -- mod_minmax_tests::minmax_depth8_inital_position
#[cfg(test)]
mod mod_minmax_tests {
    use super::*;

    #[test]
    fn minmax_depth8_inital_position() {
        let mut engine = MinMaxEngine::new(Chess::default());
        let end_time = Instant::now().add(Duration::from_secs(60 * 10));
        let depth = 8;
        let result = engine.negamax(Chess::default(), depth, 2, -1e9, 1e9, end_time);
        send_info(String::from("Score ") + &*result.score.to_string());
        println!("Evaluation_cnt={}", engine.evaluations_cnt);
    }
}

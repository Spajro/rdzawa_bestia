use crate::engine::Engine;
use crate::evaluation::eval;
use crate::output::{self, send, send_info};
use crate::time_management::default_time_manager;
use arrayvec::ArrayVec;
use output::send_move;
use shakmaty::{Chess, Move, MoveList, Position};
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
        }
    }
    fn negamax(
        &mut self,
        pos: Chess,
        depth: usize,
        mut alpha: f32,
        beta: f32,
        end_time: Instant,
    ) -> Result {
        if end_time <= Instant::now() {
            return Result {
                score: alpha,
                chosen_move: None,
                computed: false,
            };
        }

        if pos.outcome().is_some() || depth == 0 {
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
        let legal_moves: MoveList = pos.legal_moves();

        let mut best_move: Move = legal_moves[0].clone();

        let km_size = self.killer_moves[depth].size;
        for i in 0..km_size {
            // if i > 0 {
            //     send_info("killer_moves".to_string() + &i.to_string());
            // }
            let next_move = self.killer_moves[depth].moves[i].clone();
            if pos.is_legal(&next_move) {
                let mut new_pos = pos.clone();
                new_pos.play_unchecked(&next_move);

                let result = &mut self.negamax(new_pos, depth - 1, -beta, -alpha, end_time);
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
        }

        for next_move in legal_moves {
            let mut new_pos = pos.clone();
            new_pos.play_unchecked(&next_move);

            let mut result: Result = self.negamax(new_pos, depth - 1, -beta, -alpha, end_time);
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

                self.killer_moves[depth].add(next_move);
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
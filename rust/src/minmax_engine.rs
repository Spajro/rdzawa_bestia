use std::fs;
use crate::engine::Engine;
use crate::evaluation::eval;
use crate::output::{self, send_info};
use crate::time_management::default_time_manager;
use arrayvec::ArrayVec;
use output::send_move;
use shakmaty::{CastlingMode, Chess, Move, MoveList, Position, Role};
use std::ops::Add;
use std::str::FromStr;
use std::time::{Duration, Instant};
use json::{JsonValue};
use shakmaty::uci::Uci;

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
    pub book: JsonValue,
    pub use_book: bool,
}

impl Engine for MinMaxEngine {
    fn start(&mut self, time: u64) {
        send_move(self.find_best_move(time))
    }

    fn stop(&mut self) {
        send_move(self.find_best_move(0))
    }

    fn update(&mut self, mv: Move) {
        let book = self.book.clone();
        let mov = mv.to_uci(CastlingMode::Standard).to_string();
        if self.use_book && book.has_key(mov.as_str()) {
            send_info("Move in book:".to_string() + &*mov);
            let nxt = self.book[mov.as_str()].clone();
            self.book = nxt;
        } else {
            self.use_book = false
        }
        send_info("Move updated:".to_string() + &*mov);
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
        let json = fs::read_to_string("book.json").unwrap();
        let book = json::parse(&*json).unwrap();
        MinMaxEngine {
            pos: pos,
            killer_moves: km,
            evaluations_cnt: 0,
            book: book,
            use_book: true,
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
        self.evaluations_cnt += 1;
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
                }
            })
            .collect::<Vec<(i16, &Move)>>();

        move_order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (_, next_move) in move_order {
            let mut new_pos = pos.clone();
            new_pos.play_unchecked(&next_move);

            let mut result: Result = self.quiescence(new_pos, qdepth - 1, -beta, -alpha, end_time);
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
                eval(&pos, &legal_moves, false)
            } else {
                -eval(&pos, &legal_moves, false)
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
        let book = self.book.clone();
        if self.use_book && book.has_key("best") {
            let mv = book["best"].as_str().unwrap();
            let nxt = self.book[mv].clone();
            self.book = nxt;
            send_info("Move from book:".to_string() + mv);
            let mov = Uci::from_str(mv).unwrap().to_move(&self.pos).unwrap();
            self.pos.play_unchecked(&mov);
            return mov;
        } else {
            self.use_book = false
        }
        send_info("No move found".to_string());

        let mut depth = 1;

        // let mut best_score = -1e9;
        let mut best_move: Option<Move> = Some(self.pos.legal_moves()[0].clone());
        let end_time = Instant::now().add(Duration::from_millis(default_time_manager(time)));
        while depth <= Self::MAX_DEPTH {
            send_info(String::from("Depth:") + &*depth.to_string());
            let result = self.negamax(
                self.pos.clone(),
                depth,
                2 * depth,
                -1000000000.0,
                1000000000.0,
                end_time,
            );
            send_info(String::from("Score ") + &*result.score.to_string());
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
        // eval(&self.pos, true);
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
        let result = engine.negamax(Chess::default(), depth, 2 * depth, -1e9, 1e9, end_time);
        send_info(String::from("Score ") + &*result.score.to_string());
        println!("Evaluation_cnt={}", engine.evaluations_cnt);
    }
}

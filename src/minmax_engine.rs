use crate::engine::Engine;
use crate::features::board_utils::{is_insufficient_material, status};
use crate::features::Evaluation;
use crate::features::killer_moves::KillerMoves;
use crate::features::opening_book::OpeningBook;
use crate::features::quiescence::quiescence;
use crate::features::time_management::default_time_manager;
use crate::io::output::send_info;
use crate::io::output::send_move;
use arrayvec::ArrayVec;
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
use std::ops::Add;
use std::str::FromStr;
use std::time::{Duration, Instant};
use chess::Piece::King;
use crate::features::nnue::accumulator::{Accumulator, from};
use crate::features::nnue::half_kp::HalfKP;
use crate::features::nnue::network::NNUE;
use crate::io::uci::Position;
use crate::features::transposition_table::{EntryType, TranspositionTable};
use crate::io::options::Options;

pub struct Result {
    pub score: f32,
    pub chosen_move: Option<ChessMove>,
    pub computed: bool,
}

pub struct MinMaxEngine {
    pub pos: Board,
    pub killer_moves: ArrayVec<KillerMoves<{ Self::KILLER_MOVES_SIZE }>, { Self::MAX_DEPTH }>,
    pub evaluations_cnt: i32,
    pub book: OpeningBook,
    pub transposition_table: TranspositionTable,
    pub evaluator: NNUE,
    pub accumulator: Accumulator,
}

impl Engine for MinMaxEngine {
    fn start(&mut self, time: u64) {
        send_move(self.find_best_move(time))
    }

    fn stop(&mut self) {
        send_move(self.find_best_move(0))
    }

    fn update(&mut self, fen: Position, moves: Vec<ChessMove>) {
        match fen {
            Position::FEN(fen) => {
                self.pos = Board::from_str(&fen).unwrap();
                self.book = OpeningBook::empty();
                for mv in moves {
                    self.pos = self.pos.make_move_new(mv)
                }
                self.book = OpeningBook::empty();
                self.accumulator = Accumulator::refresh(&self.evaluator.l_0, &HalfKP::board_to_feature_set(&self.pos), from(self.pos.side_to_move()))
            }
            Position::START => {
                let mv = moves.last().unwrap();
                let mov = mv.to_string();
                self.book = self.book.clone().update(mov);
                self.pos = self.pos.make_move_new(*mv);
                self.accumulator = Accumulator::refresh(&self.evaluator.l_0, &HalfKP::board_to_feature_set(&self.pos), from(self.pos.side_to_move()))
            }
        }
    }

    fn restart(&mut self) {
        self.pos = Board::default();
        self.evaluations_cnt = 0;
        self.transposition_table.restart();
        self.book = self.book.restart();

        self.killer_moves = ArrayVec::<_, { Self::MAX_DEPTH }>::new();
        for _ in 0..Self::MAX_DEPTH {
            self.killer_moves.push(KillerMoves::<{ Self::KILLER_MOVES_SIZE }>::new());
        }
    }

    fn evaluate(&self) -> f32 {
        let moves_generator = MoveGen::new_legal(&self.pos);
        let any_legal_move = moves_generator.size_hint().0 > 0;
        let insufficient_material = is_insufficient_material(&self.pos);

        let board_status = status(&self.pos, any_legal_move, insufficient_material);

        if self.pos.side_to_move() == Color::White {
            self.evaluator.eval(&self.pos, board_status, 0, &self.accumulator)
        } else {
            -self.evaluator.eval(&self.pos, board_status, 0, &self.accumulator)
        }
    }
}

impl MinMaxEngine {
    const MAX_DEPTH: usize = 30;
    const KILLER_MOVES_SIZE: usize = 2;
    pub fn new(pos: Board, options: &Options) -> Self {
        let mut km = ArrayVec::<_, { Self::MAX_DEPTH }>::new();
        for _ in 0..Self::MAX_DEPTH {
            km.push(KillerMoves::<{ Self::KILLER_MOVES_SIZE }>::new());
        }
        MinMaxEngine {
            pos: pos,
            killer_moves: km,
            evaluations_cnt: 0,
            book: OpeningBook::new(options.get_value("openings".to_string()).unwrap_or(&"book.json".to_string())),
            transposition_table: TranspositionTable::new(),
            evaluator: NNUE::new(),
            accumulator: Accumulator::new(),
        }
    }

    fn negamax(
        &mut self,
        pos: Board,
        depth: usize,
        qdepth: usize,
        total_depth: usize,
        mut alpha: f32,
        mut beta: f32,
        end_time: Instant,
    ) -> Result {
        if (self.evaluations_cnt & 511) == 0 && end_time <= Instant::now() {
            return Result {
                score: alpha,
                chosen_move: None,
                computed: false,
            };
        }

        let transposition_entry = self.transposition_table.find(&pos);
        if transposition_entry.is_some() {
            let entry = transposition_entry.unwrap();
            if entry.depth >= depth {
                match entry.entry_type {
                    EntryType::EXACT => return Result {
                        score: entry.score,
                        chosen_move: entry.mv,
                        computed: true,
                    },
                    EntryType::LOWER => beta = beta.min(entry.score),
                    EntryType::UPPER => alpha = alpha.max(alpha),
                }
            }
        }

        let moves_generator = MoveGen::new_legal(&pos);
        let any_legal_move = moves_generator.size_hint().0 > 0;
        let insufficient_material = is_insufficient_material(&pos);

        let board_status = status(&pos, any_legal_move, insufficient_material);
        if board_status != BoardStatus::Ongoing {
            self.evaluations_cnt += 1;

            let evl = if pos.side_to_move() == Color::White {
                self.evaluator.eval(&pos, board_status, total_depth, &self.accumulator)
            } else {
                -self.evaluator.eval(&pos, board_status, total_depth, &self.accumulator)
            };
            self.transposition_table.insert(&pos, evl, None, depth, EntryType::EXACT);
            return Result {
                score: evl,
                chosen_move: None,
                computed: true,
            };
        }

        if depth == 0 {
            return quiescence(self, pos, qdepth, total_depth, alpha, beta, end_time);
        }

        let km_min_value = 1e6;
        let km_size: f32 = Self::KILLER_MOVES_SIZE as f32;
        // move ordering (killer moves first)
        let mut move_order = moves_generator
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
        let mut best_move = move_order[0].1.clone();

        // send_info("len: ".to_string() + move_ordering.len().to_string().as_str());
        let mut new_pos = pos.clone();
        for (value, next_move) in move_order {
            pos.make_move(next_move, &mut new_pos);

            let features = HalfKP::move_to_features_difference(&next_move, &pos);
            if pos.piece_on(next_move.get_source()).unwrap() == King {
                self.accumulator = Accumulator::refresh(&self.evaluator.l_0, &HalfKP::board_to_feature_set(&new_pos), from(new_pos.side_to_move()))
            } else {
                self.accumulator.update(&self.evaluator.l_0, &features.added, &features.removed, from(pos.side_to_move()));
            }

            let mut result: Result = self.negamax(
                new_pos,
                depth - 1,
                qdepth,
                total_depth + 1,
                -beta,
                -alpha,
                end_time,
            );

            if pos.piece_on(next_move.get_source()).unwrap() == King {
                self.accumulator = Accumulator::refresh(&self.evaluator.l_0, &HalfKP::board_to_feature_set(&pos), from(pos.side_to_move()))
            } else {
                self.accumulator.update(&self.evaluator.l_0, &features.removed, &features.added, from(pos.side_to_move()));
            }

            result.score = -result.score;

            if result.computed == false {
                return Result {
                    score: alpha,
                    chosen_move: Some(best_move),
                    computed: false,
                };
            }

            if result.score >= beta {
                self.transposition_table.insert(&pos, beta, Some(best_move.clone()), depth, EntryType::LOWER);
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
        self.transposition_table.insert(&pos, alpha, Some(best_move.clone()), depth, EntryType::UPPER);
        return Result {
            score: alpha,
            chosen_move: Some(best_move),
            computed: true,
        };
    }

    fn find_best_move(&mut self, time: u64) -> ChessMove {
        let book_result = self.book.clone().try_get_best();
        self.book = book_result.book;
        if book_result.mv.is_some() {
            let mov = book_result.mv.unwrap();
            self.pos = self.pos.make_move_new(mov);
            return mov;
        }

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
                result = self.negamax(
                    self.pos.clone(),
                    depth,
                    qdepth,
                    0,
                    neg_inf,
                    pos_inf,
                    end_time,
                );
            } else {
                result = self.negamax(self.pos.clone(), depth, qdepth, 0, alpha, beta, end_time);

                if result.score >= beta {
                    result = self.negamax(
                        self.pos.clone(),
                        depth,
                        qdepth,
                        0,
                        result.score,
                        pos_inf,
                        end_time,
                    );
                } else if result.score <= alpha {
                    result = self.negamax(
                        self.pos.clone(),
                        depth,
                        qdepth,
                        0,
                        neg_inf,
                        result.score,
                        end_time,
                    );
                }

                if result.score <= alpha || result.score >= beta {
                    result = self.negamax(
                        self.pos.clone(),
                        depth,
                        qdepth,
                        0,
                        neg_inf,
                        pos_inf,
                        end_time,
                    );
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
    use std::str::FromStr;

    #[test]
    fn minmax_depth8_inital_position() {
        let pos = Board::default();
        let mut engine = MinMaxEngine::new(pos, &Options::new());
        let start_time = Instant::now();
        let max_time = start_time.add(Duration::from_secs(60 * 10));
        let depth = 8;
        let result = engine.negamax(pos, depth, 2 * depth, 0, -1e9, 1e9, max_time);
        let duration = Instant::now().duration_since(start_time);

        println!("best move: {:?}", result.chosen_move);
        let m = result.chosen_move.unwrap();
        println!(
            "{:?} {:?}",
            m.get_source().get_file(),
            m.get_source().get_rank()
        );
        println!(
            "{:?} {:?}",
            m.get_dest().get_file(),
            m.get_dest().get_rank()
        );

        send_info(String::from("Score ") + &*result.score.to_string());
        println!("Evaluation_cnt={}", engine.evaluations_cnt);

        let evaluations_per_second = engine.evaluations_cnt as f32 / duration.as_secs_f32();
        println!("Evaluations per second = {}", evaluations_per_second);
    }

    #[test]
    fn test_quiescence() {
        let mut engine = MinMaxEngine::new(Board::default(), &Options::new());
        let end_time = Instant::now().add(Duration::from_secs(60 * 10));
        let pos = Board::from_str("r1b2r1k/4qp1p/p1Nppb1Q/4nP2/1p2P3/2N5/PPP4P/2KR1BR1 b - - 5 18")
            .unwrap();
        quiescence(&mut engine, pos, 10, 0, -1e9, 1e9, end_time);
    }
}

mod checkmate_tests {
    use super::*;
    use std::str::FromStr;
    use test_case::test_case;

    // Tests are from:
    // https://www.chess.com/forum/view/more-puzzles/hardest-mate-in-1-puzzles
    // https://www.chess.com/forum/view/livechess/practice-your-checkmate-in-4-moves-in-24-puzzles
    #[test_case(
    "r1b2b1r/pp3Qp1/2nkn2p/3ppP1p/P1p5/1NP1NB2/1PP1PPR1/1K1R3q w - - 0 1",
    1
    )]
    #[test_case("r4r1k/1R1R2p1/7p/8/8/3Q1Ppq/P7/6K1 w - - 0 1", 4)]
    #[test_case("3rr1k1/pp3ppp/3b4/2p5/2Q5/6qP/PPP1B1P1/R1B2K1R b - - 0 1", 4)]
    #[test_case("6k1/ppp2pp1/1q4b1/5rQp/8/1P6/PBP2PPP/3R2K1 w - - 0 1", 4)]
    #[test_case("8/6k1/8/3b3Q/pP4P1/1P6/KP3r2/N4r2 b - - 0 1", 4)]
    #[test_case("3r1rk1/pp3p1p/2b1n1pQ/q2pRN2/8/bP1B4/PB3PPP/2K4R w - - 0 1", 4)]
    #[test_case("1k3r2/ppN3bp/3R4/8/1P2nQ2/P4pPq/5P2/6K1 w - - 0 1", 4)]
    #[test_case("6rk/5Q1p/p1b2p1B/8/7N/8/PqP2PPP/6K1 w - - 0 1", 4)]
    #[test_case("r1b2r1k/1ppq1p1p/p2p4/4n3/2BQ4/1P6/PB3P1P/6RK w - - 0 1", 4)]
    #[test_case("3r2k1/1pQ3p1/p6p/8/4b3/PP2P1PP/5P1n/4K3 b - - 0 1", 4)]
    #[test_case("1k6/2p3r1/p6p/1pQP4/3N2q1/8/P5P1/6K1 w - - 0 1", 4)]
    #[test_case("R1Q5/1p3p2/1k2pb2/1B1p4/P7/P2P2P1/4rPK1/4q3 w - - 0 1", 4)]
    #[test_case("1rr1q2k/p4p1p/8/3pPp2/3P3Q/5R2/P5PP/6K1 w - - 0 1", 4)]
    #[test_case("6rk/5p1p/3p1p1Q/1p2qP2/4P3/1P2BR2/r5PP/6K1 w - - 0 1", 4)]
    #[test_case("1k2r2r/2p3pp/Q2pqp2/2B5/2P5/1P3P2/5NPP/6K1 w - - 0 1", 4)]
    #[test_case("r3r1k1/2RR4/1p5P/3p2p1/q7/3P1P2/1PPNB3/1K6 w - - 0 1", 4)]
    #[test_case("R4nk1/4rpbp/1p4p1/5bPP/3QN3/1qP5/1P6/2K5 w - - 0 1", 4)]
    #[test_case("6k1/r5r1/2bpNp1R/q1bN1P2/1p6/6P1/1PPQ4/1K6 w - - 0 1", 4)]
    #[test_case("7r/1k2b3/6p1/4p3/1pBn1n2/1P6/5PNP/R2R2K1 b - - 0 1", 4)]
    #[test_case("1r4r1/5ppk/pqp1p2p/4Nn1N/6QP/8/PPP2P2/2K3R1 w - - 0 1", 4)]
    #[test_case("kr5r/p2R1ppp/2p2q2/4n3/8/1P1B2P1/P2Q1P1P/5RK1 w - - 0 1", 4)]
    fn checkmate(fen: &str, moves_to_mate: usize) {
        let expected_depth = moves_to_mate * 2 - 1;
        let board = Board::from_str(fen).unwrap();

        let mut engine = MinMaxEngine::new(board, &Options::new());

        for depth in 1..(expected_depth + 1) {
            engine.evaluations_cnt = 0;
            let start_time = Instant::now();
            let max_time = start_time.add(Duration::from_secs(60 * 10));

            // quiescence has to be disabled!
            let result = engine.negamax(board, depth, 0, 0, -1e9, 1e9, max_time);

            let duration = Instant::now().duration_since(start_time);
            println!(
                "Depth= {} Score={} Evaluation_cnt= {} Duration= {:?}",
                depth, result.score, engine.evaluations_cnt, duration
            );

            if depth < expected_depth {
                assert!(result.score.abs() < 1e8);
            } else {
                assert!(result.score > 1e8);
            }
        }
    }
}

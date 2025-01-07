use crate::engine::Engine;
use crate::features::board_utils::{is_insufficient_material, status};
use crate::features::evaluation::eval;
use crate::features::opening_book::OpeningBook;
use crate::io::options::Options;
use crate::io::output::send_info;
use crate::io::output::send_move;
use crate::io::uci::Position;
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
use std::str::FromStr;

pub struct Result {
    pub(crate) score: f32,
    pub(crate) chosen_move: Option<ChessMove>,
    pub(crate) computed: bool,
}

pub struct BasicMinMaxEngine {
    pub pos: Board,
    pub evaluations_cnt: i32,
    pub book: OpeningBook,
}

impl Engine for BasicMinMaxEngine {
    fn start(&mut self, _time: u64) {
        send_move(self.find_best_move())
    }

    fn stop(&mut self) {
        send_move(self.find_best_move())
    }

    fn update(&mut self, fen: Position, moves: Vec<ChessMove>) {
        match fen {
            Position::FEN(fen) => {
                self.pos = Board::from_str(&fen).unwrap();
                self.book = OpeningBook::empty();
                for mv in moves {
                    self.pos = self.pos.make_move_new(mv)
                }
            }
            Position::START => {
                let mv = moves.last().unwrap();
                let mov = mv.to_string();
                self.book = self.book.clone().update(mov);
                self.pos = self.pos.make_move_new(*mv);
            }
        }
    }

    fn restart(&mut self) {
        self.pos = Board::default();
        self.evaluations_cnt = 0;
        self.book = self.book.restart();
    }

    fn evaluate(&self) -> f32 {
        let moves_generator = MoveGen::new_legal(&self.pos);
        let any_legal_move = moves_generator.size_hint().0 > 0;
        let insufficient_material = is_insufficient_material(&self.pos);

        let board_status = status(&self.pos, any_legal_move, insufficient_material);

        if self.pos.side_to_move() == Color::White {
            eval(&self.pos, board_status, 0)
        } else {
            -eval(&self.pos, board_status, 0)
        }
    }
}

impl BasicMinMaxEngine {
    pub fn _new(pos: Board, options: &Options) -> Self {
        BasicMinMaxEngine {
            pos: pos,
            evaluations_cnt: 0,
            book: OpeningBook::new(
                options
                    .get_value("openings".to_string())
                    .unwrap_or(&"book.json".to_string()),
            ),
        }
    }

    fn negamax(
        &mut self,
        pos: Board,
        depth: usize,
        total_depth: usize,
        mut alpha: f32,
        beta: f32,
    ) -> Result {
        let moves_generator = MoveGen::new_legal(&pos);
        let any_legal_move = moves_generator.size_hint().0 > 0;
        let insufficient_material = is_insufficient_material(&pos);

        let board_status = status(&pos, any_legal_move, insufficient_material);
        if board_status != BoardStatus::Ongoing {
            self.evaluations_cnt += 1;

            let evl = if pos.side_to_move() == Color::White {
                eval(&pos, board_status, total_depth)
            } else {
                -eval(&pos, board_status, total_depth)
            };
            return Result {
                score: evl,
                chosen_move: None,
                computed: true,
            };
        }

        // move ordering (killer moves first)
        let mut move_order = moves_generator
            .into_iter()
            .map(|mv: ChessMove| (0.0, mv))
            .collect::<Vec<(f32, ChessMove)>>();

        // reverse sort
        move_order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let mut best_move = move_order[0].1.clone();

        // send_info("len: ".to_string() + move_ordering.len().to_string().as_str());
        let mut new_pos = pos.clone();
        for (_, next_move) in move_order {
            pos.make_move(next_move, &mut new_pos);

            let mut result: Result =
                self.negamax(new_pos, depth - 1, total_depth + 1, -beta, -alpha);
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

    fn find_best_move(&mut self) -> ChessMove {
        let book_result = self.book.clone().try_get_best();
        self.book = book_result.book;
        if book_result.mv.is_some() {
            let mov = book_result.mv.unwrap();
            self.pos = self.pos.make_move_new(mov);
            return mov;
        }

        let depth = 2;
        let pos_inf = 1e9;
        let neg_inf = -1e9;
        // let mut best_score = -1e9;

        send_info(String::from("Depth:") + &*depth.to_string());

        let result = self.negamax(self.pos.clone(), depth, 0, neg_inf, pos_inf);

        send_info(String::from("Score ") + &*result.score.to_string());

        let chosen_move = result.chosen_move.unwrap();
        self.pos = self.pos.make_move_new(chosen_move);
        // eval(&self.pos, true);
        chosen_move.clone()
    }
}

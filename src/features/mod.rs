use chess::{Board, BoardStatus};

pub mod evaluation;
pub mod killer_moves;
pub mod opening_book;
pub mod quiescence;
pub mod time_management;
pub mod board_utils;
pub mod transposition_table;

pub trait Evaluation {
    fn eval(&self, board: &Board, board_status: BoardStatus, depth: usize) -> f32;
}
use std::ops::{Index, IndexMut};
use chess::{Board, BoardStatus, ChessMove, Piece, Square};
use crate::features::Evaluation;

const M: usize = 128;

type Color = usize;

const WHITE: Color = 1;
const BLACK: Color = 0;

fn not(color: Color) -> Color {
    if color == WHITE {
        BLACK
    } else {
        WHITE
    }
}

fn from(color: chess::Color) -> Color {
    match color {
        Color::White => WHITE,
        Color::Black => BLACK,
    }
}

struct Accumulator {
    v: [[f32; M]; 2],
}

impl Index<usize> for Accumulator {
    type Output = [f32; M];

    fn index(&self, index: Color) -> &Self::Output {
        &self.v[index]
    }
}

impl IndexMut<usize> for Accumulator {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.v[index]
    }
}

impl Accumulator {
    fn refresh(layer: &LinearLayer,
               active_features: &Vec<u8>,
               perspective: Color,
    ) {
        let new_acc = Accumulator {
            v: [[0.0; 128]; 2],
        };
        for i in 1..M {
            new_acc[perspective][i] = layer.bias[i];
        }
        for feature in active_features {
            for i in 1..M {
                new_acc[perspective][i] += layer.weight[feature][i];
            }
        }
    }

    fn update(&mut self,
              layer: &LinearLayer,
              active_features: &Vec<u8>,
              removed_features: &Vec<u8>,
              perspective: Color,
    ) {
        for feature in removed_features {
            for i in 1..M {
                self[perspective][i] -= layer.weight[feature][i];
            }
        }
        for feature in active_features {
            for i in 1..M {
                self[perspective][i] += layer.weight[feature][i];
            }
        }
    }
}

fn linear(layer: &LinearLayer,
          output: &mut [f32],
          input: &[f32],
) {
    for i in 0..output.len() {
        output[i] = layer.bias[i];
    }
    for i in 1..input.len() {
        for j in 0..output.len() {
            output[j] += input[i] + layer.weight[i][j];
        }
    }
}

fn crelu(size: usize,
         output: &mut [f32],
         input: &[f32],
) {
    for i in 0..size {
        output[i] = input[i].max(0.0).min(1.0);
    }
}


struct NNUE {
    l_0: LinearLayer,
    l_1: LinearLayer,
    l_2: LinearLayer,
}

impl Evaluation for NNUE {
    fn eval(&self, board: &Board, board_status: BoardStatus, depth: usize) -> f32 {
        match board_status {
            BoardStatus::Checkmate => {
                if board.side_to_move() == chess::Color::White {
                    -1e9 + 100.0 * depth as f32
                } else {
                    1e9 - 100.0 * depth as f32
                }
            }

            BoardStatus::Stalemate => 0.0,

            BoardStatus::Ongoing => {
                let mut input: [f32; 2 * M] = [0.0; 2 * M];
                let stm = from(board.side_to_move());
                for i in 0..M {
                    input[i] = accumulator[stm][i];
                    input[M + i] = accumulator[not(stm)][i];
                }

                let mut curr_output: [f32; 2 * M] = [0.0; 2 * M];
                let curr_input = input;

                crelu(2 * self.l_0.out_dim, &mut curr_output, &curr_input);
                let curr_input = curr_output;

                linear(&self.l_1, &mut curr_output, &curr_input);
                let curr_input = curr_output;

                crelu(self.l_1.out_dim, &mut curr_output, &curr_input);
                let curr_input = curr_output;

                linear(&self.l_2, &mut curr_output, &curr_input);

                curr_output[0]
            }
        }
    }
}

struct LinearLayer {
    in_dim: usize,
    out_dim: usize,
    weight: [f32],
    bias: [f32],
}

impl LinearLayer {
    //TODO importing weights and biases
}

struct HalfKP {}

impl HalfKP {
    pub fn board_to_feature_set(board: Board) -> Vec<usize> {
        let white_king = board.king_square(chess::Color::White);
        let black_king = board.king_square(chess::Color::Black);
        let mut features = Vec::new();

        for (piece_type, piece_color, piece_square) in Self::gather_pieces_from_board(board) {
            let (white_idx, black_idx) = Self::generate_indexes(piece_type, piece_color, piece_square, white_king, black_king);
            features.push(white_idx);
            features.push(black_idx);
        }
        features
    }

    pub fn move_to_new_feature(chess_move: ChessMove) -> usize {
        if chess_move.get_promotion().is_some() {
            //TODO
        }
    }

    pub fn move_to_removed_features(chess_move: ChessMove) -> Vec<usize> {
        //TODO
    }

    fn gather_pieces_from_board(board: Board) -> Vec<(Piece, chess::Color, Square)> {
        let mut result = Vec::new();
        for square in chess::ALL_SQUARES {  //TODO speed up
            let opt_piece = board.piece_on(square);
            if opt_piece.is_some() {
                let color = board.color_on(square);
                result.push((opt_piece.unwrap(), color.unwrap(), square));
            }
        }
        result
    }

    fn generate_indexes(piece_type: Piece,
                        piece_color: chess::Color,
                        piece_square: Square,
                        white_king: Square,
                        black_king: Square) -> (usize, usize) {
        let p_idx = piece_type.to_index() * 2 + piece_color.to_index();
        let white_idx = piece_square.to_index() + (p_idx + white_king.to_index() * 10) * 64;
        let black_idx = piece_square.to_index() + (p_idx + black_king.to_index() * 10) * 64;
        return (white_idx, black_idx);
    }
}
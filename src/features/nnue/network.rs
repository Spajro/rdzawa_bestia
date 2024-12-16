use chess::{Board, BoardStatus};

use crate::features::nnue::accumulator::{Accumulator, from, M, not};

pub struct NNUE {
    pub accumulator: Accumulator,
    pub l_0: LinearLayer,
    pub l_1: LinearLayer,
    pub l_2: LinearLayer,
}

impl NNUE {
    pub(crate) fn eval(&self, board: &Board, board_status: BoardStatus, depth: usize) -> f32 {
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
                    input[i] = self.accumulator[stm][i];
                    input[M + i] = self.accumulator[not(stm)][i];
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

pub struct LinearLayer {
    pub in_dim: usize,
    pub out_dim: usize,
    pub weight: [f32],
    pub bias: [f32],
}

impl LinearLayer {
    //TODO importing weights and biases
}

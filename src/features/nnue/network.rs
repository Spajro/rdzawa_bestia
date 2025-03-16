use chess::{Board, BoardStatus};

use crate::features::nnue::accumulator::{Accumulator, from, M, not};
use crate::io::output::send_info;

pub struct NNUE<const X0: usize, const X1: usize, const X2: usize> {
    pub accumulator: Accumulator,
    pub l_0: LinearLayer<X0, X1>,
    pub l_1: LinearLayer<X1, X2>,
    pub l_2: LinearLayer<X2, 1>,
}

impl<const X0: usize, const X1: usize, const X2: usize> NNUE<X0, X1, X2> {
    pub fn empty() -> Self {
        NNUE {
            accumulator: Accumulator::new(),
            l_0: LinearLayer::empty(),
            l_1: LinearLayer::empty(),
            l_2: LinearLayer::empty(),
        }
    }
    pub fn new(weight1: Vec<[f32; X0]>, bias1: [f32; X1],
               weight2: Vec<[f32; X1]>, bias2: [f32; X2],
               weight3: Vec<[f32; X2]>, bias3: [f32; 1]) -> Self {
        NNUE {
            accumulator: Accumulator::new(),
            l_0: LinearLayer::new(weight1, bias1),
            l_1: LinearLayer::new(weight2, bias2),
            l_2: LinearLayer::new(weight3, bias3),
        }
    }
    pub fn eval(&self, board: &Board, board_status: BoardStatus, depth: usize, accumulator: &Accumulator) -> i32 {
        match board_status {
            BoardStatus::Checkmate => {
                if board.side_to_move() == chess::Color::White {
                    -1e9 as i32 + 100 * depth as i32
                } else {
                    1e9 as i32 - 100 * depth as i32
                }
            }

            BoardStatus::Stalemate => 0,

            BoardStatus::Ongoing => {
                let mut input: [f32; 2 * M] = [0.0; 2 * M];
                let stm = from(board.side_to_move());
                for i in 0..M {
                    input[i] = accumulator[stm][i];
                    input[M + i] = accumulator[not(stm)][i];
                }

                let mut curr_output: [f32; 2 * M] = [0.0; 2 * M];
                let curr_input = input;

                crelu(self.l_0.out_dim, &mut curr_output, &curr_input);
                let curr_input = curr_output;

                linear(&self.l_1, &mut curr_output, &curr_input);
                let curr_input = curr_output;

                crelu(self.l_1.out_dim, &mut curr_output, &curr_input);
                let curr_input = curr_output;

                linear(&self.l_2, &mut curr_output, &curr_input);

                (curr_output[0] * 410.0) as i32
            }
        }
    }
}

fn linear<const IN: usize, const OUT: usize>(layer: &LinearLayer<IN, OUT>,
                                             output: &mut [f32],
                                             input: &[f32],
) {
    for i in 0..OUT {
        output[i] = layer.bias[i];
    }
    for i in 0..OUT {
        for j in 0..IN {
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

pub struct LinearLayer<const IN: usize, const OUT: usize> {
    pub in_dim: usize,
    pub out_dim: usize,
    pub weight: Vec<[f32; IN]>,
    pub bias: [f32; OUT],
}

impl<const IN: usize, const OUT: usize> LinearLayer<IN, OUT> {
    pub fn empty() -> Self {
        LinearLayer {
            in_dim: IN,
            out_dim: OUT,
            weight: vec![],
            bias: [0.0; OUT],
        }
    }
    pub fn new(weight: Vec<[f32; IN]>, bias: [f32; OUT]) -> Self {
        Self {
            in_dim: IN,
            out_dim: OUT,
            weight,
            bias,
        }
    }
}

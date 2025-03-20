use std::str::FromStr;

use chess::{Board, BoardStatus, ChessMove, MoveGen};
use chess::Color::{Black, White};

use crate::features::board_utils::{is_insufficient_material, status};
use crate::features::nnue::accumulator::{Accumulator, from, not};
use crate::features::nnue::depickle::load_state;
use crate::features::nnue::half_kp::HalfKP;

pub struct NNUE<const M: usize, const X0: usize, const X1: usize, const X2: usize> {
    pub accumulator: Accumulator<M>,
    pub l_0: LinearLayer<X0, X1>,
    pub l_1: LinearLayer<X1, X2>,
    pub l_2: LinearLayer<X2, 1>,
}

impl<const M: usize, const X0: usize, const X1: usize, const X2: usize> NNUE<M, X0, X1, X2> {
    pub fn empty() -> Self {
        NNUE {
            accumulator: Accumulator::new(),
            l_0: LinearLayer::empty(),
            l_1: LinearLayer::empty(),
            l_2: LinearLayer::empty(),
        }
    }
    pub fn new(weight1: Vec<[f64; X0]>, bias1: [f64; X1],
               weight2: Vec<[f64; X1]>, bias2: [f64; X2],
               weight3: Vec<[f64; X2]>, bias3: [f64; 1]) -> Self {
        NNUE {
            accumulator: Accumulator::new(),
            l_0: LinearLayer::new(weight1, bias1),
            l_1: LinearLayer::new(weight2, bias2),
            l_2: LinearLayer::new(weight3, bias3),
        }
    }
    pub fn eval(&self, board: &Board, board_status: BoardStatus, depth: usize, accumulator: &Accumulator<M>) -> i32 {
        match board_status {
            BoardStatus::Checkmate => {
                if board.side_to_move() == White {
                    -1e9 as i32 + 100 * depth as i32
                } else {
                    1e9 as i32 - 100 * depth as i32
                }
            }

            BoardStatus::Stalemate => 0,

            BoardStatus::Ongoing => {
                let mut curr_output: [f64; X1] = [0.0; X1];
                let mut curr_input: [f64; X1] = [0.0; X1];

                let stm = from(board.side_to_move());
                for i in 0..M {
                    curr_input[i] = accumulator[stm][i];
                    curr_input[M + i] = accumulator[not(stm)][i];
                }

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

    pub fn eval_no_acc(&self, board: &Board, board_status: BoardStatus, depth: usize) -> i32 {
        match board_status {
            BoardStatus::Checkmate => {
                if board.side_to_move() == White {
                    -1e9 as i32 + 100 * depth as i32
                } else {
                    1e9 as i32 - 100 * depth as i32
                }
            }

            BoardStatus::Stalemate => 0,

            BoardStatus::Ongoing => {
                let (white_features, black_features) = &HalfKP::board_to_feature_set(&board);
                let mut curr_input: [f64; { 2 * 40960 }] = [0.0; { 2 * 40960 }];
                let mut curr_output: [f64; X1] = [0.0; X1];

                for &feature in white_features {
                    curr_input[feature] = 1.0;
                }
                for &feature in black_features {
                    curr_input[feature] = 1.0;
                }

                linear(&self.l_0, &mut curr_output, &curr_input);
                let curr_input = curr_output;

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
                                             output: &mut [f64],
                                             input: &[f64],
) {
    for i in 0..OUT {
        output[i] = layer.bias[i];
    }
    for i in 0..IN {
        for j in 0..OUT {
            output[j] += input[i] * layer.weight[j][i];
        }
    }
}

fn crelu(size: usize,
         output: &mut [f64],
         input: &[f64],
) {
    for i in 0..size {
        output[i] = input[i].max(0.0).min(1.0);
    }
}

pub struct LinearLayer<const IN: usize, const OUT: usize> {
    pub in_dim: usize,
    pub out_dim: usize,
    pub weight: Vec<[f64; IN]>,
    pub bias: [f64; OUT],
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
    pub fn new(weight: Vec<[f64; IN]>, bias: [f64; OUT]) -> Self {
        println!("IN {} | OUT {} weight {}x{} | bias {}", IN, OUT, weight.len(), weight[0].len(), bias.len());
        Self {
            in_dim: IN,
            out_dim: OUT,
            weight,
            bias,
        }
    }
}


#[test]
fn test() {
    let nnue_wb = load_state::<{ 2 * 40960 }, 256, 32>(&"resources/halfkp2-cp-50.pth_wb.pt".to_string());
    let wb = nnue_wb.unwrap();
    let mut nnue: NNUE<128,{ 2 * 40960 }, 256, 32> = NNUE::new(wb.0, wb.1, wb.2, wb.3, wb.4, wb.5);
    println!("{:?}", nnue.l_2.weight);
    let pos = Board::default();
    let moves_generator = MoveGen::new_legal(&pos);
    let board_status = status(&pos, moves_generator.size_hint().0 > 0, is_insufficient_material(&pos));
    let (white_features, black_features) = &HalfKP::board_to_feature_set(&pos);

    nnue.accumulator.refresh(&nnue.l_0, white_features, from(White));
    nnue.accumulator.refresh(&nnue.l_0, black_features, from(Black));
    println!("Acc: {} NoAcc: {} Ref:{}", nnue.eval(&pos, board_status, 0, &nnue.accumulator), nnue.eval_no_acc(&pos, board_status, 0), 102);

    let mv: ChessMove = moves_generator.collect::<Vec<ChessMove>>()[5];
    println!("{}", mv);
    let new_pos = pos.make_move_new(mv);
    let (white_diff, black_diff) = HalfKP::move_to_features_difference(&mv, &pos);
    nnue.accumulator.update(&nnue.l_0, &white_diff.added, &white_diff.removed, from(White));
    nnue.accumulator.update(&nnue.l_0, &black_diff.added, &black_diff.removed, from(Black));
    let moves_generator = MoveGen::new_legal(&new_pos);
    let board_status = status(&new_pos, moves_generator.size_hint().0 > 0, is_insufficient_material(&new_pos));
    println!("Acc: {} NoAcc: {} Ref:{}", nnue.eval(&new_pos, board_status, 0, &nnue.accumulator), nnue.eval_no_acc(&new_pos, board_status, 0), 140);

    let mv: ChessMove = moves_generator.collect::<Vec<ChessMove>>()[5];
    println!("{}", mv);
    let new_pos = new_pos.make_move_new(mv);
    let (white_diff, black_diff) = HalfKP::move_to_features_difference(&mv, &pos);
    nnue.accumulator.update(&nnue.l_0, &white_diff.added, &white_diff.removed, from(White));
    nnue.accumulator.update(&nnue.l_0, &black_diff.added, &black_diff.removed, from(Black));
    let moves_generator = MoveGen::new_legal(&new_pos);
    let board_status = status(&new_pos, moves_generator.size_hint().0 > 0, is_insufficient_material(&new_pos));
    println!("Acc: {} NoAcc: {} Ref:{}", nnue.eval(&new_pos, board_status, 0, &nnue.accumulator), nnue.eval_no_acc(&new_pos, board_status, 0), 172);

    let mv: ChessMove = moves_generator.collect::<Vec<ChessMove>>()[5];
    println!("{}", mv);
    let new_pos = new_pos.make_move_new(mv);
    let (white_diff, black_diff) = HalfKP::move_to_features_difference(&mv, &pos);
    nnue.accumulator.update(&nnue.l_0, &white_diff.added, &white_diff.removed, from(White));
    nnue.accumulator.update(&nnue.l_0, &black_diff.added, &black_diff.removed, from(Black));
    let moves_generator = MoveGen::new_legal(&new_pos);
    let board_status = status(&new_pos, moves_generator.size_hint().0 > 0, is_insufficient_material(&new_pos));
    println!("Acc: {} NoAcc: {} Ref:{}", nnue.eval(&new_pos, board_status, 0, &nnue.accumulator), nnue.eval_no_acc(&new_pos, board_status, 0), 220);
}

#[test]
fn two_kings_update() {
    let nnue_wb = load_state::<{ 2 * 40960 }, 256, 32>(&"resources/halfkp2-cp-50.pth_wb.pt".to_string());
    let wb = nnue_wb.unwrap();
    let mut nnue: NNUE<128,{ 2 * 40960 }, 256, 32> = NNUE::new(wb.0, wb.1, wb.2, wb.3, wb.4, wb.5);

    let pos = Board::from_str("4k3/4p3/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();
    let (white_features, black_features) = &HalfKP::board_to_feature_set(&pos);
    nnue.accumulator.refresh(&nnue.l_0, white_features, from(White));
    nnue.accumulator.refresh(&nnue.l_0, black_features, from(Black));
    let moves_generator = MoveGen::new_legal(&pos);
    let board_status = status(&pos, moves_generator.size_hint().0 > 0, is_insufficient_material(&pos));
    println!("{} | {}", nnue.eval(&pos, board_status, 0, &nnue.accumulator), nnue.eval_no_acc(&pos, board_status, 0));

    let mv: ChessMove = moves_generator.collect::<Vec<ChessMove>>()[0];
    println!("{}", mv);
    let new_pos = pos.make_move_new(mv);
    let (white_diff, black_diff) = HalfKP::move_to_features_difference(&mv, &pos);
    nnue.accumulator.update(&nnue.l_0, &white_diff.added, &white_diff.removed, from(White));
    nnue.accumulator.update(&nnue.l_0, &black_diff.added, &black_diff.removed, from(Black));
    let moves_generator = MoveGen::new_legal(&new_pos);
    let board_status = status(&new_pos, moves_generator.size_hint().0 > 0, is_insufficient_material(&new_pos));
    println!("{} | {}", nnue.eval(&new_pos, board_status, 0, &nnue.accumulator), nnue.eval_no_acc(&new_pos, board_status, 0));

    let (white_features, black_features) = &HalfKP::board_to_feature_set(&new_pos);
    nnue.accumulator.refresh(&nnue.l_0, white_features, from(White));
    nnue.accumulator.refresh(&nnue.l_0, black_features, from(Black));
    let moves_generator = MoveGen::new_legal(&new_pos);
    let board_status = status(&new_pos, moves_generator.size_hint().0 > 0, is_insufficient_material(&new_pos));
    println!("{} | {}", nnue.eval(&new_pos, board_status, 0, &nnue.accumulator), nnue.eval_no_acc(&new_pos, board_status, 0));
}

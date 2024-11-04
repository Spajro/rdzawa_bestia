use chess::{BitBoard, Board, BoardStatus, Color};
use chess::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};
use tch::{Device, nn, Tensor};
use tch::nn::{Module, Sequential};
use tch::nn::VarStore;
use crate::features::Evaluation;

const INPUT_DIM: i64 = 512;
const HIDDEN_DIM_1: i64 = 2_i64.pow(10);
const HIDDEN_DIM_2: i64 = 2_i64.pow(5);
const HIDDEN_DIM_3: i64 = 2_i64.pow(3);

pub struct NeuralEvaluator {
    var_store: VarStore,
    model: Sequential,
}

impl NeuralEvaluator {
    pub fn new() -> Self {
        let mut var_store = VarStore::new(Device::Cpu);
        let model = Self::create_model(&var_store.root());
        var_store.load("model.pt").unwrap();
        Self { var_store, model }
    }

    fn create_model(vs: &nn::Path) -> Sequential {
        nn::seq()
            .add(nn::linear(
                vs / "layer1",
                INPUT_DIM,
                HIDDEN_DIM_1,
                Default::default(),
            ))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(
                vs / "layer2",
                HIDDEN_DIM_1,
                HIDDEN_DIM_2,
                Default::default(),
            ))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(
                vs / "layer3",
                HIDDEN_DIM_2,
                HIDDEN_DIM_3,
                Default::default(),
            ))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(
                vs / "layer4",
                HIDDEN_DIM_3,
                1,
                Default::default(),
            ))
    }
}

impl Evaluation for NeuralEvaluator {
    fn eval(&self, board: &Board, board_status: BoardStatus, depth: usize) -> f32 {
        match board_status {
            BoardStatus::Checkmate => {
                if board.side_to_move() == Color::White {
                    -1e9 + 100.0 * depth as f32
                } else {
                    1e9 - 100.0 * depth as f32
                }
            }

            BoardStatus::Stalemate => 0.0,

            BoardStatus::Ongoing => {
                self.model.forward(&board_to_tensor(board).reshape([-1])).double_value(&[0]) as f32
            }
        }
    }
}


fn board_to_tensor(board: &Board) -> Tensor {
    Tensor::stack(&[
        bitboard_to_tensor(board.color_combined(Color::White)),
        bitboard_to_tensor(board.color_combined(Color::Black)),
        bitboard_to_tensor(board.pieces(Pawn)),
        bitboard_to_tensor(board.pieces(King)),
        bitboard_to_tensor(board.pieces(Queen)),
        bitboard_to_tensor(board.pieces(Knight)),
        bitboard_to_tensor(board.pieces(Bishop)),
        bitboard_to_tensor(board.pieces(Rook))
    ], 0)
}

fn bitboard_to_tensor(bit_board: &BitBoard) -> Tensor {
    let mut array: [f32; 64] = [0.0; 64];
    for x in 0..64 {
        if bit_board.0 & (1u64 << x) == (1u64 << x) {
            array[x] = 1.0
        } else {
            array[x] = 0.0
        }
        if x % 8 == 7 {}
    }
    Tensor::from_slice(&array).reshape([8, 8])
}
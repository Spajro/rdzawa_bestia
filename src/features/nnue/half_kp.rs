use chess::{Board, ChessMove, Piece, Square};

struct HalfKP {}

struct FeaturesDifference {
    pub added: Vec<usize>,
    pub removed: Vec<usize>,
}

impl FeaturesDifference {
    pub fn new() -> Self {
        Self {
            added: vec![],
            removed: vec![],
        }
    }
}

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

    pub fn move_to_features_difference(chess_move: ChessMove,
                                       board: Board,
    ) -> FeaturesDifference {
        let white_king = board.king_square(chess::Color::White);
        let black_king = board.king_square(chess::Color::Black);
        let piece_type = board.piece_on(chess_move.get_source()).unwrap();
        let mut result = FeaturesDifference::new();


        if chess_move.get_promotion().is_some() {
            let (white_idx, black_idx) = Self::generate_indexes(chess_move.get_promotion().unwrap(),
                                                                board.side_to_move(),
                                                                chess_move.get_dest(),
                                                                white_king,
                                                                black_king);
            result.added.push(white_idx);
            result.added.push(black_idx);
        } else {
            let (white_idx, black_idx) = Self::generate_indexes(piece_type,
                                                                board.side_to_move(),
                                                                chess_move.get_dest(),
                                                                white_king,
                                                                black_king);
            result.added.push(white_idx);
            result.added.push(black_idx);
        }

        let (white_idx, black_idx) = Self::generate_indexes(piece_type,
                                                            board.side_to_move(),
                                                            chess_move.get_source(),
                                                            white_king,
                                                            black_king);
        result.removed.push(white_idx);
        result.removed.push(black_idx);

        let capture_type = board.piece_on(chess_move.get_dest());
        if capture_type.is_some() {
            let (white_idx, black_idx) = Self::generate_indexes(capture_type.unwrap(),
                                                                board.side_to_move(),
                                                                chess_move.get_dest(),
                                                                white_king,
                                                                black_king);
            result.removed.push(white_idx);
            result.removed.push(black_idx);
        }
        result
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

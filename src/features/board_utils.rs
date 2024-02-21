use chess::{BitBoard, Board, BoardStatus, Color, Piece, ALL_SQUARES, EMPTY};

#[inline]
pub fn status(board: &Board, any_legal_move: bool, insufficient_material: bool) -> BoardStatus {
    if !any_legal_move || insufficient_material {
        if *board.checkers() != EMPTY && !any_legal_move {
            return BoardStatus::Checkmate;
        }
        return BoardStatus::Stalemate;
    }
    return BoardStatus::Ongoing;
}

fn has_insufficient_material(board: &Board, color: Color) -> bool {
    // Pawns, rooks and queens are never insufficient material.
    let combined = board.color_combined(color);

    if (combined
        & (board.pieces(Piece::Pawn) | board.pieces(Piece::Rook) | board.pieces(Piece::Queen)))
        != EMPTY
    {
        return false;
    }

    // Knights are only insufficient material if:
    // (1) We do not have any other pieces, including more than one knight.
    // (2) The opponent does not have pawns, knights, bishops or rooks.
    //     These would allow self mate.
    if (combined & board.pieces(Piece::Knight)) != EMPTY {
        return combined.popcnt() <= 2
            && (board.color_combined(!color)
                & !board.pieces(Piece::King)
                & !board.pieces(Piece::Queen))
                == EMPTY;
    }

    // TODO: change both to const/static
    let DARK_BITBOARD: BitBoard = ALL_SQUARES
        .iter()
        .enumerate()
        .filter(|&(i, _)| i % 2 == 0)
        .fold(EMPTY, |acc, (_, sq)| acc | BitBoard::from_square(*sq));
    let LIGHT_BITBOARD = !DARK_BITBOARD;

    // Bishops are only insufficient material if:
    // (1) We do not have any other pieces, including bishops on the
    //     opposite color.
    // (2) The opponent does not have bishops on the opposite color,
    //      pawns or knights. These would allow self mate.
    if (combined & board.pieces(Piece::Bishop)) != EMPTY {
        let same_color = (board.pieces(Piece::Bishop) & DARK_BITBOARD) == EMPTY
            || (board.pieces(Piece::Bishop) & LIGHT_BITBOARD) == EMPTY;
        return same_color && (board.pieces(Piece::Knight) | board.pieces(Piece::Pawn)) == EMPTY;
    }

    true
}

pub fn is_insufficient_material(board: &Board) -> bool {
    return has_insufficient_material(board, Color::White)
        && has_insufficient_material(board, Color::Black);
}

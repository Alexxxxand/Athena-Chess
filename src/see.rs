use chess::{BitBoard, Board, ChessMove, Color, Piece, Square, get_bishop_moves, get_king_moves, get_knight_moves, get_pawn_attacks, get_rook_moves};
use crate::eval::piece_value;

/// Returns all pieces attacking `sq` on the board with `occupied` pieces
pub fn attackers_to(board: &Board, sq: Square, occupied: BitBoard) -> BitBoard {
    let white_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::White);
    let black_pawns = board.pieces(Piece::Pawn) & board.color_combined(Color::Black);
    let knights = board.pieces(Piece::Knight);
    let kings = board.pieces(Piece::King);
    let bishops_queens = board.pieces(Piece::Bishop) | board.pieces(Piece::Queen);
    let rooks_queens = board.pieces(Piece::Rook) | board.pieces(Piece::Queen);

    let pawn_attackers = (get_pawn_attacks(sq, Color::Black, !BitBoard(0)) & white_pawns)
        | (get_pawn_attacks(sq, Color::White, !BitBoard(0)) & black_pawns);
    let knight_attackers = get_knight_moves(sq) & knights;
    let king_attackers = get_king_moves(sq) & kings;
    let bishop_attackers = get_bishop_moves(sq, occupied) & bishops_queens;
    let rook_attackers = get_rook_moves(sq, occupied) & rooks_queens;

    pawn_attackers | knight_attackers | king_attackers | bishop_attackers | rook_attackers
}

/// Finds the least valuable attacker of `side` to square `sq`
fn get_least_valuable_attacker(
    board: &Board,
    attackers: BitBoard,
    side: Color,
) -> Option<(Square, Piece)> {
    let side_pieces = board.color_combined(side);
    let side_attackers = attackers & side_pieces;
    if side_attackers.0 == 0 {
        return None;
    }

    let pieces_order = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    for piece in pieces_order {
        let candidates = side_attackers & board.pieces(piece);
        if candidates.0 != 0 {
            let sq = candidates.into_iter().next().unwrap();
            return Some((sq, piece));
        }
    }

    None
}

/// Static Exchange Evaluation (SEE): evaluates material balance resulting from a series of captures on `mv.get_dest()`.
/// Returns expected centipawn score change (from perspective of side making the move).
pub fn see(board: &Board, mv: &ChessMove) -> i32 {
    let dest = mv.get_dest();
    let src = mv.get_source();
    let mut side = board.side_to_move();

    let mut gain = [0i32; 32];
    let mut d = 0;

    let victim = match board.piece_on(dest) {
        Some(p) => p,
        None => {
            // En passant or non-capture
            if board.piece_on(src) == Some(Piece::Pawn) && src.get_file() != dest.get_file() {
                Piece::Pawn
            } else {
                return 0;
            }
        }
    };

    gain[0] = piece_value(victim);
    let mut attacker_piece = board.piece_on(src).unwrap_or(Piece::Pawn);
    let mut occupied = board.combined() ^ BitBoard::from_square(src);

    let mut attackers = attackers_to(board, dest, occupied);

    side = !side;

    loop {
        d += 1;
        gain[d] = piece_value(attacker_piece) - gain[d - 1];

        if (-gain[d - 1]).max(gain[d]) < 0 {
            break;
        }

        match get_least_valuable_attacker(board, attackers & occupied, side) {
            Some((from_sq, next_piece)) => {
                attacker_piece = next_piece;
                occupied ^= BitBoard::from_square(from_sq);

                if next_piece == Piece::Pawn
                    || next_piece == Piece::Bishop
                    || next_piece == Piece::Rook
                    || next_piece == Piece::Queen
                {
                    let bishops_queens = board.pieces(Piece::Bishop) | board.pieces(Piece::Queen);
                    let rooks_queens = board.pieces(Piece::Rook) | board.pieces(Piece::Queen);
                    let bishop_xrays = get_bishop_moves(dest, occupied) & bishops_queens;
                    let rook_xrays = get_rook_moves(dest, occupied) & rooks_queens;
                    attackers |= bishop_xrays | rook_xrays;
                }

                side = !side;
            }
            None => break,
        }
    }

    while d > 1 {
        d -= 1;
        gain[d - 1] = -((-gain[d - 1]).max(gain[d]));
    }

    gain[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_see_simple_exchange() {
        // White pawn on e4 defended by d3 pawn. Black Knight on f6 captures e4.
        // 1... Nxe4 2. dxe4 -> White gains 320 - 100 = 220, so Black SEE should be negative (-220).
        let fen = "rnbqkb1r/pppppppp/5n2/8/4P3/3P4/PPP2PPP/RNBQKBNR b KQkq - 0 2";
        let board = Board::from_str(fen).unwrap();
        let mv = ChessMove::new(
            Square::make_square(chess::Rank::Sixth, chess::File::F),
            Square::make_square(chess::Rank::Fourth, chess::File::E),
            None,
        );
        let score = see(&board, &mv);
        assert!(score < 0, "Knight capturing defended pawn should have negative SEE, got {}", score);
    }

    #[test]
    fn test_see_winning_capture() {
        // White Rook on a1 captures completely undefended Black pawn on a4.
        let fen = "8/8/8/8/p7/8/8/R3K2k w - - 0 1";
        let board = Board::from_str(fen).unwrap();
        let mv = ChessMove::new(
            Square::make_square(chess::Rank::First, chess::File::A),
            Square::make_square(chess::Rank::Fourth, chess::File::A),
            None,
        );
        let score = see(&board, &mv);
        assert_eq!(score, 100, "Winning undefended pawn should give +100 SEE");
    }
}

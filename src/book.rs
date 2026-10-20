use chess::{Board, ChessMove};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::OnceLock;

static BOOK: OnceLock<HashMap<&'static str, &'static [&'static str]>> = OnceLock::new();

fn init_book() -> HashMap<&'static str, &'static [&'static str]> {
    let mut m = HashMap::new();

    // Initial position
    m.insert(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &["e2e4", "d2d4"][..],
    );

    // 1. e4
    m.insert(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
        &["e7e5", "c7c5", "e7e6", "c7c6"][..],
    );

    // 1. d4
    m.insert(
        "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1",
        &["d7d5", "g8f6"][..],
    );

    // 1. e4 e5
    m.insert(
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        &["g1f3", "b1c3"][..],
    );

    // 1. e4 e5 2. Nf3
    m.insert(
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        &["b8c6", "g8f6"][..],
    );

    // 1. e4 e5 2. Nf3 Nc6 (Italian Game, Ruy Lopez, Scotch Game)
    m.insert(
        "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
        &["f1c4", "f1b5", "d2d4"][..],
    );

    // 1. e4 e5 2. Nf3 Nc6 3. Bc4 (Italian Game)
    m.insert(
        "r1bqkbnr/pppp1ppp/2n5/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        &["f8c5", "g8f6"][..],
    );

    // 1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. c3
    m.insert(
        "r1bqk1nr/pppp1ppp/2n5/2b1p3/2B1P3/2P2N2/PP1P1PPP/RNBQK2R b KQkq - 0 4",
        &["g8f6"][..],
    );

    // 1. e4 e5 2. Nf3 Nc6 3. Bb5 (Ruy Lopez)
    m.insert(
        "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        &["a7a6", "g8f6"][..],
    );

    // 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4
    m.insert(
        "r1bqkbnr/1ppp1ppp/p1n5/4p3/B3P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 1 4",
        &["g8f6", "d7d6"][..],
    );

    // 1. e4 c5 (Sicilian Defense)
    m.insert(
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        &["g1f3", "b1c3"][..],
    );

    // 1. e4 c5 2. Nf3
    m.insert(
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        &["d7d6", "b8c6", "e7e6"][..],
    );

    // 1. e4 c5 2. Nf3 d6 3. d4
    m.insert(
        "rnbqkbnr/pp2pppp/3p4/2p5/3PP3/5N2/PPP2PPP/RNBQKB1R b KQkq - 0 3",
        &["c5d4"][..],
    );

    // 1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4
    m.insert(
        "rnbqkbnr/pp2pppp/3p4/8/3NP3/8/PPP2PPP/RNBQKB1R b KQkq - 0 4",
        &["g8f6"][..],
    );

    // 1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 (Najdorf, Dragon, Scheveningen)
    m.insert(
        "rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b KQkq - 2 5",
        &["a7a6", "g7g6", "e7e6"][..],
    );

    // 1. e4 e6 (French Defense)
    m.insert(
        "rnbqkbnr/pppp1ppp/4p3/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        &["d2d4"][..],
    );

    // 1. e4 e6 2. d4 d5
    m.insert(
        "rnbqkbnr/ppp2ppp/4p3/3p4/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3",
        &["b1c3", "e4e5", "b1d2"][..],
    );

    // 1. e4 c6 (Caro-Kann Defense)
    m.insert(
        "rnbqkbnr/pp1ppppp/2p5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        &["d2d4"][..],
    );

    // 1. e4 c6 2. d4 d5
    m.insert(
        "rnbqkbnr/pp2pppp/2p5/3p4/3PP3/8/PPP2PPP/RNBQKBNR w KQkq - 0 3",
        &["b1c3", "e4e5"][..],
    );

    // 1. e4 d5 (Scandinavian Defense)
    m.insert(
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        &["e4d5"][..],
    );

    // 1. d4 d5 2. c4 (Queen's Gambit)
    m.insert(
        "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b KQkq - 0 2",
        &["e7e6", "c7c6", "d5c4"][..],
    );

    // 1. d4 d5 2. c4 e6 3. Nc3
    m.insert(
        "rnbqkbnr/ppp2ppp/4p3/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR b KQkq - 1 3",
        &["g8f6", "c7c6"][..],
    );

    // 1. d4 Nf6 2. c4 (King's Indian / Nimzo-Indian)
    m.insert(
        "rnbqkb1r/pppppppp/5n2/8/2PP4/8/PP2PPPP/RNBQKBNR b KQkq - 0 2",
        &["g7g6", "e7e6"][..],
    );

    m
}

/// Checks opening book for a known move in the current position, randomly selecting among choices
pub fn get_book_move(board: &Board) -> Option<ChessMove> {
    let book = BOOK.get_or_init(init_book);
    let fen = format!("{}", board);

    if let Some(&moves) = book.get(fen.as_str()) {
        if !moves.is_empty() {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            let mut seed = board.get_hash() ^ nanos;
            seed ^= seed >> 12;
            seed ^= seed << 25;
            seed ^= seed >> 27;
            seed = seed.wrapping_mul(0x2545F4914F6CDD1D);

            let idx = (seed as usize) % moves.len();
            let chosen_str = moves[idx];
            if let Ok(mv) = ChessMove::from_str(chosen_str) {
                return Some(mv);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opening_book_returns_legal_moves() {
        let board = Board::default();
        let mv = get_book_move(&board);
        assert!(mv.is_some());
        let mv = mv.unwrap();
        let legal_moves: Vec<ChessMove> = chess::MoveGen::new_legal(&board).collect();
        assert!(legal_moves.contains(&mv), "Book move must be legal");
    }
}

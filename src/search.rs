use crate::eval::{self, piece_value};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct SearchLimits {
    pub max_depth: i32,
    pub time_budget: Option<Duration>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u64,
    pub depth: i8,
    pub flag: TTFlag,
    pub score: i32,
    pub best_move: Option<ChessMove>,
}

pub const TT_SIZE: usize = 1_048_576; // 1M entries (~24 MB)
pub const TT_MASK: usize = TT_SIZE - 1;

pub struct Searcher {
    pub stop_signal: Arc<AtomicBool>,
    pub start_time: Instant,
    pub time_budget: Option<Duration>,
    pub nodes: u64,
    pub tt: Vec<Option<TTEntry>>,
    pub killers: [[Option<ChessMove>; 2]; 64],
    pub history: [[[i32; 64]; 64]; 2],
    pub root_color: Color,
    pub pos_history: Vec<u64>,
}

impl Searcher {
    pub fn new(
        stop_signal: Arc<AtomicBool>,
        time_budget: Option<Duration>,
        game_history: &[u64],
        root_color: Color,
        tt: Vec<Option<TTEntry>>,
    ) -> Self {
        let mut pos_history = Vec::with_capacity(game_history.len() + 64);
        pos_history.extend_from_slice(game_history);

        Self {
            stop_signal,
            start_time: Instant::now(),
            time_budget,
            nodes: 0,
            tt,
            killers: [[None; 2]; 64],
            history: [[[0; 64]; 64]; 2],
            root_color,
            pos_history,
        }
    }

    #[inline]
    fn should_stop(&self) -> bool {
        if self.stop_signal.load(Ordering::Relaxed) {
            return true;
        }
        if self.nodes % 2048 == 0 {
            if let Some(budget) = self.time_budget {
                if self.start_time.elapsed() >= budget {
                    self.stop_signal.store(true, Ordering::Relaxed);
                    return true;
                }
            }
        }
        false
    }

    /// Checks for threefold repetition and cycles in the search tree.
    #[inline]
    pub fn is_repetition(&self, hash: u64, ply: i32) -> bool {
        if ply == 0 || self.pos_history.len() < 2 {
            return false;
        }

        let curr_idx = self.pos_history.len() - 1;
        let root_idx = curr_idx.saturating_sub(ply as usize);

        let mut i = curr_idx;
        let mut count = 0;

        // Positions can only repeat on the same side to move (step 2 plies).
        while i >= 2 {
            i -= 2;
            if self.pos_history[i] == hash {
                count += 1;
                // Cycle within the current search branch
                if i > root_idx {
                    return true;
                }
                // Repetition from game history
                if count >= 1 {
                    return true;
                }
            }
        }

        false
    }

    /// Evaluates draws with a contempt factor.
    /// From the root perspective, draws are penalized with -15 cp so the engine
    /// prefers playing for a win unless the position is worse than -15 cp.
    #[inline]
    pub fn draw_score(&self, board: &Board) -> i32 {
        const CONTEMPT: i32 = 15;
        if board.side_to_move() == self.root_color {
            -CONTEMPT
        } else {
            CONTEMPT
        }
    }
}

pub fn is_capture(board: &Board, mv: &ChessMove) -> bool {
    if board.piece_on(mv.get_dest()).is_some() {
        return true;
    }
    if let Some(Piece::Pawn) = board.piece_on(mv.get_source()) {
        if mv.get_source().get_file() != mv.get_dest().get_file() {
            return true;
        }
    }
    false
}

pub fn is_tactical(board: &Board, mv: &ChessMove) -> bool {
    is_capture(board, mv) || mv.get_promotion().is_some()
}

fn score_move(
    board: &Board,
    mv: &ChessMove,
    tt_move: Option<ChessMove>,
    killers: &[Option<ChessMove>; 2],
    history: &[[[i32; 64]; 64]; 2],
) -> i32 {
    // 1. TT move has highest priority
    if Some(*mv) == tt_move {
        return 200_000;
    }

    let mut score = 0;

    if let Some(prom) = mv.get_promotion() {
        score += piece_value(prom) * 20;
    }

    let attacker = board.piece_on(mv.get_source()).unwrap_or(Piece::Pawn);
    let victim = board.piece_on(mv.get_dest()).or_else(|| {
        if attacker == Piece::Pawn && mv.get_source().get_file() != mv.get_dest().get_file() {
            Some(Piece::Pawn)
        } else {
            None
        }
    });

    if let Some(vic) = victim {
        // 2. MVV-LVA (Most Valuable Victim - Least Valuable Aggressor)
        score += 50_000 + piece_value(vic) * 10 - piece_value(attacker);
    } else {
        // 3. Killer moves
        if killers[0] == Some(*mv) {
            score += 15_000;
        } else if killers[1] == Some(*mv) {
            score += 14_000;
        } else {
            // 4. History heuristic
            let c = match board.side_to_move() {
                chess::Color::White => 0,
                chess::Color::Black => 1,
            };
            let src = mv.get_source().to_index();
            let dst = mv.get_dest().to_index();
            score += history[c][src][dst].min(10_000);
        }
    }

    score
}

fn order_moves(
    board: &Board,
    moves: &mut [ChessMove],
    tt_move: Option<ChessMove>,
    killers: &[Option<ChessMove>; 2],
    history: &[[[i32; 64]; 64]; 2],
) {
    moves.sort_by_cached_key(|mv| -score_move(board, mv, tt_move, killers, history));
}

fn quiescence(searcher: &mut Searcher, board: &Board, mut alpha: i32, beta: i32) -> i32 {
    searcher.nodes += 1;

    if searcher.should_stop() {
        return 0;
    }

    let stand_pat = eval::evaluate(board);
    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    let mut tactical_moves: Vec<ChessMove> = MoveGen::new_legal(board)
        .filter(|mv| is_tactical(board, mv))
        .collect();

    order_moves(board, &mut tactical_moves, None, &[None, None], &searcher.history);

    for mv in tactical_moves {
        let new_board = board.make_move_new(mv);
        let score = -quiescence(searcher, &new_board, -beta, -alpha);

        if searcher.stop_signal.load(Ordering::Relaxed) {
            return 0;
        }

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}

fn negamax(
    searcher: &mut Searcher,
    board: &Board,
    depth: i32,
    ply: i32,
    mut alpha: i32,
    beta: i32,
    allow_null: bool,
) -> i32 {
    searcher.nodes += 1;

    if searcher.should_stop() {
        return 0;
    }

    if ply > 0 && searcher.is_repetition(board.get_hash(), ply) {
        return searcher.draw_score(board);
    }

    match board.status() {
        BoardStatus::Checkmate => {
            return -30000 + ply;
        }
        BoardStatus::Stalemate => return searcher.draw_score(board),
        BoardStatus::Ongoing => {}
    }

    let in_check = board.checkers().popcnt() > 0;
    let extension = if in_check && ply < 24 { 1 } else { 0 };

    if depth <= 0 && extension == 0 {
        return quiescence(searcher, board, alpha, beta);
    }

    // --- Null-Move Pruning (NMP) ---
    if allow_null && !in_check && depth >= 3 && ply > 0 && has_non_pawn_material(board) {
        if let Some(null_board) = board.null_move() {
            let r = 2; // Depth reduction
            let null_score = -negamax(searcher, &null_board, depth - 1 - r, ply + 1, -beta, -beta + 1, false);
            if null_score >= beta {
                return beta; // Pruned!
            }
        }
    }

    // --- Transposition Table probe ---
    let hash = board.get_hash();
    let tt_index = (hash as usize) & TT_MASK;
    let mut tt_move = None;

    if let Some(entry) = &searcher.tt[tt_index] {
        if entry.key == hash {
            tt_move = entry.best_move;

            if entry.depth >= depth as i8 && ply > 0 {
                let mut score = entry.score;
                if score > 20000 {
                    score -= ply;
                } else if score < -20000 {
                    score += ply;
                }

                match entry.flag {
                    TTFlag::Exact => return score,
                    TTFlag::LowerBound => {
                        if score >= beta {
                            return beta;
                        }
                    }
                    TTFlag::UpperBound => {
                        if score <= alpha {
                            return alpha;
                        }
                    }
                }
            }
        }
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if moves.is_empty() {
        return if in_check {
            -30000 + ply
        } else {
            searcher.draw_score(board)
        };
    }

    let alpha_orig = alpha;
    let current_killers = if (ply as usize) < 64 {
        searcher.killers[ply as usize]
    } else {
        [None, None]
    };

    order_moves(board, &mut moves, tt_move, &current_killers, &searcher.history);

    let mut best_move_node = None;
    let mut best_score = -50000;
    let mut moves_searched = 0;

    for mv in moves {
        moves_searched += 1;
        let new_board = board.make_move_new(mv);
        let is_tactical_move = is_tactical(board, &mv);
        let gives_check = new_board.checkers().popcnt() > 0;

        searcher.pos_history.push(new_board.get_hash());

        // --- PVS (Principal Variation Search) + Late Move Reductions (LMR) ---
        let score = if moves_searched == 1 {
            // Search PV move with full window [alpha, beta]
            -negamax(searcher, &new_board, depth - 1 + extension, ply + 1, -beta, -alpha, true)
        } else {
            // Search non-PV moves with null window [-alpha - 1, -alpha]
            let mut s = if moves_searched > 4
                && depth >= 3
                && !in_check
                && !is_tactical_move
                && !gives_check
                && current_killers[0] != Some(mv)
                && current_killers[1] != Some(mv)
            {
                // LMR search with reduced depth
                let red = -negamax(searcher, &new_board, depth - 2, ply + 1, -alpha - 1, -alpha, true);
                if red > alpha {
                    // Re-search at full depth with null window if reduced search failed high
                    -negamax(searcher, &new_board, depth - 1 + extension, ply + 1, -alpha - 1, -alpha, true)
                } else {
                    red
                }
            } else {
                -negamax(searcher, &new_board, depth - 1 + extension, ply + 1, -alpha - 1, -alpha, true)
            };

            // Re-search with full window if score exceeds alpha
            if s > alpha && s < beta {
                s = -negamax(searcher, &new_board, depth - 1 + extension, ply + 1, -beta, -alpha, true);
            }
            s
        };

        searcher.pos_history.pop();

        if searcher.stop_signal.load(Ordering::Relaxed) {
            return 0;
        }

        if score > best_score {
            best_score = score;
            best_move_node = Some(mv);
        }

        if score > alpha {
            alpha = score;
        }

        if score >= beta {
            // Beta-cutoff: store quiet killer move and update history heuristic
            if !is_tactical(board, &mv) {
                if (ply as usize) < 64 {
                    if searcher.killers[ply as usize][0] != Some(mv) {
                        searcher.killers[ply as usize][1] = searcher.killers[ply as usize][0];
                        searcher.killers[ply as usize][0] = Some(mv);
                    }
                }
                let c = match board.side_to_move() {
                    chess::Color::White => 0,
                    chess::Color::Black => 1,
                };
                let src = mv.get_source().to_index();
                let dst = mv.get_dest().to_index();
                searcher.history[c][src][dst] += depth * depth;
            }
            break;
        }
    }

    // --- Transposition Table storage with depth-preferred replacement ---
    if !searcher.stop_signal.load(Ordering::Relaxed) {
        let flag = if best_score >= beta {
            TTFlag::LowerBound
        } else if best_score > alpha_orig {
            TTFlag::Exact
        } else {
            TTFlag::UpperBound
        };

        let mut stored_score = best_score;
        if stored_score > 20000 {
            stored_score += ply;
        } else if stored_score < -20000 {
            stored_score -= ply;
        }

        let best_move_to_store = best_move_node.or_else(|| {
            if let Some(existing) = &searcher.tt[tt_index] {
                if existing.key == hash {
                    return existing.best_move;
                }
            }
            None
        });

        let should_replace = match &searcher.tt[tt_index] {
            None => true,
            Some(existing) => {
                existing.key != hash
                    || (depth as i8) >= existing.depth
                    || flag == TTFlag::Exact
            }
        };

        if should_replace {
            searcher.tt[tt_index] = Some(TTEntry {
                key: hash,
                depth: depth as i8,
                flag,
                score: stored_score,
                best_move: best_move_to_store,
            });
        }
    }

    best_score
}

fn has_non_pawn_material(board: &Board) -> bool {
    let side = board.side_to_move();
    let knights = (board.pieces(Piece::Knight) & board.color_combined(side)).popcnt();
    let bishops = (board.pieces(Piece::Bishop) & board.color_combined(side)).popcnt();
    let rooks = (board.pieces(Piece::Rook) & board.color_combined(side)).popcnt();
    let queens = (board.pieces(Piece::Queen) & board.color_combined(side)).popcnt();

    (knights + bishops + rooks + queens) > 0
}

pub fn search(
    board: &Board,
    game_history: &[u64],
    limits: SearchLimits,
    stop_signal: Arc<AtomicBool>,
    tt: Vec<Option<TTEntry>>,
) -> (ChessMove, Vec<Option<TTEntry>>) {
    let mut searcher = Searcher::new(
        stop_signal.clone(),
        limits.time_budget,
        game_history,
        board.side_to_move(),
        tt,
    );

    let mut legal_moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    if legal_moves.is_empty() {
        panic!("Search called on terminal board state");
    }

    let mut best_move = legal_moves[0];
    order_moves(board, &mut legal_moves, None, &[None, None], &searcher.history);

    let mut prev_score: i32 = 0;

    for depth in 1..=limits.max_depth {
        let mut best_move_this_iteration = None;
        let mut best_score_this_iteration = -50000;

        // Aspiration Windows: narrow search window around previous iteration score at depth >= 4
        let (mut alpha, mut beta) = if depth >= 4 && prev_score.abs() < 25000 {
            (prev_score - 50, prev_score + 50)
        } else {
            (-50000, 50000)
        };

        let mut root_moves_searched = 0;
        for &mv in &legal_moves {
            root_moves_searched += 1;
            let new_board = board.make_move_new(mv);
            searcher.pos_history.push(new_board.get_hash());

            let mut score = if root_moves_searched == 1 {
                -negamax(&mut searcher, &new_board, depth - 1, 1, -beta, -alpha, true)
            } else {
                let mut s = -negamax(&mut searcher, &new_board, depth - 1, 1, -alpha - 1, -alpha, true);
                if s > alpha && s < beta {
                    s = -negamax(&mut searcher, &new_board, depth - 1, 1, -beta, -alpha, true);
                }
                s
            };

            // Re-search with full window if aspiration window failed
            if score <= alpha && alpha > -50000 {
                alpha = -50000;
                score = -negamax(&mut searcher, &new_board, depth - 1, 1, -beta, -alpha, true);
            } else if score >= beta && beta < 50000 {
                beta = 50000;
                score = -negamax(&mut searcher, &new_board, depth - 1, 1, -beta, -alpha, true);
            }

            searcher.pos_history.pop();

            if stop_signal.load(Ordering::Relaxed) {
                break;
            }

            if score > best_score_this_iteration {
                best_score_this_iteration = score;
                best_move_this_iteration = Some(mv);
            }

            if score > alpha {
                alpha = score;
            }
        }

        if !stop_signal.load(Ordering::Relaxed) {
            if let Some(mv) = best_move_this_iteration {
                best_move = mv;
                prev_score = best_score_this_iteration;

                let elapsed = searcher.start_time.elapsed();
                let elapsed_ms = elapsed.as_millis().max(1);
                let nps = (searcher.nodes as u128 * 1000) / elapsed_ms;

                println!(
                    "info depth {} score cp {} nodes {} time {} nps {} pv {}",
                    depth, best_score_this_iteration, searcher.nodes, elapsed_ms, nps, best_move
                );

                if let Some(pos) = legal_moves.iter().position(|&m| m == best_move) {
                    legal_moves.swap(0, pos);
                }

                if best_score_this_iteration >= 25000 || best_score_this_iteration <= -25000 {
                    break;
                }
            }
        } else {
            break;
        }

        if let Some(budget) = limits.time_budget {
            if searcher.start_time.elapsed() >= budget / 2 {
                break;
            }
        }
    }

    (best_move, searcher.tt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_startpos_search() {
        let board = Board::default();
        let limits = SearchLimits {
            max_depth: 3,
            time_budget: None,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (best_move, _) = search(&board, &[board.get_hash()], limits, stop, vec![None; TT_SIZE]);
        println!("Found best move: {}", best_move);
    }

    #[test]
    fn test_mate_in_one() {
        let board = Board::from_str("r1bqkb1r/pppp1ppp/2n5/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 1").unwrap();
        let limits = SearchLimits {
            max_depth: 2,
            time_budget: None,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (best_move, _) = search(&board, &[board.get_hash()], limits, stop, vec![None; TT_SIZE]);
        assert_eq!(best_move, ChessMove::from_str("h5f7").unwrap());
        println!("Scholar's mate found: {}", best_move);
    }

    #[test]
    fn test_stop_dangerous_passed_pawn() {
        let board = Board::from_str("8/4k3/8/2r5/8/2R5/3p4/4K3 w - - 0 1").unwrap();
        let limits = SearchLimits {
            max_depth: 4,
            time_budget: None,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (best_move, _) = search(&board, &[board.get_hash()], limits, stop, vec![None; TT_SIZE]);
        assert_ne!(best_move, ChessMove::from_str("c3c5").unwrap());
    }

    #[test]
    fn test_queen_sacrifice_mate_in_two() {
        // White to play and mate in 2: 1. Qxf7+! Rxf7 2. Re8#
        let board = Board::from_str("r1b2rk1/1pp2ppp/p7/4R3/2B5/5Q2/PPP2PPP/R1B3K1 w - - 0 1").unwrap();
        let limits = SearchLimits {
            max_depth: 4,
            time_budget: None,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (best_move, _) = search(&board, &[board.get_hash()], limits, stop, vec![None; TT_SIZE]);
        println!("Sacrifice move found: {}", best_move);
        assert_eq!(best_move, ChessMove::from_str("f3f7").unwrap(), "Engine should find brilliant queen sacrifice Qxf7+!");
    }

    #[test]
    fn test_avoid_threefold_repetition_when_winning() {
        // White is winning and should deliver mate with b1b7# instead of repeating with b1b2
        let board = Board::from_str("k7/8/2K5/8/8/8/8/1Q6 w - - 0 1").unwrap();
        let board_after_b2 = board.make_move_new(ChessMove::from_str("b1b2").unwrap());

        let game_history = vec![
            board_after_b2.get_hash(),
            board.get_hash(),
        ];

        let limits = SearchLimits {
            max_depth: 3,
            time_budget: None,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (best_move, _) = search(&board, &game_history, limits, stop, vec![None; TT_SIZE]);
        println!("Winning move found: {}", best_move);
        assert_ne!(best_move, ChessMove::from_str("b1b2").unwrap());
        assert_eq!(best_move, ChessMove::from_str("b1b7").unwrap());
    }

    #[test]
    fn test_seek_threefold_repetition_when_losing() {
        // White is losing and must force a draw via threefold repetition with e8e5+
        let board = Board::from_str("4Q3/6k1/8/8/8/8/qr6/7K w - - 0 1").unwrap();
        let pos_after_e5 = board.make_move_new(ChessMove::from_str("e8e5").unwrap());

        let game_history = vec![
            pos_after_e5.get_hash(),
            board.get_hash(),
            pos_after_e5.get_hash(),
            board.get_hash(),
        ];

        let limits = SearchLimits {
            max_depth: 3,
            time_budget: None,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (best_move, _) = search(&board, &game_history, limits, stop, vec![None; TT_SIZE]);
        println!("Drawing move found when losing: {}", best_move);
        assert_eq!(best_move, ChessMove::from_str("e8e5").unwrap());
    }

    #[test]
    fn test_opponent_starts_repetition_engine_refuses_draw() {
        // Opponent offers repetition, engine refuses draw and plays checkmate Qg1-g7#
        let board = Board::from_str("7k/3R4/8/8/8/8/r7/6QK w - - 0 1").unwrap();
        let board_repeat = board.make_move_new(ChessMove::from_str("d7d1").unwrap());

        let game_history = vec![
            board_repeat.get_hash(),
            board.get_hash(),
        ];

        let limits = SearchLimits {
            max_depth: 3,
            time_budget: None,
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (best_move, _) = search(&board, &game_history, limits, stop, vec![None; TT_SIZE]);
        println!("Opponent repetition refused, move chosen: {}", best_move);
        assert_ne!(best_move, ChessMove::from_str("d7d1").unwrap(), "Should refuse opponent's repetition");
        assert_eq!(best_move, ChessMove::from_str("g1g7").unwrap(), "Should play checkmate Qg1-g7#");
    }

    #[test]
    fn test_zobrist_hash() {
        let b1 = Board::default();
        let b2 = Board::default();
        assert_eq!(b1.get_hash(), b2.get_hash());
    }
}

use crate::search::{self, SearchLimits, TTEntry};
use chess::{Board, ChessMove, Color};
use std::io::{self, BufRead};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct UciHandler {
    board: Board,
    history: Vec<u64>,
    tt: Option<Vec<Option<TTEntry>>>,
    stop_signal: Arc<AtomicBool>,
    search_thread: Option<thread::JoinHandle<Vec<Option<TTEntry>>>>,
}

impl UciHandler {
    pub fn new() -> Self {
        let board = Board::default();
        let history = vec![board.get_hash()];
        Self {
            board,
            history,
            tt: Some(vec![None; search::TT_SIZE]),
            stop_signal: Arc::new(AtomicBool::new(false)),
            search_thread: None,
        }
    }

    pub fn run(&mut self) {
        let stdin = io::stdin();
        let lines = stdin.lock().lines();

        for line_res in lines {
            let line = match line_res {
                Ok(l) => l,
                Err(_) => break,
            };

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let tokens: Vec<&str> = line.split_whitespace().collect();
            match tokens[0] {
                "uci" => {
                    println!("id name AthenaChess 0.1");
                    println!("id author Sanya");
                    println!("uciok");
                }
                "isready" => {
                    println!("readyok");
                }
                "ucinewgame" => {
                    self.stop_search();
                    self.board = Board::default();
                    self.history = vec![self.board.get_hash()];
                    if let Some(ref mut tt) = self.tt {
                        tt.fill(None);
                    }
                }
                "position" => {
                    self.handle_position(&tokens[1..]);
                }
                "go" => {
                    self.handle_go(&tokens[1..]);
                }
                "stop" => {
                    self.stop_search();
                }
                "quit" => {
                    self.stop_search();
                    break;
                }
                _ => {}
            }
        }
    }

    fn stop_search(&mut self) {
        self.stop_signal.store(true, Ordering::Relaxed);
        if let Some(handle) = self.search_thread.take() {
            if let Ok(returned_tt) = handle.join() {
                self.tt = Some(returned_tt);
            }
        }
    }

    fn handle_position(&mut self, tokens: &[&str]) {
        self.stop_search();

        if tokens.is_empty() {
            return;
        }

        let mut base_board = Board::default();
        let mut moves_idx: Option<usize> = None;

        if tokens[0] == "startpos" {
            base_board = Board::default();
            if tokens.len() > 1 && tokens[1] == "moves" {
                moves_idx = Some(2);
            }
        } else if tokens[0] == "fen" {
            // Parse 6-part FEN string
            let mut fen_parts = Vec::new();
            let mut idx = 1;
            while idx < tokens.len() && tokens[idx] != "moves" {
                fen_parts.push(tokens[idx]);
                idx += 1;
            }
            let fen = fen_parts.join(" ");
            match Board::from_str(&fen) {
                Ok(b) => base_board = b,
                Err(e) => {
                    eprintln!("Invalid FEN: {} ({:?})", fen, e);
                    return;
                }
            }
            if idx < tokens.len() && tokens[idx] == "moves" {
                moves_idx = Some(idx + 1);
            }
        }

        self.board = base_board;
        self.history = vec![self.board.get_hash()];

        if let Some(start) = moves_idx {
            for &mv_str in &tokens[start..] {
                match ChessMove::from_str(mv_str) {
                    Ok(mv) => {
                        if self.board.legal(mv) {
                            self.board = self.board.make_move_new(mv);
                            self.history.push(self.board.get_hash());
                        } else {
                            eprintln!("Illegal move: {}", mv_str);
                        }
                    }
                    Err(e) => {
                        eprintln!("Invalid move: {} ({:?})", mv_str, e);
                    }
                }
            }
        }
    }

    fn handle_go(&mut self, tokens: &[&str]) {
        self.stop_search();

        // Check opening book for instant response
        if let Some(book_move) = crate::book::get_book_move(&self.board) {
            println!("info depth 1 score cp 20 time 0 pv {}", book_move);
            println!("bestmove {}", book_move);
            return;
        }

        let mut depth: Option<i32> = None;
        let mut movetime: Option<u64> = None;
        let mut wtime: Option<u64> = None;
        let mut btime: Option<u64> = None;
        let mut winc: u64 = 0;
        let mut binc: u64 = 0;

        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "depth" if i + 1 < tokens.len() => {
                    depth = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "movetime" if i + 1 < tokens.len() => {
                    movetime = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "wtime" if i + 1 < tokens.len() => {
                    wtime = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "btime" if i + 1 < tokens.len() => {
                    btime = tokens[i + 1].parse().ok();
                    i += 2;
                }
                "winc" if i + 1 < tokens.len() => {
                    winc = tokens[i + 1].parse().unwrap_or(0);
                    i += 2;
                }
                "binc" if i + 1 < tokens.len() => {
                    binc = tokens[i + 1].parse().unwrap_or(0);
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        // Time management
        let time_budget = if let Some(mt) = movetime {
            Some(Duration::from_millis(mt.saturating_sub(20).max(10)))
        } else {
            let (my_time, my_inc) = match self.board.side_to_move() {
                Color::White => (wtime, winc),
                Color::Black => (btime, binc),
            };

            my_time.map(|time_left| {
                // Allocate ~1/30 of remaining time + half of increment
                let alloc = (time_left / 30) + (my_inc / 2);
                let safe_alloc = alloc.min(time_left.saturating_sub(50)).max(10);
                Duration::from_millis(safe_alloc)
            })
        };

        let max_depth = depth.unwrap_or(if time_budget.is_some() { 64 } else { 6 });

        let limits = SearchLimits {
            max_depth,
            time_budget,
        };

        self.stop_signal = Arc::new(AtomicBool::new(false));
        let stop_clone = self.stop_signal.clone();
        let board_clone = self.board;
        let history_clone = self.history.clone();
        let tt = self.tt.take().unwrap_or_else(|| vec![None; search::TT_SIZE]);

        // Run search in worker thread to keep main thread responsive to UCI commands
        self.search_thread = Some(thread::spawn(move || {
            let (best_move, returned_tt) = search::search(&board_clone, &history_clone, limits, stop_clone, tt);
            println!("bestmove {}", best_move);
            returned_tt
        }));
    }
}

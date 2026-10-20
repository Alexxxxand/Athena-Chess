use chess::{Board, Color, File, Piece, Rank, Square};

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 330;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20000;

pub fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => PAWN_VALUE,
        Piece::Knight => KNIGHT_VALUE,
        Piece::Bishop => BISHOP_VALUE,
        Piece::Rook => ROOK_VALUE,
        Piece::Queen => QUEEN_VALUE,
        Piece::King => KING_VALUE,
    }
}

#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     5,  10,  10, -20, -20,  10,  10,   5,
     5,  -5, -10,   0,   0, -10,  -5,   5,
     0,   0,   0,  20,  20,   0,   0,   0,
     5,   5,  10,  25,  25,  10,   5,   5,
    10,  10,  20,  30,  30,  20,  10,  10,
    50,  50,  50,  50,  50,  50,  50,  50,
     0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = [
   -50, -40, -30, -30, -30, -30, -40, -50,
   -40, -20,   0,   5,   5,   0, -20, -40,
   -30,   5,  10,  15,  15,  10,   5, -30,
   -30,   0,  15,  20,  20,  15,   0, -30,
   -30,   5,  15,  20,  20,  15,   5, -30,
   -30,   0,  10,  15,  15,  10,   0, -30,
   -40, -20,   0,   0,   0,   0, -20, -40,
   -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
   -20, -10, -10, -10, -10, -10, -10, -20,
   -10,   5,   0,   0,   0,   0,   5, -10,
   -10,  10,  10,  10,  10,  10,  10, -10,
   -10,   0,  10,  10,  10,  10,   0, -10,
   -10,   5,   5,  10,  10,   5,   5, -10,
   -10,   0,   5,  10,  10,   5,   0, -10,
   -10,   0,   0,   0,   0,   0,   0, -10,
   -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
     0,   0,   0,   5,   5,   0,   0,   0,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
     5,  10,  10,  10,  10,  10,  10,   5,
     0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
   -20, -10, -10,  -5,  -5, -10, -10, -20,
   -10,   0,   5,   0,   0,   0,   0, -10,
   -10,   5,   5,   5,   5,   5,   0, -10,
     0,   0,   5,   5,   5,   5,   0,  -5,
    -5,   0,   5,   5,   5,   5,   0,  -5,
   -10,   0,   5,   5,   5,   5,   0, -10,
   -10,   0,   0,   0,   0,   0,   0, -10,
   -20, -10, -10,  -5,  -5, -10, -10, -20,
];

#[rustfmt::skip]
const KING_TABLE_MG: [i32; 64] = [
    20,  30,  10,   0,   0,  10,  30,  20,
    20,  20,   0,   0,   0,   0,  20,  20,
   -10, -20, -20, -20, -20, -20, -20, -10,
   -20, -30, -30, -40, -40, -30, -30, -20,
   -30, -40, -40, -50, -50, -40, -40, -30,
   -30, -40, -40, -50, -50, -40, -40, -30,
   -30, -40, -40, -50, -50, -40, -40, -30,
   -30, -40, -40, -50, -50, -40, -40, -30,
];

#[rustfmt::skip]
const KING_TABLE_EG: [i32; 64] = [
   -50, -40, -30, -20, -20, -30, -40, -50,
   -30, -20, -10,   0,   0, -10, -20, -30,
   -30, -10,  20,  30,  30,  20, -10, -30,
   -30, -10,  30,  40,  40,  30, -10, -30,
   -30, -10,  30,  40,  40,  30, -10, -30,
   -30, -10,  20,  30,  30,  20, -10, -30,
   -30, -30,   0,   0,   0,   0, -30, -30,
   -50, -30, -30, -30, -30, -30, -30, -50,
];

#[inline]
fn king_pst(square_idx: usize, phase: i32) -> i32 {
    let mg = KING_TABLE_MG[square_idx];
    let eg = KING_TABLE_EG[square_idx];
    (mg * phase + eg * (24 - phase)) / 24
}

fn pst_value(piece: Piece, square_idx: usize) -> i32 {
    match piece {
        Piece::Pawn => PAWN_TABLE[square_idx],
        Piece::Knight => KNIGHT_TABLE[square_idx],
        Piece::Bishop => BISHOP_TABLE[square_idx],
        Piece::Rook => ROOK_TABLE[square_idx],
        Piece::Queen => QUEEN_TABLE[square_idx],
        Piece::King => 0,
    }
}

const FILE_MASKS: [u64; 8] = [
    0x0101010101010101,
    0x0202020202020202,
    0x0404040404040404,
    0x0808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080,
];

fn init_passed_masks() -> [[u64; 64]; 2] {
    let mut masks = [[0u64; 64]; 2];
    for sq_idx in 0..64 {
        let file = sq_idx % 8;
        let rank = sq_idx / 8;

        let min_f = if file > 0 { file - 1 } else { 0 };
        let max_f = if file < 7 { file + 1 } else { 7 };

        let mut w_mask = 0u64;
        for f in min_f..=max_f {
            for r in (rank + 1)..8 {
                w_mask |= 1u64 << (r * 8 + f);
            }
        }
        masks[0][sq_idx] = w_mask;

        let mut b_mask = 0u64;
        for f in min_f..=max_f {
            for r in 0..rank {
                b_mask |= 1u64 << (r * 8 + f);
            }
        }
        masks[1][sq_idx] = b_mask;
    }
    masks
}

#[inline]
fn is_passed_pawn(square_idx: usize, color: Color, enemy_pawns: u64) -> bool {
    use std::sync::OnceLock;
    static MASKS: OnceLock<[[u64; 64]; 2]> = OnceLock::new();
    let masks = MASKS.get_or_init(init_passed_masks);
    let c = match color {
        Color::White => 0,
        Color::Black => 1,
    };
    (enemy_pawns & masks[c][square_idx]) == 0
}

fn passed_pawn_bonus(rank_idx: usize, color: Color) -> i32 {
    let relative_rank = match color {
        Color::White => rank_idx,
        Color::Black => 7 - rank_idx,
    };
    match relative_rank {
        3 => 25,
        4 => 60,
        5 => 130,
        6 => 280,
        _ => 0,
    }
}

/// King safety and pawn shield evaluation
fn evaluate_king_safety(board: &Board, king_sq: Square, color: Color) -> i32 {
    let enemy_color = !color;
    let enemy_has_queen = (board.pieces(Piece::Queen) & board.color_combined(enemy_color)).popcnt() > 0;
    let enemy_has_heavy = enemy_has_queen || (board.pieces(Piece::Rook) & board.color_combined(enemy_color)).popcnt() > 0;
    if !enemy_has_heavy {
        return 0; // Without heavy pieces, back-rank threats are minimal
    }

    let mut score = 0;
    let rank = king_sq.get_rank();
    let file = king_sq.get_file();

    match color {
        Color::White => {
            // White kingside castle (g1, h1)
            if (file == File::G || file == File::H) && rank == Rank::First {
                let f2 = Square::make_square(Rank::Second, File::F);
                let g2 = Square::make_square(Rank::Second, File::G);
                let h2 = Square::make_square(Rank::Second, File::H);
                let h3 = Square::make_square(Rank::Third, File::H);

                if board.piece_on(f2) != Some(Piece::Pawn) || board.color_on(f2) != Some(Color::White) {
                    score -= 30; // Weak f2
                }
                if board.piece_on(g2) != Some(Piece::Pawn) || board.color_on(g2) != Some(Color::White) {
                    score -= 55; // Open g-file in front of king
                }

                // King's "Luft" (escape square)
                let has_h2 = board.piece_on(h2) == Some(Piece::Pawn) && board.color_on(h2) == Some(Color::White);
                let has_h3 = board.piece_on(h3) == Some(Piece::Pawn) && board.color_on(h3) == Some(Color::White);

                if has_h3 {
                    score += 25; // Bonus for safe Luft
                } else if !has_h2 {
                    score -= 20; // Missing h-pawn leaves flank exposed
                }
            } else if (file == File::B || file == File::C) && rank == Rank::First {
                // White queenside castle (b1, c1)
                let b2 = Square::make_square(Rank::Second, File::B);
                let a2 = Square::make_square(Rank::Second, File::A);
                let a3 = Square::make_square(Rank::Third, File::A);

                if board.piece_on(b2) != Some(Piece::Pawn) || board.color_on(b2) != Some(Color::White) {
                    score -= 40;
                }
                let has_a2 = board.piece_on(a2) == Some(Piece::Pawn) && board.color_on(a2) == Some(Color::White);
                let has_a3 = board.piece_on(a3) == Some(Piece::Pawn) && board.color_on(a3) == Some(Color::White);

                if has_a3 {
                    score += 20; // Queenside Luft (a3)
                } else if !has_a2 {
                    score -= 20;
                }
            } else if (file == File::D || file == File::E) && rank == Rank::First {
                // King trapped uncastled in center
                score -= 60;
            }
        }
        Color::Black => {
            // Black kingside castle (g8, h8)
            if (file == File::G || file == File::H) && rank == Rank::Eighth {
                let f7 = Square::make_square(Rank::Seventh, File::F);
                let g7 = Square::make_square(Rank::Seventh, File::G);
                let h7 = Square::make_square(Rank::Seventh, File::H);
                let h6 = Square::make_square(Rank::Sixth, File::H);

                if board.piece_on(f7) != Some(Piece::Pawn) || board.color_on(f7) != Some(Color::Black) {
                    score -= 30;
                }
                if board.piece_on(g7) != Some(Piece::Pawn) || board.color_on(g7) != Some(Color::Black) {
                    score -= 55;
                }

                // Black's Luft (escape square)
                let has_h7 = board.piece_on(h7) == Some(Piece::Pawn) && board.color_on(h7) == Some(Color::Black);
                let has_h6 = board.piece_on(h6) == Some(Piece::Pawn) && board.color_on(h6) == Some(Color::Black);

                if has_h6 {
                    score += 25; // Bonus for h6 Luft
                } else if !has_h7 {
                    score -= 20;
                }
            } else if (file == File::B || file == File::C) && rank == Rank::Eighth {
                // Black queenside castle
                let b7 = Square::make_square(Rank::Seventh, File::B);
                let a7 = Square::make_square(Rank::Seventh, File::A);
                let a6 = Square::make_square(Rank::Sixth, File::A);

                if board.piece_on(b7) != Some(Piece::Pawn) || board.color_on(b7) != Some(Color::Black) {
                    score -= 40;
                }
                let has_a7 = board.piece_on(a7) == Some(Piece::Pawn) && board.color_on(a7) == Some(Color::Black);
                let has_a6 = board.piece_on(a6) == Some(Piece::Pawn) && board.color_on(a6) == Some(Color::Black);

                if has_a6 {
                    score += 20; // Queenside Luft (a6)
                } else if !has_a7 {
                    score -= 20;
                }
            } else if (file == File::D || file == File::E) && rank == Rank::Eighth {
                // King trapped in center
                score -= 60;
            }
        }
    }

    score
}

/// Evaluates rooks on open and semi-open files
fn evaluate_rooks(board: &Board, color: Color, our_pawns: u64, enemy_pawns: u64) -> i32 {
    let mut bonus = 0;
    let our_rooks = board.pieces(Piece::Rook) & board.color_combined(color);

    for sq in our_rooks {
        let f = sq.get_file().to_index();
        let mask = FILE_MASKS[f];
        let has_our_pawn = (our_pawns & mask) != 0;
        let has_enemy_pawn = (enemy_pawns & mask) != 0;

        if !has_our_pawn && !has_enemy_pawn {
            bonus += 35; // Open file
        } else if !has_our_pawn && has_enemy_pawn {
            bonus += 20; // Semi-open file against enemy pawn
        }
    }
    bonus
}

/// Pawn structure defects: doubled and isolated pawns
fn evaluate_pawn_structure(board: &Board, color: Color) -> i32 {
    let mut penalty = 0;
    let mut file_counts = [0; 8];

    let pawns = board.pieces(Piece::Pawn) & board.color_combined(color);
    for sq in pawns {
        file_counts[sq.get_file().to_index()] += 1;
    }

    for f in 0..8 {
        // Doubled pawns
        if file_counts[f] > 1 {
            penalty += (file_counts[f] - 1) * 25;
        }
        // Isolated pawn
        if file_counts[f] > 0 {
            let has_left = f > 0 && file_counts[f - 1] > 0;
            let has_right = f < 7 && file_counts[f + 1] > 0;
            if !has_left && !has_right {
                penalty += 20;
            }
        }
    }

    penalty
}

/// Opening development harmony: penalty for undeveloped minor pieces and premature queen excursions
fn evaluate_development(board: &Board, color: Color, total_pieces: usize) -> i32 {
    if total_pieces < 24 {
        return 0; // Inactive outside opening phase
    }

    let mut penalty = 0;

    match color {
        Color::White => {
            let b1 = Square::make_square(Rank::First, File::B);
            let g1 = Square::make_square(Rank::First, File::G);
            let c1 = Square::make_square(Rank::First, File::C);
            let f1 = Square::make_square(Rank::First, File::F);

            let mut asleep = 0;
            if board.piece_on(b1) == Some(Piece::Knight) && board.color_on(b1) == Some(Color::White) { asleep += 1; }
            if board.piece_on(g1) == Some(Piece::Knight) && board.color_on(g1) == Some(Color::White) { asleep += 1; }
            if board.piece_on(c1) == Some(Piece::Bishop) && board.color_on(c1) == Some(Color::White) { asleep += 1; }
            if board.piece_on(f1) == Some(Piece::Bishop) && board.color_on(f1) == Some(Color::White) { asleep += 1; }

            penalty += asleep * 15;

            // Penalty if queen wanders while 3+ minor pieces sleep
            let d1 = Square::make_square(Rank::First, File::D);
            if asleep >= 3 && board.piece_on(d1) != Some(Piece::Queen) {
                penalty += 35;
            }
        }
        Color::Black => {
            let b8 = Square::make_square(Rank::Eighth, File::B);
            let g8 = Square::make_square(Rank::Eighth, File::G);
            let c8 = Square::make_square(Rank::Eighth, File::C);
            let f8 = Square::make_square(Rank::Eighth, File::F);

            let mut asleep = 0;
            if board.piece_on(b8) == Some(Piece::Knight) && board.color_on(b8) == Some(Color::Black) { asleep += 1; }
            if board.piece_on(g8) == Some(Piece::Knight) && board.color_on(g8) == Some(Color::Black) { asleep += 1; }
            if board.piece_on(c8) == Some(Piece::Bishop) && board.color_on(c8) == Some(Color::Black) { asleep += 1; }
            if board.piece_on(f8) == Some(Piece::Bishop) && board.color_on(f8) == Some(Color::Black) { asleep += 1; }

            penalty += asleep * 15;

            let d8 = Square::make_square(Rank::Eighth, File::D);
            if asleep >= 3 && board.piece_on(d8) != Some(Piece::Queen) {
                penalty += 35;
            }
        }
    }

    penalty
}

fn evaluate_mating_endgame(
    winning_king: Square,
    losing_king: Square,
    winning_material: i32,
    losing_material: i32,
) -> i32 {
    if winning_material < losing_material + 400 || losing_material > 500 {
        return 0;
    }

    let mut bonus = 0;

    let lose_f = losing_king.get_file().to_index() as i32;
    let lose_r = losing_king.get_rank().to_index() as i32;
    let f_dist = (3 - lose_f).abs().max((lose_f - 4).abs());
    let r_dist = (3 - lose_r).abs().max((lose_r - 4).abs());
    let center_dist = f_dist + r_dist;
    bonus += center_dist * 45;

    let win_f = winning_king.get_file().to_index() as i32;
    let win_r = winning_king.get_rank().to_index() as i32;
    let king_dist = (win_f - lose_f).abs().max((win_r - lose_r).abs());
    bonus += (7 - king_dist) * 25;

    bonus
}

pub fn evaluate(board: &Board) -> i32 {
    let mut white_score = 0;
    let mut black_score = 0;
    let mut white_material = 0;
    let mut black_material = 0;
    let mut total_pieces = 0;

    let mut white_king_sq = Square::make_square(Rank::First, File::E);
    let mut black_king_sq = Square::make_square(Rank::Eighth, File::E);

    let white_pawns = (board.pieces(Piece::Pawn) & board.color_combined(Color::White)).0;
    let black_pawns = (board.pieces(Piece::Pawn) & board.color_combined(Color::Black)).0;

    // Calculate game phase from non-pawn material (24 = full middlegame, 0 = pure pawn endgame)
    let knights = board.pieces(Piece::Knight).popcnt() as i32;
    let bishops = board.pieces(Piece::Bishop).popcnt() as i32;
    let rooks = board.pieces(Piece::Rook).popcnt() as i32;
    let queens = board.pieces(Piece::Queen).popcnt() as i32;
    let phase = ((knights * 1) + (bishops * 1) + (rooks * 2) + (queens * 4)).min(24);

    for square_idx in 0..64 {
        let square = unsafe { Square::new(square_idx as u8) };

        if let Some(piece) = board.piece_on(square) {
            total_pieces += 1;
            match board.color_on(square) {
                Some(Color::White) => {
                    let val = piece_value(piece);
                    white_score += val;

                    if piece != Piece::King {
                        white_material += val;
                        white_score += pst_value(piece, square_idx);
                    } else {
                        white_score += king_pst(square_idx, phase);
                        white_king_sq = square;
                    }

                    if piece == Piece::Pawn && is_passed_pawn(square_idx, Color::White, black_pawns) {
                        white_score += passed_pawn_bonus(square.get_rank().to_index(), Color::White);
                    }
                }
                Some(Color::Black) => {
                    let val = piece_value(piece);
                    black_score += val;
                    let flipped_idx = square_idx ^ 56;

                    if piece != Piece::King {
                        black_material += val;
                        black_score += pst_value(piece, flipped_idx);
                    } else {
                        black_score += king_pst(flipped_idx, phase);
                        black_king_sq = square;
                    }

                    if piece == Piece::Pawn && is_passed_pawn(square_idx, Color::Black, white_pawns) {
                        black_score += passed_pawn_bonus(square.get_rank().to_index(), Color::Black);
                    }
                }
                None => {}
            }
        }
    }

    // Bishop pair bonus (+50)
    let white_bishops = (board.pieces(Piece::Bishop) & board.color_combined(Color::White)).popcnt();
    if white_bishops >= 2 {
        white_score += 50;
    }
    let black_bishops = (board.pieces(Piece::Bishop) & board.color_combined(Color::Black)).popcnt();
    if black_bishops >= 2 {
        black_score += 50;
    }

    // Penalty for blocking d2/e2 central pawns with bishops
    let d3 = Square::make_square(Rank::Third, File::D);
    let d2 = Square::make_square(Rank::Second, File::D);
    if board.piece_on(d3) == Some(Piece::Bishop) && board.color_on(d3) == Some(Color::White)
        && board.piece_on(d2) == Some(Piece::Pawn) && board.color_on(d2) == Some(Color::White) {
        white_score -= 40;
    }

    let e3 = Square::make_square(Rank::Third, File::E);
    let e2 = Square::make_square(Rank::Second, File::E);
    if board.piece_on(e3) == Some(Piece::Bishop) && board.color_on(e3) == Some(Color::White)
        && board.piece_on(e2) == Some(Piece::Pawn) && board.color_on(e2) == Some(Color::White) {
        white_score -= 40;
    }

    let d6 = Square::make_square(Rank::Sixth, File::D);
    let d7 = Square::make_square(Rank::Seventh, File::D);
    if board.piece_on(d6) == Some(Piece::Bishop) && board.color_on(d6) == Some(Color::Black)
        && board.piece_on(d7) == Some(Piece::Pawn) && board.color_on(d7) == Some(Color::Black) {
        black_score -= 40;
    }

    let e6 = Square::make_square(Rank::Sixth, File::E);
    let e7 = Square::make_square(Rank::Seventh, File::E);
    if board.piece_on(e6) == Some(Piece::Bishop) && board.color_on(e6) == Some(Color::Black)
        && board.piece_on(e7) == Some(Piece::Pawn) && board.color_on(e7) == Some(Color::Black) {
        black_score -= 40;
    }

    // Positional evaluation terms:
    // 1. King safety and pawn shield (scaled by game phase)
    let white_safety = evaluate_king_safety(board, white_king_sq, Color::White);
    let black_safety = evaluate_king_safety(board, black_king_sq, Color::Black);
    white_score += (white_safety * phase) / 24;
    black_score += (black_safety * phase) / 24;

    // 2. Open and semi-open files for rooks
    white_score += evaluate_rooks(board, Color::White, white_pawns, black_pawns);
    black_score += evaluate_rooks(board, Color::Black, black_pawns, white_pawns);

    // 3. Doubled and isolated pawn penalties
    white_score -= evaluate_pawn_structure(board, Color::White);
    black_score -= evaluate_pawn_structure(board, Color::Black);

    // 4. Opening development harmony
    white_score -= evaluate_development(board, Color::White, total_pieces);
    black_score -= evaluate_development(board, Color::Black, total_pieces);

    // 5. Endgame mating king-drive
    if white_material > black_material {
        white_score += evaluate_mating_endgame(white_king_sq, black_king_sq, white_material, black_material);
    } else if black_material > white_material {
        black_score += evaluate_mating_endgame(black_king_sq, white_king_sq, black_material, white_material);
    }

    let score = white_score - black_score;

    match board.side_to_move() {
        Color::White => score,
        Color::Black => -score,
    }
}

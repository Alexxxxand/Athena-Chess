<p align="center">
  <img src="assets/logo.png" alt="Athena Chess Engine" width="180">
</p>

<h1 align="center">Athena Chess Engine</h1>

<p align="center">
  <b>A fast, tactical UCI chess engine written in Rust.</b>
</p>

<p align="center">
  <a href="https://lichess.org/@/Athena-ChessBot">Play on Lichess (@Athena-ChessBot)</a>
</p>

## Overview

Athena is an efficient, tactically sharp chess engine written in Rust that communicates via the standard UCI (Universal Chess Interface) protocol.

You can challenge and play against the engine live on Lichess: [@Athena-ChessBot](https://lichess.org/@/Athena-ChessBot).

## Features

- **Search Architecture:**
  - Iterative Deepening with dynamic Aspiration Windows
  - Principal Variation Search (PVS / NegaScout) with null-window search and full re-search
  - Static Exchange Evaluation (SEE) pruning in Quiescence Search
  - Reverse Futility Pruning (RFP) & Futility Pruning at shallow depths
  - Late Move Reductions (LMR) for quiet non-tactical moves
  - Persistent Transposition Table (24 MB, 1M entries) across moves with depth-preferred replacement
  - Killer Move Heuristic & History Heuristic
  - SEE-enhanced MVV-LVA move ordering
  - Threefold Repetition & 50-move draw detection with Contempt factor

- **Evaluation:**
  - Tapered Evaluation (Middlegame vs Endgame phase interpolation)
  - Safe Piece Mobility evaluation for Knights, Bishops, and Rooks
  - Knight Outpost bonuses on central files
  - Precomputed 64-bit Bitboard evaluation for Passed and Connected Pawns
  - Open & Semi-open Rook File bonuses
  - Bishop pair bonuses and development harmony penalties
  - King Safety with pawn shield & Luft detection
  - Endgame mating king-drive (driving opponent king to edges/corners)
  - Built-in randomized opening book

- **Speed:**
  - ~3.7 - 4.0 Million NPS (Nodes Per Second) in release mode
  - Optimized release profile with Link-Time Optimization (LTO = "fat")

## Building

Requires Rust:

```bash
cargo build --release
```

The compiled binary will be located at `target/release/athena-chess` (or `.exe` on Windows).

## UCI Usage

Athena implements the standard UCI protocol and works with any chess GUI (Arena, Cutechess, Banksia, Lichess-bot, etc.):

```bash
./athena-chess
```

```text
uci
isready
ucinewgame
position startpos moves e2e4 e7e5
go wtime 300000 btime 300000
```

## Artwork Credit

Logo artwork: Athena by [Cynthia Sheppard](https://www.cynthia-sheppard.com/athena). Used with appreciation.

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

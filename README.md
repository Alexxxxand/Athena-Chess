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
  - Iterative Deepening with Aspiration Windows
  - Principal Variation Search (PVS / NegaScout) with zero-window search
  - Quiescence Search with Stand-pat pruning
  - Late Move Reductions (LMR) for late quiet branches
  - Persistent Transposition Table (24 MB, 1M entries) across moves with depth-preferred replacement
  - Killer Move Heuristic & History Heuristic
  - MVV-LVA (Most Valuable Victim - Least Valuable Aggressor) move ordering
  - Threefold Repetition & 50-move draw detection with Contempt factor (avoids repetition when winning, forces repetition when losing)

- **Evaluation:**
  - Tapered Evaluation (Middlegame vs Endgame phase interpolation)
  - Active King centralisation in endgames & King Safety / Pawn Shield in middlegames
  - Precomputed 64-bit Bitboard evaluation for Passed Pawns and Open/Semi-open Rook Files
  - Bishop pair bonuses and development harmony penalties
  - Mating endgame king drive (driving opponent king to edges/corners)
  - Built-in opening book

- **Speed:**
  - ~3.7 - 4.0 Million NPS (Nodes Per Second) in release mode
  - Optimized release profile with Link-Time Optimization (LTO = "fat")

## Building

Requires Rust (edition 2021):

```bash
cargo build --release
```

The compiled binary will be located at `target/release/athena-chess` (or `.exe` on Windows).

## UCI Usage

Athena implements the standard UCI protocol and works with any chess GUI (Arena, Cutechess, Banksia, Lichess-bot, etc.):

```bash
./athena-chess
uci
isready
position startpos moves e2e4 e7e5
go movetime 1000
```

## Credits

- Logo artwork: *Athena* by [Cynthia Sheppard](https://www.cynthia-sheppard.com/athena).

## License

GNU General Public License v3.0 (see [LICENSE](LICENSE)).

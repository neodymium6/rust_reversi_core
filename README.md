# rust_reversi_core

A Rust library for the game of Reversi (Othello) including game engine, AI players, and arena for playing games.

This is also the core implementation for [rust_reversi](https://github.com/neodymium6/rust_reversi).

See also the [documentation](https://docs.rs/rust_reversi_core).

## Overview

This project provides:

- Complete Reversi game rule engine
- Multiple AI player implementations
- Arena for playing games (both local and network)
- Alpha-beta search engine implementation

## Features

### Board Engine

- 8x8 Reversi board management
- Legal move validation
- Move execution and piece flipping
- Pass detection
- Win condition checking
- Fast bitboard-based implementation

### AI Players

Multiple AI strategies are implemented:

- Random Player - Makes random legal moves
- Piece Evaluator - Evaluates based on piece count difference
- Matrix Evaluator - Uses position weights for evaluation

You can also use your own Evaluator that implements the `Evaluator` trait.

### Arena Features

- Local game support
- Network play over TCP/IP
- Automatic execution of multiple games between players
- Statistics collection (win rates, piece counts)
- Progress bar visualization

### Search Engine

- Alpha-beta pruning implementation
- Iterative deepening
- Timeout control
- Pluggable evaluation functions

## Installation

```bash
cargo add rust_reversi_core
```

## Usage

Basic usage:

```rust
use rust_reversi_core::board::Board;

// Create a new board
let mut board = Board::new();

// Get legal moves
let legal_moves = board.get_legal_moves_vec();

// Make a move
board.do_move(legal_moves[0]).unwrap();
```

Using AI players:

```rust
use rust_reversi_core::search::{AlphaBetaSearch, MatrixEvaluator};

// Setup evaluator and search
let evaluator = MatrixEvaluator::new(matrix);
let search = AlphaBetaSearch::new(depth, Arc::new(evaluator));

// Get best move
let best_move = search.get_move(&board);
```

## Arena Usage

Running local games:

```rust
use rust_reversi_core::arena::LocalArena;

let mut arena = LocalArena::new(command1, command2, true);
arena.play_n(100).unwrap();
let (wins1, wins2, draws) = arena.get_stats();
```

Network games:

```rust
use rust_reversi_core::arena::{NetworkArenaServer, NetworkArenaClient};

// Server
let mut server = NetworkArenaServer::new(100, true).unwrap();
server.start("127.0.0.1".to_string(), 12345).unwrap();

// Client
let mut client = NetworkArenaClient::new(command);
client.connect("127.0.0.1".to_string(), 12345).unwrap();
```

## Project Structure

- `src/board.rs` - Core game logic and board representation
- `src/search/` - Search algorithms and evaluation functions
- `src/arena/` - Local and network game coordination
- `tests/` - Test cases and example players

## Testing

Run the test suite:

```bash
cargo test
```

The test suite includes both unit tests and integration tests with example AI players.

## Benchmarking

The project includes benchmarks for core functionality using Criterion:

```bash
cargo bench
```

Available benchmarks:

- Board operations (`board` benchmark)
  - Full game playthrough with random moves
  
- Search algorithms (`search` benchmark)
  - Alpha-beta search with various evaluators (depth 4):
    - Piece count evaluator
    - Legal moves evaluator
    - Matrix-based evaluator
    - Custom evaluator example

Each evaluator is tested with a small probability (ε=0.01) of making random moves to add variety.

## License

MIT License

## Author

neodymium6

## Contributing

Contributions are welcome! Feel free to:

- Report bugs
- Suggest features
- Submit pull requests

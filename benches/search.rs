use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use rust_reversi_core::board::Board;
use rust_reversi_core::search::AlphaBetaSearch;
use rust_reversi_core::search::MatrixEvaluator;
use rust_reversi_core::search::PieceEvaluator;

const EPSILON: f64 = 1e-2;

fn alpha_beta4_piece() {
    let search = AlphaBetaSearch::new(4, Box::new(PieceEvaluator::new()));
    let mut board = Board::new();
    while !board.is_game_over() {
        if board.is_pass() {
            board.do_pass().unwrap();
        } else {
            let m = if rand::thread_rng().gen_bool(EPSILON) {
                board.get_random_move().unwrap()
            } else {
                search.get_move(&board).unwrap()
            };
            board.do_move(m).unwrap();
        }
    }
}

fn alpha_beta4_matrix() {
    let matrix = black_box([
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1, 1, 1, 1],
    ]);
    let evaluator = MatrixEvaluator::new(matrix);
    let search = AlphaBetaSearch::new(4, Box::new(evaluator));
    let mut board = Board::new();
    while !board.is_game_over() {
        if board.is_pass() {
            board.do_pass().unwrap();
        } else {
            let m = if rand::thread_rng().gen_bool(EPSILON) {
                board.get_random_move().unwrap()
            } else {
                search.get_move(&board).unwrap()
            };
            board.do_move(m).unwrap();
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("alpha_beta4_piece", |b| b.iter(alpha_beta4_piece));
    c.bench_function("alpha_beta4_matrix", |b| b.iter(alpha_beta4_matrix));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

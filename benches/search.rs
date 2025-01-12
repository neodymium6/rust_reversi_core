use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use rust_reversi_core::board::Board;
use rust_reversi_core::search::AlphaBetaSearch;
use rust_reversi_core::search::Evaluator;
use rust_reversi_core::search::LegalNumEvaluator;
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

fn alpha_beta4_legal_num() {
    let search = AlphaBetaSearch::new(4, Box::new(LegalNumEvaluator::new()));
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

struct CustomEvaluator {}
impl Evaluator for CustomEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        board.diff_piece_num() + board.get_legal_moves().count_ones() as i32
    }
}

fn alpha_beta4_custom() {
    let search = AlphaBetaSearch::new(4, Box::new(CustomEvaluator {}));
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
    c.bench_function("alpha_beta4_legal_num", |b| b.iter(alpha_beta4_legal_num));
    c.bench_function("alpha_beta4_matrix", |b| b.iter(alpha_beta4_matrix));
    c.bench_function("alpha_beta4_custom", |b| b.iter(alpha_beta4_custom));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

use criterion::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use rust_reversi_core::board::Board;
use rust_reversi_core::search::AlphaBetaSearch;
use rust_reversi_core::search::BitMatrixEvaluator;
use rust_reversi_core::search::Evaluator;
use rust_reversi_core::search::LegalNumEvaluator;
use rust_reversi_core::search::MatrixEvaluator;
use rust_reversi_core::search::PieceEvaluator;

const EPSILON: f64 = 1e-2;

fn play_with_search(search: &AlphaBetaSearch) {
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

fn get_alpha_beta4_piece() -> AlphaBetaSearch {
    AlphaBetaSearch::new(4, Box::new(PieceEvaluator::new()))
}

fn get_alpha_beta4_legal_num() -> AlphaBetaSearch {
    AlphaBetaSearch::new(4, Box::new(LegalNumEvaluator::new()))
}

fn get_alpha_beta4_matrix() -> AlphaBetaSearch {
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
    AlphaBetaSearch::new(4, Box::new(evaluator))
}

struct CustomEvaluator {}
impl Evaluator for CustomEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        board.diff_piece_num() + board.get_legal_moves().count_ones() as i32
    }
}

fn get_alpha_beta4_custom() -> AlphaBetaSearch {
    AlphaBetaSearch::new(4, Box::new(CustomEvaluator {}))
}

fn get_alpha_beta4_bitmatrix5() -> AlphaBetaSearch {
    let masks: Vec<u64> = black_box(vec![
        0x0000001818000000,
        0x0000182424180000,
        0x0000240000240000,
        0x0018004242001800,
        0x0024420000422400,
    ]);
    let weights: Vec<i32> = black_box(vec![-1, 1, 1, -2, -2]);
    let evaluator = BitMatrixEvaluator::<5>::new(weights, masks);
    AlphaBetaSearch::new(4, Box::new(evaluator))
}

fn get_alpha_beta4_bitmatrix5n() -> AlphaBetaSearch {
    let masks: Vec<u64> = black_box(vec![
        0x0000001818000000,
        0x0000182424180000,
        0x0000240000240000,
        0x0018004242001800,
        0x0024420000422400,
    ]);
    let weights: Vec<i32> = black_box(vec![-1, -1, -1, -2, -2]);
    let evaluator = BitMatrixEvaluator::<5>::new(weights, masks);
    AlphaBetaSearch::new(4, Box::new(evaluator))
}

fn get_alpha_beta4_bitmatrix10() -> AlphaBetaSearch {
    let masks: Vec<u64> = black_box(vec![
        0x0000001818000000,
        0x0000182424180000,
        0x0000240000240000,
        0x0018004242001800,
        0x0024420000422400,
        0x0042000000004200,
        0x1800008181000018,
        0x2400810000810024,
        0x4281000000008142,
        0x8100000000000081,
    ]);
    let weights: Vec<i32> = black_box(vec![-1, -1, -1, -2, -2, -50, 5, 10, -20, 100]);
    let evaluator = BitMatrixEvaluator::<10>::new(weights, masks);
    AlphaBetaSearch::new(4, Box::new(evaluator))
}

fn criterion_benchmark(c: &mut Criterion) {
    let alpha_beta4_piece = get_alpha_beta4_piece();
    let alpha_beta4_legal_num = get_alpha_beta4_legal_num();
    let alpha_beta4_matrix = get_alpha_beta4_matrix();
    let alpha_beta4_custom = get_alpha_beta4_custom();
    let alpha_beta4_bitmatrix5 = get_alpha_beta4_bitmatrix5();
    let alpha_beta4_bitmatrix5n = get_alpha_beta4_bitmatrix5n();
    let alpha_beta4_bitmatrix10 = get_alpha_beta4_bitmatrix10();

    c.bench_function("alpha_beta4_piece", |b| {
        b.iter(|| play_with_search(&alpha_beta4_piece))
    });
    c.bench_function("alpha_beta4_legal_num", |b| {
        b.iter(|| play_with_search(&alpha_beta4_legal_num))
    });
    c.bench_function("alpha_beta4_matrix", |b| {
        b.iter(|| play_with_search(&alpha_beta4_matrix))
    });
    c.bench_function("alpha_beta4_custom", |b| {
        b.iter(|| play_with_search(&alpha_beta4_custom))
    });
    c.bench_function("alpha_beta4_bitmatrix5", |b| {
        b.iter(|| play_with_search(&alpha_beta4_bitmatrix5))
    });
    c.bench_function("alpha_beta4_bitmatrix5n", |b| {
        b.iter(|| play_with_search(&alpha_beta4_bitmatrix5n))
    });
    c.bench_function("alpha_beta4_bitmatrix10", |b| {
        b.iter(|| play_with_search(&alpha_beta4_bitmatrix10))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

use criterion::{criterion_group, criterion_main, Criterion};
use rust_reversi_core::board::Board;

fn play_game() {
    let mut board = Board::new();
    while !board.is_game_over() {
        if board.is_pass() {
            board.do_pass().unwrap();
        } else {
            let m = board.get_random_move().unwrap();
            board.do_move(m).unwrap();
        }
    }
}

fn perft() {
    let depth: usize = 6;
    const PERFT_MODE1: [usize; 11] = [
        1, 4, 12, 56, 244, 1396, 8200, 55092, 390216, 3005288, 24571284,
    ];
    fn perft_rec(board: &mut Board, depth: usize) -> usize {
        if depth == 0 || board.is_game_over() {
            return 1;
        } else if board.is_pass() {
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            return perft_rec(&mut new_board, depth - 1);
        }
        let mut nodes = 0;
        for mut b in board.get_child_boards().unwrap() {
            nodes += perft_rec(&mut b, depth - 1);
        }
        nodes
    }
    let mut board = Board::new();
    let nodes = perft_rec(&mut board, depth);
    assert_eq!(nodes, PERFT_MODE1[depth]);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("play_game", |b| b.iter(play_game));
    c.bench_function("perft", |b| b.iter(perft));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

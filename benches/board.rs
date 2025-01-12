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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("play_game", |b| b.iter(play_game));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

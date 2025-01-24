use rand::Rng;
use rust_reversi_core::board::{Board, Turn};
use rust_reversi_core::search::MatrixEvaluator;
use rust_reversi_core::search::{AlphaBetaSearch, Search};
use std::env;
use std::rc::Rc;

const EPSILON: f64 = 1e-2;
const MATRIX: [[i32; 8]; 8] = [
    [50, -10, 11, 6, 6, 11, -10, 50],
    [-10, -15, 1, 2, 2, 1, -15, -10],
    [11, 1, 1, 1, 1, 1, 1, 11],
    [6, 2, 1, 3, 3, 1, 2, 6],
    [6, 2, 1, 3, 3, 1, 2, 6],
    [11, 1, 1, 1, 1, 1, 1, 11],
    [-10, -15, 1, 2, 2, 1, -15, -10],
    [50, -10, 11, 6, 6, 11, -10, 50],
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let depth = args[1].parse::<usize>().unwrap();
    let turn = match args[2].as_str() {
        "BLACK" => Turn::Black,
        "WHITE" => Turn::White,
        _ => panic!("Invalid turn"),
    };
    let mut board = Board::new();
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "ping" {
            println!("pong");
        } else {
            if board.set_board_str(input, turn).is_err() {
                eprintln!("Invalid board string");
                eprintln!("{}", input);
                return;
            }
            if rand::thread_rng().gen_bool(EPSILON) {
                let m = board.get_random_move();
                if m.is_err() {
                    eprintln!("No legal moves");
                    eprintln!("{}", input);
                    return;
                } else {
                    println!("{}", m.unwrap());
                }
            } else {
                let evaluator = MatrixEvaluator::new(MATRIX);
                let search = AlphaBetaSearch::new(depth, Rc::new(evaluator), 1 << 10);
                let m = search.get_move(&mut board);
                if m.is_none() {
                    eprintln!("No legal moves");
                    eprintln!("{}", input);
                    return;
                } else {
                    println!("{}", m.unwrap());
                }
            }
        }
    }
}

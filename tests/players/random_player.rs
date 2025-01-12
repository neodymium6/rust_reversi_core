#![cfg(not(test))]
use rust_reversi_core::board::{Board, Turn};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let turn = match args[1].as_str() {
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
            let m = board.get_random_move();
            if m.is_err() {
                eprintln!("No legal moves");
                eprintln!("{}", input);
                return;
            } else {
                println!("{}", m.unwrap());
            }
        }
    }
}

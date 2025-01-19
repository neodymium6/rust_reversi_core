mod players;

const N_GAMES: usize = 100;
const EPSILON: f64 = 0.1;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use players::compile_player;
    use players::get_player_path;
    use rand::Rng;
    use rust_reversi_core::arena::LocalArena;
    use rust_reversi_core::board::Board;
    use rust_reversi_core::board::Turn;
    use rust_reversi_core::search::AlphaBetaSearch;
    use rust_reversi_core::search::BitMatrixEvaluator;
    use rust_reversi_core::search::MatrixEvaluator;
    use rust_reversi_core::search::MctsSearch;
    use rust_reversi_core::search::PieceEvaluator;
    use rust_reversi_core::search::Search;

    trait Player {
        fn get_move(&self, board: &mut Board) -> Option<usize>;
        fn get_move_with_timeout(&self, board: &mut Board, timeout: Duration) -> Option<usize>;
    }

    struct RandomPlayer {}
    impl Player for RandomPlayer {
        fn get_move(&self, board: &mut Board) -> Option<usize> {
            Some(board.get_random_move().unwrap())
        }

        fn get_move_with_timeout(&self, board: &mut Board, _timeout: Duration) -> Option<usize> {
            self.get_move(board)
        }
    }

    struct SearchPlayer {
        search: Box<dyn Search>,
    }
    impl Player for SearchPlayer {
        fn get_move(&self, board: &mut Board) -> Option<usize> {
            self.search.get_move(board)
        }

        fn get_move_with_timeout(&self, board: &mut Board, timeout: Duration) -> Option<usize> {
            self.search.get_move_with_timeout(board, timeout)
        }
    }
    enum TurnOrder {
        P1IsBlack,
        P1IsWhite,
    }
    enum PlayResult {
        P1Win,
        P2Win,
        Draw,
    }
    fn play_game_with_timeout(
        p1: Box<dyn Player>,
        p2: Box<dyn Player>,
        timeout: Duration,
        turn_order: TurnOrder,
    ) -> PlayResult {
        let p1_turn = match turn_order {
            TurnOrder::P1IsBlack => Turn::Black,
            TurnOrder::P1IsWhite => Turn::White,
        };
        let mut board = Board::new();
        let mut rng = rand::thread_rng();
        while !board.is_game_over() {
            if board.is_pass() {
                board.do_pass().unwrap();
                continue;
            }

            // for variety
            if rng.gen_bool(EPSILON) {
                let action = board.get_random_move().unwrap();
                board.do_move(action).unwrap();
                continue;
            }

            if board.get_turn() == p1_turn {
                let action = p1.get_move_with_timeout(&mut board, timeout).unwrap();
                board.do_move(action).unwrap();
            } else {
                let action = p2.get_move_with_timeout(&mut board, timeout).unwrap();
                board.do_move(action).unwrap();
            }
        }
        let winner = board.get_winner().unwrap();
        match winner {
            Some(turn) => match turn == p1_turn {
                true => PlayResult::P1Win,
                false => PlayResult::P2Win,
            },
            None => PlayResult::Draw,
        }
    }

    #[test]
    fn random_vs_piece() {
        compile_player("random_player");
        let random_player = get_player_path("random_player");

        compile_player("piece_player");
        let piece_player = get_player_path("piece_player");
        let depth = 3;

        let command1 = vec![random_player.to_str().unwrap().to_string()];
        let command2 = vec![
            piece_player.to_str().unwrap().to_string(),
            depth.to_string(),
        ];
        let mut arena = LocalArena::new(command1, command2, false);
        arena.play_n(N_GAMES).unwrap();

        let (wins1, wins2, _draws) = arena.get_stats();
        let (pieces1, pieces2) = arena.get_pieces();

        assert!(wins2 > wins1);
        assert!(pieces2 > pieces1);
    }

    #[test]
    fn depth_comparison() {
        compile_player("piece_player");
        let piece_player = get_player_path("piece_player");

        let depth1 = 3;
        let command1 = vec![
            piece_player.to_str().unwrap().to_string(),
            depth1.to_string(),
        ];

        let depth2 = 2;
        let command2 = vec![
            piece_player.to_str().unwrap().to_string(),
            depth2.to_string(),
        ];
        let mut arena = LocalArena::new(command1, command2, false);
        arena.play_n(N_GAMES).unwrap();

        let (wins1, wins2, _draws) = arena.get_stats();
        let (pieces1, pieces2) = arena.get_pieces();

        assert!(wins1 > wins2);
        assert!(pieces1 > pieces2);
    }

    #[test]
    fn matrix_vs_piece() {
        compile_player("matrix_player");
        let matrix_player = get_player_path("matrix_player");

        compile_player("piece_player");
        let piece_player = get_player_path("piece_player");
        let depth = 3;

        let command1 = vec![
            matrix_player.to_str().unwrap().to_string(),
            depth.to_string(),
        ];
        let command2 = vec![
            piece_player.to_str().unwrap().to_string(),
            depth.to_string(),
        ];
        let mut arena = LocalArena::new(command1, command2, false);
        arena.play_n(N_GAMES).unwrap();

        let (wins1, wins2, _draws) = arena.get_stats();
        let (pieces1, pieces2) = arena.get_pieces();

        assert!(wins1 > wins2);
        assert!(pieces1 > pieces2);
    }

    #[test]
    fn iter_deepening() {
        let evaluator = PieceEvaluator::new();
        let depth = 60;
        let search = AlphaBetaSearch::new(depth, Box::new(evaluator));
        let mut board = Board::new();

        let timeout = 0.01;
        let timeout_duration = std::time::Duration::from_secs_f64(timeout);

        while !board.is_game_over() {
            if board.is_pass() {
                board.do_pass().unwrap();
                continue;
            }
            let start = std::time::Instant::now();
            let m = search
                .get_move_with_timeout(&mut board, timeout_duration)
                .unwrap();
            let elapsed = start.elapsed().as_secs_f64();
            assert!(elapsed < timeout);
            board.do_move(m).unwrap();
        }
    }

    #[test]
    fn bitmatrix_matrix_can_be_same() {
        let matrix = [
            [100, -20, 10, 5, 5, 10, -20, 100],
            [-20, -50, -2, -2, -2, -2, -50, -20],
            [10, -2, -1, -1, -1, -1, -2, 10],
            [5, -2, -1, -1, -1, -1, -2, 5],
            [5, -2, -1, -1, -1, -1, -2, 5],
            [10, -2, -1, -1, -1, -1, -2, 10],
            [-20, -50, -2, -2, -2, -2, -50, -20],
            [100, -20, 10, 5, 5, 10, -20, 100],
        ];
        let matrix_evaluator = MatrixEvaluator::new(matrix);
        let depth = 0;
        let matrix_search = AlphaBetaSearch::new(depth, Box::new(matrix_evaluator));
        let masks: Vec<u64> = vec![
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
        ];
        let weights: Vec<i32> = vec![-1, -1, -1, -2, -2, -50, 5, 10, -20, 100];
        let bitmatrix_evaluator = BitMatrixEvaluator::<10>::new(weights, masks);
        let bitmatrix_search = AlphaBetaSearch::new(depth, Box::new(bitmatrix_evaluator));
        for _ in 0..1000 {
            let mut board = Board::new();
            while !board.is_game_over() {
                if board.is_pass() {
                    board.do_pass().unwrap();
                    continue;
                }
                let m1 = matrix_search.get_move(&mut board).unwrap();
                let m2 = bitmatrix_search.get_move(&mut board).unwrap();
                assert_eq!(m1, m2);
                let m = board.get_random_move().unwrap();
                board.do_move(m).unwrap();
            }
        }
    }

    #[test]
    fn random_vs_mcts() {
        let timeout = std::time::Duration::from_millis(10);
        let mut random_wins = 0;
        let mut mcts_wins = 0;
        for _ in 0..N_GAMES / 2 {
            let random_player = RandomPlayer {};
            let mcts_player = SearchPlayer {
                search: Box::new(MctsSearch::new(1000, 1.0, 10)),
            };
            let result = play_game_with_timeout(
                Box::new(random_player),
                Box::new(mcts_player),
                timeout,
                TurnOrder::P1IsBlack,
            );
            match result {
                PlayResult::P1Win => random_wins += 1,
                PlayResult::P2Win => mcts_wins += 1,
                PlayResult::Draw => (),
            }
            let random_player = RandomPlayer {};
            let mcts_player = SearchPlayer {
                search: Box::new(MctsSearch::new(1000, 1.0, 10)),
            };
            let result = play_game_with_timeout(
                Box::new(mcts_player),
                Box::new(random_player),
                timeout,
                TurnOrder::P1IsWhite,
            );
            match result {
                PlayResult::P1Win => mcts_wins += 1,
                PlayResult::P2Win => random_wins += 1,
                PlayResult::Draw => (),
            }
        }
        assert!(mcts_wins > random_wins);
    }

    #[test]
    fn mcts_timeout() {
        let search = MctsSearch::new(1000, 1.0, 10);
        let mut board = Board::new();

        let timeout = 0.01;
        let timeout_duration = std::time::Duration::from_secs_f64(timeout);

        while !board.is_game_over() {
            if board.is_pass() {
                board.do_pass().unwrap();
                continue;
            }
            let start = std::time::Instant::now();
            let m = search
                .get_move_with_timeout(&mut board, timeout_duration)
                .unwrap();
            let elapsed = start.elapsed().as_secs_f64();
            assert!(elapsed < timeout);
            board.do_move(m).unwrap();
        }
    }
}

mod players;

const N_GAMES: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;
    use players::compile_player;
    use players::get_player_path;
    use rust_reversi_core::arena::LocalArena;
    use rust_reversi_core::board::Board;
    use rust_reversi_core::search::AlphaBetaSearch;
    use rust_reversi_core::search::PieceEvaluator;

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
        let depth = 3;
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
                .get_move_with_iter_deepening(&board, timeout_duration)
                .unwrap();
            let elapsed = start.elapsed().as_secs_f64();
            assert!(elapsed < timeout);
            board.do_move(m).unwrap();
        }
    }
}

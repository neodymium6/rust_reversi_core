#[cfg(test)]
mod tests {
    use rust_reversi_core::board::{Board, BoardError, Color, Turn};

    #[test]
    fn test_new_board() {
        let board = Board::new();
        let (player_board, opponent_board, turn) = board.get_board();
        assert_eq!(player_board, 0x0000000810000000);
        assert_eq!(opponent_board, 0x0000001008000000);
        assert_eq!(turn, Turn::Black);
    }

    #[test]
    fn test_initial_piece_count() {
        let board = Board::new();
        assert_eq!(board.player_piece_num(), 2);
        assert_eq!(board.opponent_piece_num(), 2);
        assert_eq!(board.black_piece_num(), 2);
        assert_eq!(board.white_piece_num(), 2);
        assert_eq!(board.piece_sum(), 4);
        assert_eq!(board.diff_piece_num(), 0);
    }

    #[test]
    fn test_initial_legal_moves() {
        let board = Board::new();
        let legal_moves = board.get_legal_moves_vec();
        assert_eq!(legal_moves.len(), 4);
        assert!(legal_moves.contains(&19)); // (2,3)
        assert!(legal_moves.contains(&26)); // (3,2)
        assert!(legal_moves.contains(&37)); // (4,5)
        assert!(legal_moves.contains(&44)); // (5,4)
    }

    #[test]
    fn test_do_move() {
        let mut board = Board::new();

        // Make a legal move
        board.do_move(19).unwrap(); // (2,3)

        // Check piece counts after move
        assert_eq!(board.black_piece_num(), 4);
        assert_eq!(board.white_piece_num(), 1);

        // Verify it's now White's turn
        assert_eq!(board.get_turn(), Turn::White);
    }

    #[test]
    fn test_invalid_move() {
        let mut board = Board::new();

        // Try to make an invalid move
        let result = board.do_move(0); // (0,0) is not a legal move
        assert!(matches!(result, Err(BoardError::InvalidMove)));

        // Board state should remain unchanged
        assert_eq!(board.black_piece_num(), 2);
        assert_eq!(board.white_piece_num(), 2);
        assert_eq!(board.get_turn(), Turn::Black);
    }

    #[test]
    fn test_pass() {
        let mut board = Board::new();
        // Set up a position where Black must pass
        board
            .set_board_str(
                &format!(
                    "{}{}{}{}{}{}{}{}",
                    "--------",
                    "--------",
                    "--OOO---",
                    "---OOO--",
                    "--OOOO--",
                    "--OO----",
                    "---O----",
                    "---X----",
                ),
                Turn::White,
            )
            .unwrap();

        assert!(board.is_pass());

        let result = board.do_pass();
        assert!(result.is_ok());
        assert_eq!(board.get_turn(), Turn::Black);
    }

    #[test]
    fn test_game_over() -> Result<(), BoardError> {
        let mut board = Board::new();
        // Set up a completed game position
        board
            .set_board_str(
                "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOXXXX",
                Turn::Black,
            )
            .unwrap();

        assert!(board.is_game_over());
        assert!(board.is_black_win()?);
        assert!(!board.is_white_win()?);
        assert!(!board.is_draw()?);
        assert_eq!(board.get_winner()?, Some(Turn::Black));

        Ok(())
    }

    #[test]
    fn test_board_str_conversion() -> Result<(), BoardError> {
        let mut board = Board::new();
        let board_str = "------------------OOO------OX-----OOXX----OX--------------------";

        board.set_board_str(board_str, Turn::White)?;
        assert_eq!(board.get_board_line().unwrap(), board_str);

        Ok(())
    }

    #[test]
    fn test_get_board_vec() -> Result<(), BoardError> {
        let board = Board::new();
        let board_vec = board.get_board_vec_black()?;

        // Check initial position
        assert_eq!(board_vec[28], Color::Black); // (3,4)
        assert_eq!(board_vec[35], Color::Black); // (4,3)
        assert_eq!(board_vec[27], Color::White); // (3,3)
        assert_eq!(board_vec[36], Color::White); // (4,4)

        // Check empty squares
        assert_eq!(board_vec[0], Color::Empty);
        assert_eq!(board_vec[63], Color::Empty);

        Ok(())
    }

    #[test]
    fn test_board_clone() {
        let board = Board::new();
        let cloned = board.clone();

        let (player_board, opponent_board, turn) = board.get_board();
        let (cloned_player_board, cloned_opponent_board, cloned_turn) = cloned.get_board();

        assert_eq!(player_board, cloned_player_board);
        assert_eq!(opponent_board, cloned_opponent_board);
        assert_eq!(turn, cloned_turn);
    }

    const PERFT_MODE1: [u64; 11] = [
        1, 4, 12, 56, 244, 1396, 8200, 55092, 390216, 3005288, 24571284,
    ];
    const PERFT_MODE2: [u64; 11] = [
        1, 4, 12, 56, 244, 1396, 8200, 55092, 390216, 3005320, 24571420,
    ];

    fn perft1(board: &mut Board, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }
        if board.is_game_over() {
            return 1;
        }
        if board.is_pass() {
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            return perft1(&mut new_board, depth - 1);
        }
        let mut count = 0;
        for mut board in board.get_child_boards().unwrap() {
            count += perft1(&mut board, depth - 1);
        }
        count
    }
    fn perft2(board: &mut Board, depth: u8) -> u64 {
        if depth == 0 {
            return 1;
        }
        if board.is_game_over() {
            return 1;
        }
        if board.is_pass() {
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            return perft2(&mut new_board, depth); // different from perft1
        }
        let mut count = 0;
        for mut board in board.get_child_boards().unwrap() {
            count += perft2(&mut board, depth - 1);
        }
        count
    }

    #[test]
    fn test_perft_mode1() {
        let mut board = Board::new();
        for (depth, &nodes) in PERFT_MODE1.iter().enumerate() {
            assert_eq!(perft1(&mut board, depth as u8), nodes);
        }
    }

    #[test]
    fn test_perft_mode2() {
        let mut board = Board::new();
        for (depth, &nodes) in PERFT_MODE2.iter().enumerate() {
            assert_eq!(perft2(&mut board, depth as u8), nodes);
        }
    }
}

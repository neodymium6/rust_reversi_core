use crate::board::Board;
use crate::search::evaluator::Evaluator;
use crate::search::time_keeper::TimeKeeper;

pub struct AlphaBetaSearch {
    max_depth: usize,
    evaluator: Box<dyn Evaluator>,
}

impl AlphaBetaSearch {
    /// Create a new AlphaBetaSearch instance.
    /// # Arguments
    /// * `max_depth` - The maximum depth of the search tree.
    /// * `evaluator` - The evaluator to evaluate the board.
    /// # Returns
    /// A new AlphaBetaSearch instance.
    pub fn new(max_depth: usize, evaluator: Box<dyn Evaluator>) -> Self {
        Self {
            max_depth,
            evaluator,
        }
    }

    fn score_board(&self, board: &mut Board) -> i32 {
        if board.is_game_over() {
            match (board.is_win(), board.is_lose()) {
                (Ok(true), _) => return i32::MAX - 2,
                (_, Ok(true)) => return i32::MIN + 2,
                _ => return 0,
            }
        }
        self.evaluator.evaluate(board)
    }

    fn get_child_boards_ordered(&self, board: &mut Board) -> Option<Vec<Board>> {
        if board.is_pass() {
            return None;
        }
        let mut child_boards = board.get_child_boards().unwrap();
        child_boards.sort_by_key(|b| {
            let mut b_clone = b.clone();
            self.score_board(&mut b_clone)
        });
        Some(child_boards)
    }

    fn get_legal_moves_vec_ordered(&self, board: &mut Board) -> Option<Vec<usize>> {
        if board.is_pass() {
            return None;
        }
        let mut legal_moves = board.get_legal_moves_vec();
        legal_moves.sort_by_key(|&m| {
            let mut new_board = board.clone();
            new_board.do_move(m).unwrap();
            self.score_board(&mut new_board)
        });
        Some(legal_moves)
    }

    fn get_search_score(&self, board: &mut Board, depth: usize, alpha: i32, beta: i32) -> i32 {
        if board.is_game_over() {
            match (board.is_win(), board.is_lose()) {
                (Ok(true), _) => return i32::MAX - 2,
                (_, Ok(true)) => return i32::MIN + 2,
                _ => return 0,
            }
        }
        if depth == 0 {
            return self.evaluator.evaluate(board);
        }

        let mut current_alpha = alpha;
        let child_boards = match (depth > 2, board.get_legal_moves().count_ones() > 4) {
            (true, true) => self.get_child_boards_ordered(board),
            _ => board.get_child_boards(),
        };
        if let Some(child_boards) = child_boards {
            for mut child_board in child_boards {
                let score =
                    -self.get_search_score(&mut child_board, depth - 1, -beta, -current_alpha);
                if score > current_alpha {
                    current_alpha = score;
                }
                if current_alpha >= beta {
                    // cut
                    return current_alpha;
                }
            }
            current_alpha
        } else {
            // pass
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            -self.get_search_score(&mut new_board, depth, -beta, -alpha)
        }
    }

    /// Get the best move for the given board.
    /// # Arguments
    /// * `board` - The board to search the best move.
    /// # Returns
    /// * `Some(usize)` - The best move.
    /// * `None` - player must pass.
    pub fn get_move(&self, board: &mut Board) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for move_i in self.get_legal_moves_vec_ordered(board).unwrap() {
            let mut new_board = board.clone();
            new_board.do_move(move_i).unwrap();
            let score = -self.get_search_score(&mut new_board, self.max_depth, -beta, -alpha);
            if score > alpha {
                alpha = score;
                best_move = Some(move_i);
            }
        }
        best_move
    }

    fn get_search_score_with_timeout(
        &self,
        board: &mut Board,
        depth: usize,
        alpha: i32,
        beta: i32,
        time_keeper: &TimeKeeper,
    ) -> i32 {
        if board.is_game_over() {
            match (board.is_win(), board.is_lose()) {
                (Ok(true), _) => return i32::MAX - 2,
                (_, Ok(true)) => return i32::MIN + 2,
                _ => return 0,
            }
        }
        if depth == 0 {
            return self.evaluator.evaluate(board);
        }

        let mut current_alpha = alpha;
        let child_boards = match (depth > 2, board.get_legal_moves().count_ones() > 4) {
            (true, true) => self.get_child_boards_ordered(board),
            _ => board.get_child_boards(),
        };
        if let Some(child_boards) = child_boards {
            for mut child_board in child_boards {
                let score = -self.get_search_score_with_timeout(
                    &mut child_board,
                    depth - 1,
                    -beta,
                    -current_alpha,
                    time_keeper,
                );
                if score > current_alpha {
                    current_alpha = score;
                }
                if current_alpha >= beta {
                    // cut
                    return current_alpha;
                }
                if time_keeper.is_timeout() {
                    break;
                }
            }
            current_alpha
        } else {
            // pass
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            -self.get_search_score_with_timeout(&mut new_board, depth, -beta, -alpha, time_keeper)
        }
    }

    fn get_move_with_timeout(
        &self,
        board: &mut Board,
        depth: usize,
        time_keeper: &TimeKeeper,
    ) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for move_i in self.get_legal_moves_vec_ordered(board).unwrap() {
            let mut new_board = board.clone();
            new_board.do_move(move_i).unwrap();
            let score = -self.get_search_score_with_timeout(
                &mut new_board,
                depth,
                -beta,
                -alpha,
                time_keeper,
            );
            if score > alpha {
                alpha = score;
                best_move = Some(move_i);
            }
            if time_keeper.is_timeout() {
                break;
            }
        }
        best_move
    }

    const MARGIN_TIME: f64 = 0.003;
    /// Get the best move for the given board with iterative deepening.
    /// # Arguments
    /// * `board` - The board to search the best move.
    /// * `timeout` - The timeout duration.
    /// # Returns
    /// * `Some(usize)` - The best move.
    /// * `None` - player must pass.
    /// # Note
    /// * The search will stop if the timeout is reached.
    /// * The field `max_depth` will be ignored.
    /// * Depth will be increased iteratively from 0.
    pub fn get_move_with_iter_deepening(
        &self,
        board: &mut Board,
        timeout: std::time::Duration,
    ) -> Option<usize> {
        let mut best_move = None;
        let search_duration = timeout.as_secs_f64() - Self::MARGIN_TIME;
        let time_keeper = TimeKeeper::new(std::time::Duration::from_secs_f64(search_duration));
        let mut depth = 0;
        loop {
            let move_i = self.get_move_with_timeout(board, depth, &time_keeper);
            if time_keeper.is_timeout() {
                break;
            }
            if let Some(m) = move_i {
                best_move = Some(m);
            }
            depth += 1;
        }
        best_move
    }
}

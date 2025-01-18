use crate::board::Board;
use crate::search::evaluator::Evaluator;
use crate::search::time_keeper::TimeKeeper;

pub struct NegaScoutSearch {
    max_depth: usize,
    evaluator: Box<dyn Evaluator>,
}

impl NegaScoutSearch {
    /// Create a new NegaScoutSearch instance.
    /// # Arguments
    /// * `max_depth` - The maximum depth of the search tree.
    /// * `evaluator` - The evaluator to evaluate the board.
    /// # Returns
    /// A new NegaScoutSearch instance.
    pub fn new(max_depth: usize, evaluator: Box<dyn Evaluator>) -> Self {
        Self {
            max_depth,
            evaluator,
        }
    }

    fn score_board(&self, board: &Board) -> i32 {
        if board.is_game_over() {
            match (board.is_win(), board.is_lose()) {
                (Ok(true), _) => return i32::MAX - 2,
                (_, Ok(true)) => return i32::MIN + 2,
                _ => return 0,
            }
        }
        self.evaluator.evaluate(board)
    }

    fn get_child_boards_ordered(&self, board: &Board) -> Option<Vec<Board>> {
        if board.is_pass() {
            return None;
        }
        let mut child_boards = board.get_child_boards().unwrap();
        child_boards.sort_by_key(|b| self.score_board(b));
        Some(child_boards)
    }

    fn get_legal_moves_vec_ordered(&self, board: &Board) -> Option<Vec<usize>> {
        if board.is_pass() {
            return None;
        }
        let mut legal_moves = board.get_legal_moves_vec();
        legal_moves.sort_by_key(|&m| {
            let mut new_board = board.clone();
            new_board.do_move(m).unwrap();
            self.score_board(&new_board)
        });
        Some(legal_moves)
    }

    fn get_search_score(&self, board: &Board, depth: usize, alpha: i32, beta: i32) -> i32 {
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

        if let Some(child_boards) = self.get_child_boards_ordered(board) {
            let mut current_alpha = alpha;
            // first child
            let score = -self.get_search_score(&child_boards[0], depth - 1, -beta, -current_alpha);
            let mut max = score;
            if beta <= score {
                return score;
            }
            if current_alpha < score {
                current_alpha = score;
            }

            for child_board in child_boards.iter().skip(1) {
                let mut score = -self.get_search_score(
                    child_board,
                    depth - 1,
                    -current_alpha - 1,
                    -current_alpha,
                );
                if beta <= score {
                    return score;
                }
                if current_alpha < score {
                    current_alpha = score;
                    score = -self.get_search_score(child_board, depth - 1, -beta, -current_alpha);
                    if beta <= score {
                        return score;
                    }
                    if current_alpha < score {
                        current_alpha = score;
                    }
                }
                if score > max {
                    max = score;
                }
            }
            max
        } else {
            // pass
            let mut new_board = board.clone();
            new_board.do_pass().unwrap();
            -self.get_search_score(&new_board, depth, -beta, -alpha)
        }
    }

    /// Get the best move for the given board.
    /// # Arguments
    /// * `board` - The board to search the best move.
    /// # Returns
    /// * `Some(usize)` - The best move.
    /// * `None` - player must pass.
    pub fn get_move(&self, board: &Board) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        // for move_i in self.get_legal_moves_vec_ordered(board).unwrap() {
        for move_i in board.get_legal_moves_vec() {
            let mut new_board = board.clone();
            new_board.do_move(move_i).unwrap();
            let score = -self.get_search_score(&new_board, self.max_depth, -beta, -alpha);
            if score > alpha {
                alpha = score;
                best_move = Some(move_i);
            }
        }
        best_move
    }
    fn get_search_score_with_timeout(
        &self,
        board: &Board,
        depth: usize,
        alpha: i32,
        beta: i32,
        time_keeper: &TimeKeeper,
    ) -> i32 {
        if board.is_game_over() {
            match board.is_win() {
                Ok(true) => return i32::MAX - 2,
                Ok(false) => match board.is_lose() {
                    Ok(true) => return i32::MIN + 2,
                    Ok(false) => return 0,
                    Err(_) => return 0,
                },
                Err(_) => return 0,
            }
        }
        if depth == 0 {
            return self.evaluator.evaluate(board);
        }

        let mut current_alpha = alpha;
        if let Some(child_boards) = self.get_child_boards_ordered(board) {
            for child_board in child_boards {
                let score = -self.get_search_score_with_timeout(
                    &child_board,
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
            -self.get_search_score_with_timeout(&new_board, depth, -beta, -alpha, time_keeper)
        }
    }

    fn get_move_with_timeout(
        &self,
        board: &Board,
        depth: usize,
        time_keeper: &TimeKeeper,
    ) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for move_i in self.get_legal_moves_vec_ordered(board).unwrap() {
            let mut new_board = board.clone();
            new_board.do_move(move_i).unwrap();
            let score =
                -self.get_search_score_with_timeout(&new_board, depth, -beta, -alpha, time_keeper);
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

    const MARGIN_TIME: f64 = 0.001;
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
        board: &Board,
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
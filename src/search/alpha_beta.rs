use std::sync::Arc;

use crate::board::Board;
use crate::search::evaluator::Evaluator;
use crate::search::time_keeper::TimeKeeper;
use crate::search::Search;
use crate::utils::StackVec64;

#[derive(Debug)]
pub struct AlphaBetaSearch {
    max_depth: usize,
    evaluator: Arc<dyn Evaluator>,
    move_ordering_evaluator: Arc<dyn Evaluator>,
    win_score: i32,
    margin_time: f64,
}

impl AlphaBetaSearch {
    /// Create a new AlphaBetaSearch instance.
    /// # Arguments
    /// * `max_depth` - The maximum depth of the search tree.
    /// * `evaluator` - The evaluator to evaluate the board.
    /// * `win_score` - The score of the win.
    /// # Returns
    /// A new AlphaBetaSearch instance.
    /// # Note
    /// * The win_score is used to determine the score of the win.
    /// * The win_score must be greater than any possible score.
    pub fn new(max_depth: usize, evaluator: Arc<dyn Evaluator>, win_score: i32) -> Self {
        Self {
            max_depth,
            evaluator: evaluator.clone(),
            move_ordering_evaluator: evaluator,
            win_score,
            margin_time: DEFAULT_MARGIN_TIME,
        }
    }

    /// Get the maximum depth of the search tree.
    pub fn get_max_depth(&self) -> usize {
        self.max_depth
    }

    /// Set the maximum depth of the search tree.
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = max_depth;
    }

    /// Get the win score.
    pub fn get_win_score(&self) -> i32 {
        self.win_score
    }

    /// Set the win score.
    pub fn set_win_score(&mut self, win_score: i32) {
        self.win_score = win_score;
    }

    /// Get move ordering evaluator.
    pub fn get_move_ordering_evaluator(&self) -> Arc<dyn Evaluator> {
        self.move_ordering_evaluator.clone()
    }

    /// Set move ordering evaluator.
    pub fn set_move_ordering_evaluator(&mut self, evaluator: Arc<dyn Evaluator>) {
        self.move_ordering_evaluator = evaluator;
    }

    // Evaluate for move ordering.
    fn score_board(&self, board: &mut Board) -> i32 {
        if board.is_game_over() {
            match (board.is_win(), board.is_lose()) {
                (Ok(true), _) => return self.win_score,
                (_, Ok(true)) => return -self.win_score,
                _ => return 0,
            }
        }
        self.move_ordering_evaluator.evaluate(board)
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

    fn get_legal_moves_vec_ordered(&self, board: &mut Board) -> Option<StackVec64<usize>> {
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
                (Ok(true), _) => return self.win_score,
                (_, Ok(true)) => return -self.win_score,
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
                (Ok(true), _) => return self.win_score,
                (_, Ok(true)) => return -self.win_score,
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

    fn get_move_with_timeout_inner(
        &self,
        board: &mut Board,
        depth: usize,
        time_keeper: &TimeKeeper,
    ) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for &move_i in &self.get_legal_moves_vec_ordered(board).unwrap() {
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

    /// Set the margin time for the search.
    pub fn set_margin_time(&mut self, margin_time: f64) {
        self.margin_time = margin_time;
    }

    /// Get the margin time for the search.
    pub fn get_margin_time(&self) -> f64 {
        self.margin_time
    }
}

const DEFAULT_MARGIN_TIME: f64 = 0.005;
impl Search for AlphaBetaSearch {
    /// Get the best move for the given board.
    /// # Arguments
    /// * `board` - The board to search the best move.
    /// # Returns
    /// * `Some(usize)` - The best move.
    /// * `None` - player must pass.
    fn get_move(&self, board: &mut Board) -> Option<usize> {
        let mut best_move = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for &move_i in &self.get_legal_moves_vec_ordered(board).unwrap() {
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

    /// Get the best move for the given board with iterative deepening.
    /// # Arguments
    /// * `board` - The board to search the best move.
    /// * `timeout` - The timeout duration.
    /// # Returns
    /// * `Some(usize)` - The best move.
    /// * `None` - player must pass.
    /// # Note
    /// * The search will stop if the timeout is reached or max depth is reached.
    /// * Depth will be increased iteratively from 0.
    fn get_move_with_timeout(
        &self,
        board: &mut Board,
        timeout: std::time::Duration,
    ) -> Option<usize> {
        let mut best_move = None;
        let search_duration = timeout.as_secs_f64() - self.margin_time;
        let time_keeper = TimeKeeper::new(std::time::Duration::from_secs_f64(search_duration));
        for depth in 0..self.max_depth {
            let move_i = self.get_move_with_timeout_inner(board, depth, &time_keeper);
            if time_keeper.is_timeout() {
                break;
            }
            if let Some(m) = move_i {
                best_move = Some(m);
            }
        }
        best_move
    }

    /// Get the search score for the given board.
    /// # Arguments
    /// * `board` - The board to search the score.
    /// # Returns
    /// The search score.
    /// # Note
    /// The search score is the score of the best move.
    fn get_search_score(&self, board: &mut Board) -> f64 {
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX - 1;
        for &move_i in &self.get_legal_moves_vec_ordered(board).unwrap() {
            let mut new_board = board.clone();
            new_board.do_move(move_i).unwrap();
            let score = -self.get_search_score(&mut new_board, self.max_depth, -beta, -alpha);
            if score > alpha {
                alpha = score;
            }
        }
        alpha as f64
    }
}

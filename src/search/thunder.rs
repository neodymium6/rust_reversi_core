use crate::board::Board;
use crate::search::time_keeper::TimeKeeper;
use crate::search::Search;
use std::rc::Rc;
use std::time::Duration;

use super::winrate_evaluator::WinrateEvaluator;

struct ThunderNode {
    board: Board,
    evaluator: Rc<dyn WinrateEvaluator>,
    w: f64,
    n_visits: usize,
    children: Option<Vec<ThunderNode>>,
}

impl ThunderNode {
    fn new(board: Board, evaluator: Rc<dyn WinrateEvaluator>) -> Self {
        Self {
            board,
            evaluator,
            w: 0.0,
            n_visits: 0,
            children: None,
        }
    }

    fn expand(&mut self) {
        if let Some(children) = self.board.get_child_boards() {
            self.children = Some(
                children
                    .into_iter()
                    .map(|b| ThunderNode::new(b, self.evaluator.clone()))
                    .collect(),
            );
        } else {
            let mut board = self.board.clone();
            board.do_pass().unwrap();
            self.children = Some(vec![ThunderNode::new(board, self.evaluator.clone())]);
        }
    }

    fn score_board(board: &mut Board, evaluator: &Rc<dyn WinrateEvaluator>) -> f64 {
        if board.is_game_over() {
            match (board.is_win(), board.is_lose()) {
                (Ok(true), _) => return 1.0,
                (_, Ok(true)) => return 0.0,
                _ => return 0.5,
            }
        }
        evaluator.evaluate(board)
    }

    fn select_child_index(&self) -> usize {
        for (i, child) in self.children.as_ref().unwrap().iter().enumerate() {
            if child.n_visits == 0 {
                return i;
            }
        }
        let mut best_child_index = 0;
        let mut best_thunder_score = f64::NEG_INFINITY;
        for (i, child) in self.children.as_ref().unwrap().iter().enumerate() {
            let thunder_score = 1.0 - child.w / child.n_visits as f64;
            if thunder_score > best_thunder_score {
                best_thunder_score = thunder_score;
                best_child_index = i;
            }
        }
        best_child_index
    }

    fn evaluate(&mut self) -> f64 {
        if self.board.is_game_over() {
            let value = match self.board.get_winner().unwrap() {
                Some(winner) => {
                    if winner == self.board.get_turn() {
                        1.0
                    } else {
                        0.0
                    }
                }
                None => 0.5,
            };
            self.w += value;
            self.n_visits += 1;
            value
        } else if self.children.is_none() {
            let value = Self::score_board(&mut self.board, &self.evaluator);
            self.w += value;
            self.n_visits += 1;
            self.expand();
            value
        } else {
            let child_index = self.select_child_index();
            let value = 1.0 - self.children.as_mut().unwrap()[child_index].evaluate();
            self.w += value;
            self.n_visits += 1;
            value
        }
    }
}

pub struct ThunderSearch {
    n_playouts: usize,
    evaluator: Rc<dyn WinrateEvaluator>,
}

impl ThunderSearch {
    /// Create a new ThunderSearch instance.
    /// # Arguments
    /// * `n_playouts` - The number of playouts to run.
    /// * `evaluator` - The evaluator to evaluate the board.
    /// * `c` - The exploration parameter.
    /// * `expansion_threshold` - The number of visits to expand the node.
    /// # Returns
    /// A new MctsSearch instance.
    pub fn new(n_playouts: usize, evaluator: Rc<dyn WinrateEvaluator>) -> Self {
        Self {
            n_playouts,
            evaluator,
        }
    }

    /// Get the number of playouts to run.
    pub fn get_n_playouts(&self) -> usize {
        self.n_playouts
    }

    /// Set the number of playouts to run.
    pub fn set_n_playouts(&mut self, n_playouts: usize) {
        self.n_playouts = n_playouts;
    }
}

const MARGIN_TIME: f64 = 0.0011;
const CHECK_INTERVAL: usize = 100;
impl Search for ThunderSearch {
    /// Get the best move for the given board.
    /// # Arguments
    /// * `board` - The board to search.
    /// # Returns
    /// The best move.
    /// `Some(usize)` - The best move.
    /// `None` - player must pass.
    fn get_move(&self, board: &mut Board) -> Option<usize> {
        let mut root = ThunderNode::new(board.clone(), self.evaluator.clone());
        root.expand();
        for _ in 0..self.n_playouts {
            root.evaluate();
        }
        let mut best_child_index = 0;
        let mut best_n_visits = 0;
        for (i, child) in root.children.as_ref().unwrap().iter().enumerate() {
            if child.n_visits > best_n_visits {
                best_n_visits = child.n_visits;
                best_child_index = i;
            }
        }
        let legal_moves = board.get_legal_moves_vec();
        Some(legal_moves[best_child_index])
    }

    /// Get the best move for the given board with a timeout.
    /// # Arguments
    /// * `board` - The board to search.
    /// * `timeout` - The timeout duration.
    /// # Returns
    /// The best move.
    /// `Some(usize)` - The best move.
    /// `None` - player must pass.
    /// # Note
    /// The search will be stopped when the timeout is reached or the number of playouts is reached.
    /// If you want to stop the search when the timeout is reached, set the timeout to a bigger value.
    fn get_move_with_timeout(&self, board: &mut Board, timeout: Duration) -> Option<usize> {
        let mut root = ThunderNode::new(board.clone(), self.evaluator.clone());
        root.expand();
        let search_duration = timeout.as_secs_f64() - MARGIN_TIME;
        let time_keeper = TimeKeeper::new(Duration::from_secs_f64(search_duration));
        for i in 0..self.n_playouts {
            root.evaluate();
            if i % CHECK_INTERVAL == 0 && time_keeper.is_timeout() {
                break;
            }
        }
        let mut best_child_index = 0;
        let mut best_n_visits = 0;
        for (i, child) in root.children.as_ref().unwrap().iter().enumerate() {
            if child.n_visits > best_n_visits {
                best_n_visits = child.n_visits;
                best_child_index = i;
            }
        }
        let legal_moves = board.get_legal_moves_vec();
        Some(legal_moves[best_child_index])
    }
}

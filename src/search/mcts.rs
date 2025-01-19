use crate::board::Board;
use crate::search::time_keeper::TimeKeeper;
use crate::search::Search;
use std::time::Duration;

struct MctsNode {
    board: Board,
    c: f64,
    expansion_threshold: usize,
    w: f64,
    n_visits: usize,
    children: Option<Vec<MctsNode>>,
}

impl MctsNode {
    fn new(board: Board, c: f64, expansion_threshold: usize) -> Self {
        Self {
            board,
            c,
            expansion_threshold,
            w: 0.0,
            n_visits: 0,
            children: None,
        }
    }

    fn expand(&mut self) {
        if self.children.is_some() {
            panic!("MctsNode::expand called on a node that is already expanded.");
        }
        if self.board.is_game_over() {
            panic!("MctsNode::expand called on a node that is a terminal node.");
        }
        if let Some(children) = self.board.get_child_boards() {
            self.children = Some(
                children
                    .into_iter()
                    .map(|b| MctsNode::new(b, self.c, self.expansion_threshold))
                    .collect(),
            );
        } else {
            let mut board = self.board.clone();
            board.do_pass().unwrap();
            self.children = Some(vec![MctsNode::new(board, self.c, self.expansion_threshold)]);
        }
    }

    fn play_out(board: &Board) -> f64 {
        let mut board = board.clone();
        let node_turn = board.get_turn();
        while !board.is_game_over() {
            if board.is_pass() {
                board.do_pass().unwrap();
            } else {
                let m = board.get_random_move().unwrap();
                board.do_move(m).unwrap();
            }
        }
        match board.get_winner().unwrap() {
            Some(winner) => {
                if winner == node_turn {
                    1.0
                } else {
                    0.0
                }
            }
            None => 0.5,
        }
    }

    fn select_child_index(&self) -> usize {
        for (i, child) in self.children.as_ref().unwrap().iter().enumerate() {
            if child.n_visits == 0 {
                return i;
            }
        }
        let mut t: f64 = 0.0;
        for child in self.children.as_ref().unwrap() {
            t += child.n_visits as f64;
        }
        let mut best_child_index = 0;
        let mut best_ucb = f64::NEG_INFINITY;
        for (i, child) in self.children.as_ref().unwrap().iter().enumerate() {
            let ucb = 1.0 - child.w / child.n_visits as f64
                + self.c * (2.0 * t.ln() / child.n_visits as f64).sqrt();
            if ucb > best_ucb {
                best_ucb = ucb;
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
            let value = Self::play_out(&self.board);
            self.w += value;
            self.n_visits += 1;

            if self.n_visits >= self.expansion_threshold {
                self.expand();
            }

            return value;
        } else {
            let child_index = self.select_child_index();
            let value = 1.0 - self.children.as_mut().unwrap()[child_index].evaluate();
            self.w += value;
            self.n_visits += 1;
            value
        }
    }
}

pub struct MctsSearch {
    n_playouts: usize,
    c: f64,
    expansion_threshold: usize,
}

impl MctsSearch {
    /// Create a new MctsSearch instance.
    /// # Arguments
    /// * `n_playouts` - The number of playouts to run.
    /// * `c` - The exploration parameter.
    /// * `expansion_threshold` - The number of visits to expand the node.
    /// # Returns
    /// A new MctsSearch instance.
    pub fn new(n_playouts: usize, c: f64, expansion_threshold: usize) -> Self {
        Self {
            n_playouts,
            c,
            expansion_threshold,
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

const MARGIN_TIME: f64 = 0.001;
impl Search for MctsSearch {
    /// Get the best move for the given board.
    /// # Arguments
    /// * `board` - The board to search.
    /// # Returns
    /// The best move.
    /// `Some(usize)` - The best move.
    /// `None` - player must pass.
    fn get_move(&self, board: &mut Board) -> Option<usize> {
        let mut root = MctsNode::new(board.clone(), self.c, self.expansion_threshold);
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
        let mut root = MctsNode::new(board.clone(), self.c, self.expansion_threshold);
        root.expand();
        let search_duration = timeout.as_secs_f64() - MARGIN_TIME;
        let time_keeper = TimeKeeper::new(Duration::from_secs_f64(search_duration));
        let check_interval = (self.n_playouts / 100).clamp(100, 10000);
        for i in 0..self.n_playouts {
            root.evaluate();
            if i % check_interval == 0 && time_keeper.is_timeout() {
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

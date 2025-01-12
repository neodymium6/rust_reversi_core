use crate::board::{Board, Color};

pub trait Evaluator: Send + Sync {
    fn evaluate(&self, board: &Board) -> i32;
}

/// Score is the difference between the number of pieces.
#[derive(Clone, Default)]
pub struct PieceEvaluator {}

impl PieceEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluator for PieceEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        board.diff_piece_num()
    }
}

/// Score is the number of legal moves.
#[derive(Clone, Default)]
pub struct LegalNumEvaluator {}
impl LegalNumEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluator for LegalNumEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        board.get_legal_moves_vec().len() as i32
    }
}

/// Score is calculated by the following matrix:
#[derive(Clone)]
pub struct MatrixEvaluator {
    matrix: [[i32; 8]; 8],
}
impl MatrixEvaluator {
    /// Create a new MatrixEvaluator instance.
    /// # Arguments
    /// * `matrix` - The matrix to evaluate the board.
    /// # Returns
    /// A new MatrixEvaluator instance.
    /// # Example
    /// ```
    /// use rust_reversi_core::search::MatrixEvaluator;
    /// let matrix = [
    ///   [100, -20, 10, 5, 5, 10, -20, 100],
    ///   [-20, -50, -2, -2, -2, -2, -50, -20],
    ///   [10, -2, -1, -1, -1, -1, -2, 10],
    ///   [5, -2, -1, -1, -1, -1, -2, 5],
    ///   [5, -2, -1, -1, -1, -1, -2, 5],
    ///   [10, -2, -1, -1, -1, -1, -2, 10],
    ///   [-20, -50, -2, -2, -2, -2, -50, -20],
    ///   [100, -20, 10, 5, 5, 10, -20, 100],
    /// ];
    /// let evaluator = MatrixEvaluator::new(matrix);
    /// ```
    /// # Note
    /// * The matrix must be 8x8.
    /// * Score is added if the piece is player's.
    /// * Score is subtracted if the piece is opponent's.
    pub fn new(matrix: [[i32; 8]; 8]) -> Self {
        Self { matrix }
    }
}

impl Evaluator for MatrixEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let mut score = 0;
        let board_vec = board.get_board_vec_black().unwrap();
        for (i, color) in board_vec.iter().enumerate() {
            if *color == Color::Black {
                score += self.matrix[i / 8][i % 8];
            } else if *color == Color::White {
                score -= self.matrix[i / 8][i % 8];
            }
        }
        score
    }
}

/// Score is calculated by the following bit patterns and weights:
#[derive(Clone)]
pub struct BitMatrixEvaluator {
    weights: Vec<i32>,
    masks: Vec<u64>,
}
impl BitMatrixEvaluator {
    /// Create a new BitMatrixEvaluator instance.
    /// # Arguments
    /// * `weights` - The weights to evaluate the board.
    /// * `masks` - The bit patterns to evaluate the board.
    /// # Returns
    /// A new BitMatrixEvaluator instance.
    /// # Example
    /// ```
    /// use rust_reversi_core::search::BitMatrixEvaluator;
    /// let weights = vec![10, -1];
    /// let masks = vec![
    ///    0x8100000000000081,
    ///    0x7e7e7e7e7e7e7e7e,
    /// ];
    /// let evaluator = BitMatrixEvaluator::new(weights, masks);
    /// ```
    /// # Note
    /// * The length of weights and masks must be the same.
    /// * Score is added if the piece is player's.
    /// * Score is subtracted if the piece is opponent's.
    /// * This evaluator is faster than MatrixEvaluator.
    /// * If you use symmetry matrix in MatrixEvaluator, you can use faster BitMatrixEvaluator.
    pub fn new(weights: Vec<i32>, masks: Vec<u64>) -> Self {
        assert_eq!(weights.len(), masks.len());
        Self { weights, masks }
    }
}

impl Evaluator for BitMatrixEvaluator {
    fn evaluate(&self, board: &Board) -> i32 {
        let mut score = 0;
        for (weight, mask) in self.weights.iter().zip(self.masks.iter()) {
            let (player_board, opponent_board, _turn) = board.get_board();
            let player = player_board & mask;
            let opponent = opponent_board & mask;
            score += weight * (player.count_ones() as i32 - opponent.count_ones() as i32);
        }
        score
    }
}

use std::fmt::{Debug, Display};

use crate::board::{Board, Color};

/// Evaluator trait.
/// # Note
/// * The score is positive if the player is winning.
/// * The score is negative if the opponent is winning.
/// * The score is 0 if the game is draw.
pub trait Evaluator: Send + Sync + Debug {
    fn evaluate(&self, board: &mut Board) -> i32;
}

/// Score is the difference between the number of pieces.
#[derive(Clone, Default, Debug)]
pub struct PieceEvaluator {}

impl PieceEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluator for PieceEvaluator {
    fn evaluate(&self, board: &mut Board) -> i32 {
        board.diff_piece_num()
    }
}

/// Score is the number of legal moves.
#[derive(Clone, Default, Debug)]
pub struct LegalNumEvaluator {}
impl LegalNumEvaluator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Evaluator for LegalNumEvaluator {
    fn evaluate(&self, board: &mut Board) -> i32 {
        board.get_legal_moves_vec().len() as i32
    }
}

/// Score is calculated by the following matrix:
#[derive(Clone, Debug)]
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

    /// Create a new MatrixEvaluator instance with the symmetry matrix.
    /// # Returns
    /// A new MatrixEvaluator<10> instance.
    /// # Note
    /// * The matrix must be symmetric.
    pub fn to_bit_matrix_evaluator(&self) -> BitMatrixEvaluator<10> {
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
        let weights = vec![
            self.matrix[3][3],
            self.matrix[3][2],
            self.matrix[2][2],
            self.matrix[3][1],
            self.matrix[2][1],
            self.matrix[1][1],
            self.matrix[3][0],
            self.matrix[2][0],
            self.matrix[1][0],
            self.matrix[0][0],
        ];

        BitMatrixEvaluator::new(weights, masks)
    }
}

impl Evaluator for MatrixEvaluator {
    fn evaluate(&self, board: &mut Board) -> i32 {
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
#[derive(Clone, Debug)]
pub struct BitMatrixEvaluator<const N: usize> {
    weights: [i32; N],
    masks: [u64; N],
    positive_start: usize,
}
impl<const N: usize> BitMatrixEvaluator<N> {
    /// Create a new BitMatrixEvaluator instance.
    /// # Arguments
    /// * `weights` - The weights to evaluate the board.
    /// * `masks` - The bit patterns to evaluate the board.
    /// # Type Parameters
    /// * `N` - The number of weight-mask pairs. This must match the length of input vectors.
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
    /// let evaluator = BitMatrixEvaluator::<2>::new(weights, masks);
    /// ```
    /// # Note
    /// * The length of weights and masks must be N.
    /// * Score is added if the piece is player's.
    /// * Score is subtracted if the piece is opponent's.
    /// * This evaluator is faster than MatrixEvaluator.
    /// * If you use symmetry matrix in MatrixEvaluator, you can use faster BitMatrixEvaluator.
    pub fn new(weights: Vec<i32>, masks: Vec<u64>) -> Self {
        assert_eq!(weights.len(), N);
        assert_eq!(masks.len(), N);
        let mut weights_array = [0; N];
        let mut masks_array = [0; N];
        let mut sorted_masks: Vec<_> = masks.iter().zip(weights.iter()).collect();
        sorted_masks.sort_by_key(|(_mask, weight)| *weight);
        let sorted_weights: Vec<i32> = sorted_masks
            .iter()
            .map(|(_mask, weight)| **weight)
            .collect();
        let sorted_masks: Vec<u64> = sorted_masks
            .into_iter()
            .map(|(mask, _weight)| *mask)
            .collect();

        let mut positive_start = 0;
        for (i, &weight) in sorted_weights.iter().enumerate() {
            if weight > 0 {
                positive_start = i;
                break;
            }
        }

        weights_array.copy_from_slice(&sorted_weights);
        masks_array.copy_from_slice(&sorted_masks);
        Self {
            weights: weights_array,
            masks: masks_array,
            positive_start,
        }
    }
}

impl<const N: usize> Evaluator for BitMatrixEvaluator<N> {
    fn evaluate(&self, board: &mut Board) -> i32 {
        let mut score = 0;
        let (player_board, opponent_board, _turn) = board.get_board();
        for i in 0..self.positive_start {
            let player_count = (player_board & self.masks[i]).count_ones() as i32;
            let opponent_count = (opponent_board & self.masks[i]).count_ones() as i32;
            for _ in 0..-self.weights[i] {
                score -= player_count - opponent_count;
            }
        }
        for i in self.positive_start..N {
            let player_count = (player_board & self.masks[i]).count_ones() as i32;
            let opponent_count = (opponent_board & self.masks[i]).count_ones() as i32;
            for _ in 0..self.weights[i] {
                score += player_count - opponent_count;
            }
        }
        score
    }
}

impl<const N: usize> Display for BitMatrixEvaluator<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitMatrixEvaluator [")?;
        for (i, weight) in self.weights.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", weight)?;
        }
        write!(f, "]")
    }
}

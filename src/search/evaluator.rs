use crate::board::{Board, Color};

pub trait Evaluator: Send + Sync {
    fn evaluate(&self, board: &Board) -> i32;
}

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

#[derive(Clone)]
pub struct MatrixEvaluator {
    matrix: [[i32; 8]; 8],
}
impl MatrixEvaluator {
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

mod alpha_beta;
mod evaluator;
mod mcts;
mod time_keeper;
pub use alpha_beta::AlphaBetaSearch;
pub use evaluator::BitMatrixEvaluator;
pub use evaluator::Evaluator;
pub use evaluator::LegalNumEvaluator;
pub use evaluator::MatrixEvaluator;
pub use evaluator::PieceEvaluator;
pub use mcts::MctsSearch;

use crate::board::Board;

pub trait Search {
    fn get_move(&self, board: &mut Board) -> Option<usize>;
    fn get_move_with_timeout(
        &self,
        board: &mut Board,
        timeout: std::time::Duration,
    ) -> Option<usize>;
}

mod alpha_beta;
mod evaluator;
mod mcts;
mod thunder;
mod time_keeper;
mod winrate_evaluator;
use std::fmt::Debug;

pub use alpha_beta::AlphaBetaSearch;
pub use evaluator::BitMatrixEvaluator;
pub use evaluator::Evaluator;
pub use evaluator::LegalNumEvaluator;
pub use evaluator::MatrixEvaluator;
pub use evaluator::PieceEvaluator;
pub use mcts::MctsSearch;
pub use thunder::ThunderSearch;
pub use winrate_evaluator::WinrateEvaluator;

use crate::board::Board;

pub trait Search: Debug {
    fn get_move(&self, board: &mut Board) -> Option<usize>;
    fn get_move_with_timeout(
        &self,
        board: &mut Board,
        timeout: std::time::Duration,
    ) -> Option<usize>;
    fn get_search_score(&self, board: &mut Board) -> f64;
}

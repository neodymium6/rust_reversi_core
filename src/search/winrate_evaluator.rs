use std::fmt::Debug;

use crate::board::Board;

/// WinrateEvaluator trait.
/// # Note
/// * The score is 1.0 if the player is winning.
/// * The score is 0.0 if the opponent is winning.
/// * The score is 0.5 if the game is draw.
pub trait WinrateEvaluator: Send + Sync + Debug {
    fn evaluate(&self, board: &mut Board) -> f64;
}

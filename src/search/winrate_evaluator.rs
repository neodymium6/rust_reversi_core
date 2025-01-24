use std::fmt::Debug;

use crate::board::Board;

pub trait WinrateEvaluator: Send + Sync + Debug {
    fn evaluate(&self, board: &mut Board) -> f64;
}

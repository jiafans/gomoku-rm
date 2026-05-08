//! Game engine: shape recognition, position evaluation, search.

pub mod evaluate;
pub mod shape;

pub use evaluate::evaluate;

use crate::board::{Board, Color, Pos};

/// Strategic abstraction so we can later swap in mintaka / rapfi via IPC.
pub trait Engine: Send {
    fn best_move(&mut self, board: &Board, side: Color, time_budget_ms: u64) -> Pos;
    fn name(&self) -> &str;
}

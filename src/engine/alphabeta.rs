//! Negamax + alpha-beta + iterative deepening + candidate pruning + time budget.
//!
//! On rM2's i.MX7 Cortex-A7 @ 1GHz the practical depth is ~4-6 within a 3s
//! budget once we (a) prune candidates to empty cells within 2 of any existing
//! stone, and (b) order candidates by 1-ply evaluation so alpha-beta cuts hit
//! quickly.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::board::{Board, Color, Pos, BOARD_SIZE};
use crate::engine::evaluate::evaluate;
use crate::engine::shape::SCORE_FIVE;
use crate::engine::Engine;

/// Empty cells within this Chebyshev distance of any existing stone are
/// search candidates. 2 is the standard tradeoff (covers all "extension"
/// and "block" moves while keeping branching factor ~30 mid-game).
const SEARCH_RADIUS: i32 = 2;

/// Terminal score when the side-to-move sees a win in the position.
/// Multiply by depth-related factor so deeper wins beat shallower wins
/// (search prefers winning sooner, losing later).
const WIN_SCORE: i64 = SCORE_FIVE * 100;

pub struct AlphaBetaEngine {
    pub max_depth: u8,
    pub time_budget_ms: u64,
}

impl AlphaBetaEngine {
    pub fn new(max_depth: u8) -> Self {
        Self {
            max_depth,
            time_budget_ms: 3000,
        }
    }

    pub fn with_budget(max_depth: u8, time_budget_ms: u64) -> Self {
        Self {
            max_depth,
            time_budget_ms,
        }
    }
}

impl Engine for AlphaBetaEngine {
    fn name(&self) -> &str {
        "AlphaBeta"
    }

    fn best_move(&mut self, board: &Board, side: Color, time_budget_ms: u64) -> Pos {
        let deadline = Instant::now() + Duration::from_millis(time_budget_ms.max(50));

        // Empty board → 天元 (center).
        if board.history.is_empty() {
            return Pos::new(BOARD_SIZE / 2, BOARD_SIZE / 2);
        }

        let mut work = board.clone();
        let mut cands = candidates(&work);
        if cands.is_empty() {
            // Safety net: pick any empty.
            for c in 0..BOARD_SIZE {
                for r in 0..BOARD_SIZE {
                    let p = Pos::new(c, r);
                    if work.is_empty(p) {
                        return p;
                    }
                }
            }
            return Pos::new(BOARD_SIZE / 2, BOARD_SIZE / 2);
        }
        order_candidates(&mut work, &mut cands, side);

        let mut best = cands[0];
        let mut best_score = i64::MIN;

        // Iterative deepening: completed depths are kept; in-progress depth
        // discarded on time-out.
        for depth in 1..=self.max_depth {
            if Instant::now() >= deadline {
                break;
            }
            if let Some((p, s)) = root_search(&mut work, side, depth, deadline, &cands) {
                best = p;
                best_score = s;
                log::debug!(
                    "depth={} best=({},{}) score={}",
                    depth,
                    best.col(),
                    best.row(),
                    best_score
                );
                if best_score >= WIN_SCORE / 2 {
                    break; // already winning, no need to deepen
                }
            } else {
                break; // aborted by deadline
            }
        }

        log::info!(
            "ab side={:?} depth_target={} best=({},{}) score={}",
            side,
            self.max_depth,
            best.col(),
            best.row(),
            best_score
        );
        best
    }
}

/// Top-level search: returns Some(best, score) only if the depth completed.
fn root_search(
    board: &mut Board,
    side: Color,
    depth: u8,
    deadline: Instant,
    cands: &[Pos],
) -> Option<(Pos, i64)> {
    let mut alpha: i64 = i64::MIN + 1;
    let beta: i64 = i64::MAX - 1;
    let mut best = cands[0];

    for &p in cands {
        if Instant::now() >= deadline {
            return None;
        }
        if !board.is_empty(p) {
            continue;
        }
        board.place(p, side);
        // Quick win check before recursing
        let immediate = if board.check_five_at(p, side) {
            WIN_SCORE
        } else {
            -negamax(board, depth - 1, -beta, -alpha, side.opp(), deadline)
        };
        board.unplace_last();
        if immediate > alpha {
            alpha = immediate;
            best = p;
        }
    }
    Some((best, alpha))
}

/// Negamax with alpha-beta. Returns score from `side`'s perspective.
fn negamax(
    board: &mut Board,
    depth: u8,
    alpha: i64,
    beta: i64,
    side: Color,
    deadline: Instant,
) -> i64 {
    if Instant::now() >= deadline {
        return evaluate(board, side);
    }
    if depth == 0 {
        return evaluate(board, side);
    }
    // If opponent's previous move closed a five, it's already handled at the
    // caller side. We just check if the position is terminal in our view.
    if let Some((last_p, last_c)) = board.history.last().copied() {
        if board.check_five_at(last_p, last_c) {
            // Whoever just moved won. Score from current `side`'s view:
            return if last_c == side { WIN_SCORE } else { -WIN_SCORE };
        }
    }

    let mut cands = candidates(board);
    if cands.is_empty() {
        return evaluate(board, side);
    }
    order_candidates(board, &mut cands, side);

    let mut alpha = alpha;
    for p in cands {
        if Instant::now() >= deadline {
            return alpha;
        }
        board.place(p, side);
        let score = -negamax(board, depth - 1, -beta, -alpha, side.opp(), deadline);
        board.unplace_last();
        if score >= beta {
            return beta; // beta cutoff
        }
        if score > alpha {
            alpha = score;
        }
    }
    alpha
}

/// Empty cells within SEARCH_RADIUS Chebyshev of any existing stone.
pub fn candidates(board: &Board) -> Vec<Pos> {
    if board.history.is_empty() {
        return vec![Pos::new(BOARD_SIZE / 2, BOARD_SIZE / 2)];
    }
    let n = BOARD_SIZE as i32;
    let mut set: HashSet<(usize, usize)> = HashSet::new();
    for &(p, _) in &board.history {
        for dx in -SEARCH_RADIUS..=SEARCH_RADIUS {
            for dy in -SEARCH_RADIUS..=SEARCH_RADIUS {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = p.col() as i32 + dx;
                let ny = p.row() as i32 + dy;
                if (0..n).contains(&nx) && (0..n).contains(&ny) {
                    let np = Pos::new(nx as usize, ny as usize);
                    if board.is_empty(np) {
                        set.insert((nx as usize, ny as usize));
                    }
                }
            }
        }
    }
    set.into_iter().map(|(c, r)| Pos::new(c, r)).collect()
}

/// Sort candidates descending by 1-ply evaluation if `side` plays there.
/// Better move ordering → earlier alpha-beta cutoffs → much faster search.
fn order_candidates(board: &mut Board, cands: &mut Vec<Pos>, side: Color) {
    let mut scored: Vec<(Pos, i64)> = cands
        .iter()
        .map(|&p| {
            board.place(p, side);
            let s = evaluate(board, side);
            board.unplace_last();
            (p, s)
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    *cands = scored.into_iter().map(|(p, _)| p).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_board_picks_center() {
        let mut e = AlphaBetaEngine::new(2);
        let b = Board::new();
        let m = e.best_move(&b, Color::Black, 1000);
        assert_eq!(m, Pos::new(BOARD_SIZE / 2, BOARD_SIZE / 2));
    }

    #[test]
    fn ai_completes_five_when_possible() {
        // Black has 4 in a row; AI as Black should complete to 5.
        let mut b = Board::new();
        for c in 4..=7 {
            b.place(Pos::new(c, 9), Color::Black);
        }
        // Now opponent placed a blocker somewhere irrelevant
        b.place(Pos::new(0, 0), Color::White);
        let mut e = AlphaBetaEngine::new(2);
        let m = e.best_move(&b, Color::Black, 1000);
        // Either col=3 or col=8 (the two extension points to make 5)
        assert!(
            (m == Pos::new(3, 9)) || (m == Pos::new(8, 9)),
            "AI should complete the 5; got ({}, {})",
            m.col(),
            m.row()
        );
    }

    #[test]
    fn ai_blocks_opponent_five() {
        // White has 4 in a row; AI as Black must block.
        let mut b = Board::new();
        b.place(Pos::new(9, 9), Color::Black);
        for c in 4..=7 {
            b.place(Pos::new(c, 5), Color::White);
        }
        // It's Black's turn (history=5 → odd→ White's turn? wait let me think)
        // history.len()=5 (1 black + 4 white) → next is Black's turn. Good.
        let mut e = AlphaBetaEngine::new(3);
        let m = e.best_move(&b, Color::Black, 2000);
        assert!(
            (m == Pos::new(3, 5)) || (m == Pos::new(8, 5)),
            "AI should block white's 4; got ({}, {})",
            m.col(),
            m.row()
        );
    }
}

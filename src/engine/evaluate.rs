//! Position evaluation for Gomoku.
//!
//! Scores every line (row, col, diagonal, anti-diagonal) for both colors and
//! returns the difference (positive = `side` is better off).

use crate::board::{Board, Color, Pos, BOARD_SIZE};
use crate::engine::shape::score_line;

/// Encode a sequence of cells as the {O, X, _} ASCII for our pattern matcher,
/// with `X` padding on both ends to represent the board edge as opponent.
fn encode_line(cells: &[Option<Color>], me: Color) -> String {
    let mut s = String::with_capacity(cells.len() + 2);
    s.push('X');
    for c in cells {
        match c {
            None => s.push('_'),
            Some(x) if *x == me => s.push('O'),
            Some(_) => s.push('X'),
        }
    }
    s.push('X');
    s
}

/// Yield every contiguous line of length ≥ 5 (rows, cols, two diagonals).
/// Each line is a Vec<Option<Color>> of cells in order.
pub fn collect_lines(board: &Board) -> Vec<Vec<Option<Color>>> {
    let n = BOARD_SIZE as i32;
    let mut lines: Vec<Vec<Option<Color>>> = Vec::with_capacity(4 * BOARD_SIZE + 4);

    // Rows
    for r in 0..BOARD_SIZE {
        let row: Vec<Option<Color>> = (0..BOARD_SIZE)
            .map(|c| board.get(Pos::new(c, r)))
            .collect();
        lines.push(row);
    }
    // Columns
    for c in 0..BOARD_SIZE {
        let col: Vec<Option<Color>> = (0..BOARD_SIZE)
            .map(|r| board.get(Pos::new(c, r)))
            .collect();
        lines.push(col);
    }
    // Diagonals (\) — for each diagonal offset d = r - c, range -(n-1)..n
    for d in -(n - 1)..n {
        let mut diag = Vec::new();
        for c in 0..n {
            let r = c + d;
            if (0..n).contains(&r) {
                diag.push(board.get(Pos::new(c as usize, r as usize)));
            }
        }
        if diag.len() >= 5 {
            lines.push(diag);
        }
    }
    // Anti-diagonals (/) — r + c = s, range 0..(2n-1)
    for s in 0..(2 * n - 1) {
        let mut anti = Vec::new();
        for c in 0..n {
            let r = s - c;
            if (0..n).contains(&r) {
                anti.push(board.get(Pos::new(c as usize, r as usize)));
            }
        }
        if anti.len() >= 5 {
            lines.push(anti);
        }
    }
    lines
}

/// Sum pattern scores across every line, from `me`'s perspective.
fn score_for(board: &Board, me: Color) -> i64 {
    collect_lines(board)
        .into_iter()
        .map(|cells| score_line(&encode_line(&cells, me)))
        .sum()
}

/// `side`'s score minus opponent's. Positive = `side` better off.
pub fn evaluate(board: &Board, side: Color) -> i64 {
    score_for(board, side) - score_for(board, side.opp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn empty_board_zero() {
        let b = Board::new();
        assert_eq!(evaluate(&b, Color::Black), 0);
    }

    #[test]
    fn black_five_wins_eval() {
        let mut b = Board::new();
        for c in 4..=8 {
            b.place(Pos::new(c, 9), Color::Black);
        }
        let v = evaluate(&b, Color::Black);
        assert!(v >= 1_000_000, "black-five eval should be huge, got {v}");
    }

    #[test]
    fn white_five_loses_for_black() {
        let mut b = Board::new();
        for c in 4..=8 {
            b.place(Pos::new(c, 9), Color::White);
        }
        let v = evaluate(&b, Color::Black);
        assert!(v <= -1_000_000, "black sees white five as huge negative, got {v}");
    }

    #[test]
    fn open_three_is_positive() {
        let mut b = Board::new();
        for c in 5..=7 {
            b.place(Pos::new(c, 9), Color::Black);
        }
        let v = evaluate(&b, Color::Black);
        assert!(v > 0, "black open three should be positive, got {v}");
    }
}

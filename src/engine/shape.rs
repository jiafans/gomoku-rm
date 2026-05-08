//! Gomoku shape recognition + scoring.
//!
//! Lines are encoded as ASCII strings where:
//!   `O` = friendly stone, `X` = opponent stone or board edge, `_` = empty.
//! We pad each extracted line with `X` on both ends so that "edge of board"
//! plays the role of opponent for closing-pattern detection.
//!
//! Each line score is the weighted sum of pattern occurrences (overlapping
//! windowed match). Higher patterns dwarf lower ones via exponential scoring,
//! so any "double-counting" between five-contains-four etc. is harmless.
//!
//! Scoring table inspired by lihongxun945/gobang.

pub const SCORE_FIVE: i64 = 1_000_000;
pub const SCORE_OPEN_FOUR: i64 = 100_000;
pub const SCORE_FOUR: i64 = 10_000;
pub const SCORE_OPEN_THREE: i64 = 1_000;
pub const SCORE_THREE: i64 = 100;
pub const SCORE_OPEN_TWO: i64 = 100;
pub const SCORE_TWO: i64 = 10;

/// Count overlapping occurrences of `pat` as a substring of `line`.
/// (`str::matches` is non-overlapping, so we scan windowed manually.)
pub fn count_pattern(line: &str, pat: &str) -> u32 {
    let p = pat.as_bytes();
    let l = line.as_bytes();
    if p.is_empty() || p.len() > l.len() {
        return 0;
    }
    let mut count = 0u32;
    let last = l.len() - p.len();
    for i in 0..=last {
        if &l[i..i + p.len()] == p {
            count += 1;
        }
    }
    count
}

/// Sum scores of all known patterns in this encoded line.
pub fn score_line(line: &str) -> i64 {
    let mut score: i64 = 0;

    // ---- Five ----
    score += SCORE_FIVE * count_pattern(line, "OOOOO") as i64;

    // ---- Open four (winning next move regardless) ----
    score += SCORE_OPEN_FOUR * count_pattern(line, "_OOOO_") as i64;

    // ---- Four (closed four + broken four — forces a response) ----
    let four_pats = [
        "XOOOO_", "_OOOOX", // closed four
        "O_OOO", "OOO_O", "OO_OO", // broken four (4 stones with one gap, 1 placement → five)
    ];
    let mut four_count = 0u32;
    for pat in &four_pats {
        four_count += count_pattern(line, pat);
    }
    score += SCORE_FOUR * four_count as i64;

    // ---- Open three (forces response in any decent play) ----
    let open_three_pats = [
        "_OOO_",
        "_OO_O_",
        "_O_OO_",
    ];
    let mut open_three_count = 0u32;
    for pat in &open_three_pats {
        open_three_count += count_pattern(line, pat);
    }
    score += SCORE_OPEN_THREE * open_three_count as i64;

    // ---- Closed three (one end blocked) ----
    let three_pats = [
        "XOOO__", "__OOOX",
        "XOO_O_", "_O_OOX",
        "XO_OO_", "_OO_OX",
        "XOOO_X", "X_OOOX", // both ends restricted but length-3 still
    ];
    let mut three_count = 0u32;
    for pat in &three_pats {
        three_count += count_pattern(line, pat);
    }
    score += SCORE_THREE * three_count as i64;

    // ---- Open two ----
    let open_two_pats = [
        "__OO__",
        "_O_O_",
        "_OO_",
    ];
    let mut open_two_count = 0u32;
    for pat in &open_two_pats {
        open_two_count += count_pattern(line, pat);
    }
    score += SCORE_OPEN_TWO * open_two_count as i64;

    // ---- Closed two ----
    let two_pats = ["XOO__", "__OOX", "XO_O_", "_O_OX"];
    let mut two_count = 0u32;
    for pat in &two_pats {
        two_count += count_pattern(line, pat);
    }
    score += SCORE_TWO * two_count as i64;

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_overlapping() {
        assert_eq!(count_pattern("OOOOOO", "OOOOO"), 2);
        assert_eq!(count_pattern("OOOO", "OOOOO"), 0);
        assert_eq!(count_pattern("X_O_O_X", "_O_"), 2);
    }

    #[test]
    fn five_dominates() {
        let s = score_line("XX_OOOOO_XX");
        assert!(s >= SCORE_FIVE, "five-in-a-row should score >= {SCORE_FIVE}, got {s}");
    }

    #[test]
    fn open_four_outscores_closed_four() {
        let open = score_line("X__OOOO__X");
        let closed = score_line("XOOOO_XX_X"); // four with one closed end
        assert!(open > closed, "open four ({open}) should outscore closed four ({closed})");
    }

    #[test]
    fn open_three_present() {
        let s = score_line("XX_OOO_XX");
        assert!(s >= SCORE_OPEN_THREE, "open three should score, got {s}");
    }

    #[test]
    fn empty_line_zero() {
        assert_eq!(score_line("X__________X"), 0);
    }
}

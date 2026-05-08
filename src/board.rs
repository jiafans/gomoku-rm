//! 19×19 Gomoku board state, side tracking, history.

pub const BOARD_SIZE: usize = 19;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn opp(self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Pos {
    col: u8,
    row: u8,
}

impl Pos {
    pub fn new(col: usize, row: usize) -> Self {
        debug_assert!(col < BOARD_SIZE && row < BOARD_SIZE);
        Self {
            col: col as u8,
            row: row as u8,
        }
    }
    pub fn col(&self) -> usize {
        self.col as usize
    }
    pub fn row(&self) -> usize {
        self.row as usize
    }
}

#[derive(Clone)]
pub struct Board {
    cells: [[Option<Color>; BOARD_SIZE]; BOARD_SIZE],
    pub history: Vec<(Pos, Color)>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [[None; BOARD_SIZE]; BOARD_SIZE],
            history: Vec::with_capacity(BOARD_SIZE * BOARD_SIZE),
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, p: Pos) -> Option<Color> {
        self.cells[p.col()][p.row()]
    }

    pub fn is_empty(&self, p: Pos) -> bool {
        self.get(p).is_none()
    }

    pub fn place(&mut self, p: Pos, c: Color) -> bool {
        if !self.is_empty(p) {
            return false;
        }
        self.cells[p.col()][p.row()] = Some(c);
        self.history.push((p, c));
        true
    }

    pub fn unplace_last(&mut self) -> Option<(Pos, Color)> {
        let (p, c) = self.history.pop()?;
        self.cells[p.col()][p.row()] = None;
        Some((p, c))
    }

    /// Black plays first; sides alternate after.
    pub fn current_side(&self) -> Color {
        if self.history.len() % 2 == 0 {
            Color::Black
        } else {
            Color::White
        }
    }

    pub fn stones(&self) -> impl Iterator<Item = (Pos, Color)> + '_ {
        self.history.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_and_undo() {
        let mut b = Board::new();
        let p = Pos::new(9, 9);
        assert!(b.is_empty(p));
        assert!(b.place(p, Color::Black));
        assert_eq!(b.get(p), Some(Color::Black));
        assert_eq!(b.current_side(), Color::White);
        assert_eq!(b.unplace_last(), Some((p, Color::Black)));
        assert!(b.is_empty(p));
        assert_eq!(b.current_side(), Color::Black);
    }

    #[test]
    fn cannot_place_on_occupied() {
        let mut b = Board::new();
        let p = Pos::new(0, 0);
        assert!(b.place(p, Color::Black));
        assert!(!b.place(p, Color::White));
    }
}

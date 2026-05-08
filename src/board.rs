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

    /// Does placing `c` at `p` make a line of ≥5 same-color stones through `p`?
    /// Caller must ensure `p` already has color `c` on the board.
    pub fn check_five_at(&self, p: Pos, c: Color) -> bool {
        const DIRS: [(i32, i32); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
        for &(dx, dy) in &DIRS {
            let mut count = 1;
            // forward
            let (mut x, mut y) = (p.col() as i32 + dx, p.row() as i32 + dy);
            while (0..BOARD_SIZE as i32).contains(&x)
                && (0..BOARD_SIZE as i32).contains(&y)
                && self.cells[x as usize][y as usize] == Some(c)
            {
                count += 1;
                x += dx;
                y += dy;
            }
            // backward
            let (mut x, mut y) = (p.col() as i32 - dx, p.row() as i32 - dy);
            while (0..BOARD_SIZE as i32).contains(&x)
                && (0..BOARD_SIZE as i32).contains(&y)
                && self.cells[x as usize][y as usize] == Some(c)
            {
                count += 1;
                x -= dx;
                y -= dy;
            }
            if count >= 5 {
                return true;
            }
        }
        false
    }

    /// Returns the winning color if the most recent move closed a 5-in-a-row.
    pub fn winner(&self) -> Option<Color> {
        let (p, c) = self.history.last().copied()?;
        if self.check_five_at(p, c) {
            Some(c)
        } else {
            None
        }
    }

    /// Reset to empty board.
    pub fn reset(&mut self) {
        self.cells = [[None; BOARD_SIZE]; BOARD_SIZE];
        self.history.clear();
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

    #[test]
    fn five_horizontal_wins() {
        let mut b = Board::new();
        for c in 4..=8 {
            b.place(Pos::new(c, 9), Color::Black);
        }
        assert!(b.check_five_at(Pos::new(8, 9), Color::Black));
        assert_eq!(b.winner(), Some(Color::Black));
    }

    #[test]
    fn five_vertical_wins() {
        let mut b = Board::new();
        for r in 0..5 {
            b.place(Pos::new(3, r), Color::White);
        }
        assert!(b.check_five_at(Pos::new(3, 4), Color::White));
    }

    #[test]
    fn five_diagonal_wins() {
        let mut b = Board::new();
        for i in 0..5 {
            b.place(Pos::new(i, i), Color::Black);
        }
        assert!(b.check_five_at(Pos::new(4, 4), Color::Black));
    }

    #[test]
    fn five_anti_diagonal_wins() {
        let mut b = Board::new();
        for i in 0..5 {
            b.place(Pos::new(i, 4 - i), Color::Black);
        }
        assert!(b.check_five_at(Pos::new(4, 0), Color::Black));
    }

    #[test]
    fn four_is_not_win() {
        let mut b = Board::new();
        for c in 0..4 {
            b.place(Pos::new(c, 5), Color::Black);
        }
        assert!(!b.check_five_at(Pos::new(3, 5), Color::Black));
    }

    #[test]
    fn six_in_a_row_is_win() {
        let mut b = Board::new();
        for c in 2..=7 {
            b.place(Pos::new(c, 0), Color::White);
        }
        assert!(b.check_five_at(Pos::new(7, 0), Color::White));
    }
}

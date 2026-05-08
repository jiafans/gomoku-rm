//! Single-slot game persistence at /home/root/.config/gomoku-rm/savestate.yml.
//!
//! Auto-saved after each move; loaded on startup. Decouples Board/GameMode
//! from serde: we round-trip via simple data classes here.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::board::{Board, Color, Pos};
use crate::scene::game::GameMode;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SColor {
    Black,
    White,
}

impl From<Color> for SColor {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => SColor::Black,
            Color::White => SColor::White,
        }
    }
}

impl From<SColor> for Color {
    fn from(c: SColor) -> Self {
        match c {
            SColor::Black => Color::Black,
            SColor::White => Color::White,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SMove {
    pub col: u8,
    pub row: u8,
    pub color: SColor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum SMode {
    Pvp,
    PvAi {
        ai: SColor,
        depth: u8,
        time_budget_ms: u64,
    },
}

impl From<GameMode> for SMode {
    fn from(m: GameMode) -> Self {
        match m {
            GameMode::Pvp => SMode::Pvp,
            GameMode::PvAi {
                ai,
                depth,
                time_budget_ms,
            } => SMode::PvAi {
                ai: ai.into(),
                depth,
                time_budget_ms,
            },
        }
    }
}

impl From<SMode> for GameMode {
    fn from(m: SMode) -> Self {
        match m {
            SMode::Pvp => GameMode::Pvp,
            SMode::PvAi {
                ai,
                depth,
                time_budget_ms,
            } => GameMode::PvAi {
                ai: ai.into(),
                depth,
                time_budget_ms,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Savestate {
    pub history: Vec<SMove>,
    pub mode: SMode,
}

impl Savestate {
    pub fn from_board(board: &Board, mode: GameMode) -> Self {
        let history = board
            .history
            .iter()
            .map(|(p, c)| SMove {
                col: p.col() as u8,
                row: p.row() as u8,
                color: (*c).into(),
            })
            .collect();
        Self {
            history,
            mode: mode.into(),
        }
    }

    pub fn restore_into(&self, board: &mut Board) {
        board.reset();
        for m in &self.history {
            board.place(Pos::new(m.col as usize, m.row as usize), m.color.into());
        }
    }
}

pub fn savestate_path() -> PathBuf {
    PathBuf::from("/home/root/.config/gomoku-rm/savestate.yml")
}

pub fn save(state: &Savestate) -> anyhow::Result<()> {
    let path = savestate_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(state)?;
    fs::write(&path, yaml)?;
    Ok(())
}

pub fn load() -> Option<Savestate> {
    let path = savestate_path();
    let bytes = fs::read(&path).ok()?;
    serde_yaml::from_slice(&bytes).ok()
}

pub fn delete() {
    let _ = fs::remove_file(savestate_path());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_color() {
        for c in [Color::Black, Color::White] {
            let s: SColor = c.into();
            let back: Color = s.into();
            assert_eq!(c, back);
        }
    }

    #[test]
    fn roundtrip_savestate() {
        let mut b = Board::new();
        b.place(Pos::new(9, 9), Color::Black);
        b.place(Pos::new(8, 8), Color::White);
        let mode = GameMode::PvAi {
            ai: Color::White,
            depth: 4,
            time_budget_ms: 3000,
        };
        let s = Savestate::from_board(&b, mode);
        let yaml = serde_yaml::to_string(&s).unwrap();
        let back: Savestate = serde_yaml::from_str(&yaml).unwrap();
        let mut b2 = Board::new();
        back.restore_into(&mut b2);
        assert_eq!(b2.history.len(), 2);
        assert_eq!(b2.get(Pos::new(9, 9)), Some(Color::Black));
        assert_eq!(b2.get(Pos::new(8, 8)), Some(Color::White));
    }
}

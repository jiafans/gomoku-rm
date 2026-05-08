//! Game scene: draw board + handle touches to place stones.

use crate::board::{Board, Color, Pos, BOARD_SIZE};
use crate::canvas::{Canvas, SCREEN_W};
use crate::scene::Scene;
use libremarkable::framebuffer::common::{color, mxcfb_rect};
use libremarkable::input::{InputEvent, MultitouchEvent};
use std::time::Instant;

// ---- Geometry ----
pub const CELL_PX: i32 = 74;
pub const BOARD_PX: i32 = CELL_PX * (BOARD_SIZE as i32 - 1); // 1332
pub const BOARD_LEFT: i32 = (SCREEN_W - BOARD_PX) / 2; // 36
pub const BOARD_TOP: i32 = 280;
pub const STONE_RADIUS: u32 = 30;
pub const HIT_RADIUS: i32 = 40;

// Letters A..T skipping I (Go convention)
const COL_LABELS: [char; BOARD_SIZE] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
];
const STAR_COLS: [usize; 3] = [3, 9, 15];

#[inline]
pub fn grid_to_px(p: Pos) -> (i32, i32) {
    (
        BOARD_LEFT + p.col() as i32 * CELL_PX,
        BOARD_TOP + p.row() as i32 * CELL_PX,
    )
}

/// Convert (px,py) screen coords to nearest grid intersection if within HIT_RADIUS.
pub fn px_to_grid(x: i32, y: i32) -> Option<Pos> {
    let col_f = (x - BOARD_LEFT) as f32 / CELL_PX as f32;
    let row_f = (y - BOARD_TOP) as f32 / CELL_PX as f32;
    let col = col_f.round() as i32;
    let row = row_f.round() as i32;
    if !(0..BOARD_SIZE as i32).contains(&col) || !(0..BOARD_SIZE as i32).contains(&row) {
        return None;
    }
    let cx = BOARD_LEFT + col * CELL_PX;
    let cy = BOARD_TOP + row * CELL_PX;
    let dx = x - cx;
    let dy = y - cy;
    if dx * dx + dy * dy > HIT_RADIUS * HIT_RADIUS {
        return None;
    }
    Some(Pos::new(col as usize, row as usize))
}

pub struct GameScene {
    board: Board,
    needs_full_redraw: bool,
    last_drawn_history_len: usize,
    started_at: Instant,
}

impl GameScene {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            needs_full_redraw: true,
            last_drawn_history_len: 0,
            started_at: Instant::now(),
        }
    }

    fn draw_board_grid(canvas: &mut Canvas) {
        // 19 horizontal + 19 vertical lines
        for i in 0..BOARD_SIZE {
            let p1 = grid_to_px(Pos::new(0, i));
            let p2 = grid_to_px(Pos::new(BOARD_SIZE - 1, i));
            canvas.draw_line(p1.0, p1.1, p2.0, p2.1, 2);

            let p1 = grid_to_px(Pos::new(i, 0));
            let p2 = grid_to_px(Pos::new(i, BOARD_SIZE - 1));
            canvas.draw_line(p1.0, p1.1, p2.0, p2.1, 2);
        }
        // 9 star points
        for &c in &STAR_COLS {
            for &r in &STAR_COLS {
                let (x, y) = grid_to_px(Pos::new(c, r));
                canvas.fill_circle(x, y, 5, color::BLACK);
            }
        }
        // Column labels (top + bottom)
        for (col, ch) in COL_LABELS.iter().enumerate() {
            let (cx, _) = grid_to_px(Pos::new(col, 0));
            canvas.draw_text(cx - 12, BOARD_TOP - 30, &ch.to_string(), 32.0);
            canvas.draw_text(cx - 12, BOARD_TOP + BOARD_PX + 60, &ch.to_string(), 32.0);
        }
        // Row labels (left + right). Go convention: row 1 at bottom of board.
        for row in 0..BOARD_SIZE {
            let display_num = BOARD_SIZE - row;
            let (_, cy) = grid_to_px(Pos::new(0, row));
            canvas.draw_text(BOARD_LEFT - 50, cy + 10, &display_num.to_string(), 32.0);
            canvas.draw_text(
                BOARD_LEFT + BOARD_PX + 20,
                cy + 10,
                &display_num.to_string(),
                32.0,
            );
        }
    }

    fn draw_stone(canvas: &mut Canvas, pos: Pos, c: Color) {
        let (px, py) = grid_to_px(pos);
        let (fill, ring) = match c {
            Color::Black => (color::BLACK, color::BLACK),
            Color::White => (color::WHITE, color::BLACK),
        };
        canvas.fill_circle(px, py, STONE_RADIUS, fill);
        // Outline so white stones are visible against the (white) board background.
        canvas.draw_circle(px, py, STONE_RADIUS, ring);
    }

    fn stone_region(pos: Pos) -> mxcfb_rect {
        let (px, py) = grid_to_px(pos);
        let r = STONE_RADIUS as i32 + 4;
        mxcfb_rect {
            top: (py - r).max(0) as u32,
            left: (px - r).max(0) as u32,
            width: (2 * r) as u32,
            height: (2 * r) as u32,
        }
    }
}

impl Scene for GameScene {
    fn on_input(&mut self, event: InputEvent) {
        // Time gate: ignore the launching tap (AppLoad spawns us mid-touch).
        if self.started_at.elapsed().as_millis() < 250 {
            return;
        }
        if let InputEvent::MultitouchEvent {
            event: MultitouchEvent::Press { finger },
        } = event
        {
            let x = finger.pos.x as i32;
            let y = finger.pos.y as i32;
            if let Some(pos) = px_to_grid(x, y) {
                if self.board.is_empty(pos) {
                    let side = self.board.current_side();
                    self.board.place(pos, side);
                    log::info!(
                        "placed {:?} at ({}, {})",
                        side,
                        pos.col(),
                        pos.row()
                    );
                }
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        if self.needs_full_redraw {
            canvas.clear();
            canvas.draw_text(540, 130, "Gomoku", 80.0);
            Self::draw_board_grid(canvas);
            for (pos, c) in self.board.stones() {
                Self::draw_stone(canvas, pos, c);
            }
            canvas.full_refresh();
            self.needs_full_redraw = false;
            self.last_drawn_history_len = self.board.history.len();
            return;
        }

        // Incremental: render stones placed since last draw with partial refresh.
        let new_moves: Vec<(Pos, Color)> = self.board.history
            [self.last_drawn_history_len..]
            .iter()
            .copied()
            .collect();
        for (pos, c) in new_moves {
            Self::draw_stone(canvas, pos, c);
            canvas.partial_refresh(Self::stone_region(pos));
        }
        self.last_drawn_history_len = self.board.history.len();
    }
}

use libremarkable::framebuffer::cgmath::{Point2, Vector2};
use libremarkable::framebuffer::common::{
    color, display_temp, dither_mode, waveform_mode, DISPLAYHEIGHT, DISPLAYWIDTH,
};
use libremarkable::framebuffer::core::Framebuffer;
use libremarkable::framebuffer::{FramebufferDraw, FramebufferRefresh};
use libremarkable::input::{ev::EvDevContext, InputDevice, InputEvent, MultitouchEvent};
use std::sync::mpsc::channel;

// ---- Board geometry ----
const BOARD_SIZE: usize = 19;
const CELL_PX: i32 = 74;
const BOARD_PX: i32 = CELL_PX * (BOARD_SIZE as i32 - 1); // 1332
const BOARD_LEFT: i32 = (DISPLAYWIDTH as i32 - BOARD_PX) / 2; // 36
const BOARD_TOP: i32 = 280;
const STAR_RADIUS: u32 = 5;
const STAR_COLS: [usize; 3] = [3, 9, 15]; // 0-indexed

// Letters A..T skipping I (Go convention)
const COL_LABELS: [char; BOARD_SIZE] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T',
];

#[inline]
fn grid_to_px(col: usize, row: usize) -> Point2<i32> {
    Point2 {
        x: BOARD_LEFT + col as i32 * CELL_PX,
        y: BOARD_TOP + row as i32 * CELL_PX,
    }
}

fn draw_board(fb: &mut Framebuffer) {
    fb.clear();

    // Title
    fb.draw_text(
        Point2 { x: 480.0, y: 130.0 },
        "Gomoku",
        80.0,
        color::BLACK,
        false,
    );

    // 19 horizontal lines
    for row in 0..BOARD_SIZE {
        let p1 = grid_to_px(0, row);
        let p2 = grid_to_px(BOARD_SIZE - 1, row);
        fb.draw_line(p1, p2, 2, color::BLACK);
    }
    // 19 vertical lines
    for col in 0..BOARD_SIZE {
        let p1 = grid_to_px(col, 0);
        let p2 = grid_to_px(col, BOARD_SIZE - 1);
        fb.draw_line(p1, p2, 2, color::BLACK);
    }

    // 9 star points
    for &c in &STAR_COLS {
        for &r in &STAR_COLS {
            fb.fill_circle(grid_to_px(c, r), STAR_RADIUS, color::BLACK);
        }
    }

    // Column labels (top + bottom)
    for (col, ch) in COL_LABELS.iter().enumerate() {
        let center = grid_to_px(col, 0);
        // Above board
        fb.draw_text(
            Point2 {
                x: (center.x - 12) as f32,
                y: (BOARD_TOP - 30) as f32,
            },
            &ch.to_string(),
            32.0,
            color::BLACK,
            false,
        );
        // Below board
        fb.draw_text(
            Point2 {
                x: (center.x - 12) as f32,
                y: (BOARD_TOP + BOARD_PX + 60) as f32,
            },
            &ch.to_string(),
            32.0,
            color::BLACK,
            false,
        );
    }

    // Row labels (left + right). Go convention: row 1 at bottom.
    for row in 0..BOARD_SIZE {
        let display_num = BOARD_SIZE - row; // 0->19, 18->1
        let center = grid_to_px(0, row);
        let label = display_num.to_string();
        // Left of board
        fb.draw_text(
            Point2 {
                x: (BOARD_LEFT - 50) as f32,
                y: (center.y + 10) as f32,
            },
            &label,
            32.0,
            color::BLACK,
            false,
        );
        // Right of board
        fb.draw_text(
            Point2 {
                x: (BOARD_LEFT + BOARD_PX + 20) as f32,
                y: (center.y + 10) as f32,
            },
            &label,
            32.0,
            color::BLACK,
            false,
        );
    }

    // Hint at bottom
    fb.draw_text(
        Point2 {
            x: 380.0,
            y: (BOARD_TOP + BOARD_PX + 160) as f32,
        },
        "Tap anywhere to exit (M1: static board)",
        32.0,
        color::BLACK,
        false,
    );
}

fn main() {
    env_logger::init();
    log::info!(
        "gomoku-rm M1: static 19x19 board on {}x{}",
        DISPLAYWIDTH,
        DISPLAYHEIGHT
    );

    let mut fb = Framebuffer::new();
    draw_board(&mut fb);

    fb.full_refresh(
        waveform_mode::WAVEFORM_MODE_GC16,
        display_temp::TEMP_USE_REMARKABLE_DRAW,
        dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
        0,
        true,
    );

    let (tx, rx) = channel::<InputEvent>();
    EvDevContext::new(InputDevice::Multitouch, tx).start();

    for event in rx {
        if let InputEvent::MultitouchEvent {
            event: MultitouchEvent::Press { .. },
        } = event
        {
            log::info!("touch detected, exiting");
            break;
        }
    }

    fb.clear();
    fb.full_refresh(
        waveform_mode::WAVEFORM_MODE_GC16,
        display_temp::TEMP_USE_REMARKABLE_DRAW,
        dither_mode::EPDC_FLAG_USE_DITHERING_PASSTHROUGH,
        0,
        true,
    );

    let _ = Vector2::<i32> { x: 0, y: 0 }; // silence unused import
}

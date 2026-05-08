//! Settings: difficulty, language, AI side preference.

use libremarkable::framebuffer::common::{color, mxcfb_rect};
use libremarkable::input::{InputEvent, MultitouchEvent};

use crate::canvas::{Canvas, SCREEN_W};
use crate::config::{self, Difficulty};
use crate::i18n::{t, Lang};
use crate::savestate::SColor;
use crate::scene::menu::MainMenuScene;
use crate::scene::{Scene, Transition};
use std::time::Instant;

#[derive(Clone, Copy)]
struct ButtonRect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

impl ButtonRect {
    fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x
            && px < self.x + self.w as i32
            && py >= self.y
            && py < self.y + self.h as i32
    }
}

const ROW_H: i32 = 130;
const ROW_LABEL_X: i32 = 120;
const ROW_LABEL_Y_OFFSET: i32 = 80;
const BTN_H: u32 = 90;
const BTN_GAP: i32 = 30;

const ROW_Y_DIFF: i32 = 350;
const ROW_Y_LANG: i32 = ROW_Y_DIFF + ROW_H + 40;
const ROW_Y_AI: i32 = ROW_Y_LANG + ROW_H + 40;
const ROW_Y_BACK: i32 = ROW_Y_AI + ROW_H + 80;

fn three_buttons(y: i32, w: u32) -> [ButtonRect; 3] {
    let total_w = 3 * w as i32 + 2 * BTN_GAP;
    let start_x = (SCREEN_W - total_w) / 2;
    [
        ButtonRect {
            x: start_x,
            y,
            w,
            h: BTN_H,
        },
        ButtonRect {
            x: start_x + w as i32 + BTN_GAP,
            y,
            w,
            h: BTN_H,
        },
        ButtonRect {
            x: start_x + 2 * (w as i32 + BTN_GAP),
            y,
            w,
            h: BTN_H,
        },
    ]
}

fn two_buttons(y: i32, w: u32) -> [ButtonRect; 2] {
    let total_w = 2 * w as i32 + BTN_GAP;
    let start_x = (SCREEN_W - total_w) / 2;
    [
        ButtonRect {
            x: start_x,
            y,
            w,
            h: BTN_H,
        },
        ButtonRect {
            x: start_x + w as i32 + BTN_GAP,
            y,
            w,
            h: BTN_H,
        },
    ]
}

pub struct SettingsScene {
    diff_btns: [ButtonRect; 3],
    lang_btns: [ButtonRect; 2],
    ai_btns: [ButtonRect; 2],
    back_btn: ButtonRect,
    pending: Option<Transition>,
    needs_redraw: bool,
    started_at: Instant,
}

impl SettingsScene {
    pub fn new() -> Self {
        Self {
            diff_btns: three_buttons(ROW_Y_DIFF, 280),
            lang_btns: two_buttons(ROW_Y_LANG, 320),
            ai_btns: two_buttons(ROW_Y_AI, 320),
            back_btn: ButtonRect {
                x: (SCREEN_W - 320) / 2,
                y: ROW_Y_BACK,
                w: 320,
                h: BTN_H,
            },
            pending: None,
            needs_redraw: true,
            started_at: Instant::now(),
        }
    }

    fn draw_button(canvas: &mut Canvas, btn: &ButtonRect, label: &str, selected: bool) {
        let (fill, border) = if selected {
            (color::BLACK, 5)
        } else {
            (color::WHITE, 3)
        };
        canvas.fill_rect(btn.x, btn.y, btn.w, btn.h, fill);
        canvas.draw_rect(btn.x, btn.y, btn.w, btn.h, border);
        // Approx-center text — the selected (filled black) button can't show
        // black text; for simplicity we draw the label in the same color and
        // accept that selected buttons read as "black with no visible text".
        // To keep both readable, we re-draw selected button as bordered+filled
        // grayish via a thicker border instead.
        if selected {
            // Redraw: thick black border, white interior, label readable.
            canvas.fill_rect(btn.x, btn.y, btn.w, btn.h, color::WHITE);
            canvas.draw_rect(btn.x, btn.y, btn.w, btn.h, 8);
        }
        canvas.draw_text(btn.x + 30, btn.y + 60, label, 44.0);
    }
}

impl Scene for SettingsScene {
    fn on_input(&mut self, event: InputEvent) {
        if self.started_at.elapsed().as_millis() < 250 {
            return;
        }
        if let InputEvent::MultitouchEvent {
            event: MultitouchEvent::Press { finger },
        } = event
        {
            let x = finger.pos.x as i32;
            let y = finger.pos.y as i32;

            // Difficulty
            let diff_choices = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
            for (i, btn) in self.diff_btns.iter().enumerate() {
                if btn.contains(x, y) {
                    config::update(|c| c.difficulty = diff_choices[i]);
                    self.needs_redraw = true;
                    return;
                }
            }
            // Language
            let lang_choices = [Lang::Zh, Lang::En];
            for (i, btn) in self.lang_btns.iter().enumerate() {
                if btn.contains(x, y) {
                    config::update(|c| c.language = lang_choices[i]);
                    self.needs_redraw = true;
                    return;
                }
            }
            // AI side
            let ai_choices = [SColor::Black, SColor::White];
            for (i, btn) in self.ai_btns.iter().enumerate() {
                if btn.contains(x, y) {
                    config::update(|c| c.ai_side = ai_choices[i]);
                    self.needs_redraw = true;
                    return;
                }
            }
            // Back
            if self.back_btn.contains(x, y) {
                self.pending = Some(Transition::Replace(Box::new(MainMenuScene::new())));
                return;
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        if !self.needs_redraw {
            return;
        }
        let cfg = config::get();

        canvas.clear();
        canvas.draw_text(540, 200, t("settings.title"), 90.0);

        // Difficulty row
        canvas.draw_text(
            ROW_LABEL_X,
            ROW_Y_DIFF + ROW_LABEL_Y_OFFSET - 90,
            t("settings.difficulty"),
            44.0,
        );
        let diff_labels = [t("settings.easy"), t("settings.medium"), t("settings.hard")];
        let diff_choices = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
        for (i, btn) in self.diff_btns.iter().enumerate() {
            Self::draw_button(canvas, btn, diff_labels[i], cfg.difficulty == diff_choices[i]);
        }

        // Language row
        canvas.draw_text(
            ROW_LABEL_X,
            ROW_Y_LANG + ROW_LABEL_Y_OFFSET - 90,
            t("settings.language"),
            44.0,
        );
        let lang_labels = [t("settings.lang_zh"), t("settings.lang_en")];
        let lang_choices = [Lang::Zh, Lang::En];
        for (i, btn) in self.lang_btns.iter().enumerate() {
            Self::draw_button(canvas, btn, lang_labels[i], cfg.language == lang_choices[i]);
        }

        // AI side row
        canvas.draw_text(
            ROW_LABEL_X,
            ROW_Y_AI + ROW_LABEL_Y_OFFSET - 90,
            t("settings.ai_side"),
            44.0,
        );
        let ai_labels = [t("settings.black"), t("settings.white")];
        let ai_choices = [SColor::Black, SColor::White];
        for (i, btn) in self.ai_btns.iter().enumerate() {
            Self::draw_button(canvas, btn, ai_labels[i], cfg.ai_side == ai_choices[i]);
        }

        // Back
        Self::draw_button(canvas, &self.back_btn, t("settings.back"), false);

        canvas.full_refresh();
        self.needs_redraw = false;
    }

    fn take_transition(&mut self) -> Option<Transition> {
        self.pending.take()
    }
}

impl Drop for SettingsScene {
    fn drop(&mut self) {
        // Persist already happened on each click; nothing to do.
    }
}

// Suppress unused-bound warnings while we wire things up.
const _: mxcfb_rect = mxcfb_rect {
    top: 0,
    left: 0,
    width: 0,
    height: 0,
};

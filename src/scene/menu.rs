//! Main menu: launch new game, resume, settings, exit.

use libremarkable::framebuffer::common::{color, mxcfb_rect};
use libremarkable::input::{InputEvent, MultitouchEvent};

use crate::canvas::{Canvas, SCREEN_W};
use crate::config;
use crate::i18n::t;
use crate::savestate;
use crate::scene::game::{GameMode, GameScene};
use crate::scene::settings::SettingsScene;
use crate::scene::{Scene, Transition};
use std::time::Instant;

const BTN_W: u32 = 600;
const BTN_H: u32 = 110;
const BTN_X: i32 = (SCREEN_W - BTN_W as i32) / 2;
const BTN_Y0: i32 = 500;
const BTN_GAP: i32 = 30;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuItem {
    PlayAi,
    PlayLocal,
    Continue,
    Settings,
    Exit,
}

struct Item {
    kind: MenuItem,
    rect: mxcfb_rect,
    label_key: &'static str,
}

pub struct MainMenuScene {
    items: Vec<Item>,
    pending: Option<Transition>,
    needs_redraw: bool,
    started_at: Instant,
}

impl MainMenuScene {
    pub fn new() -> Self {
        let savestate_exists = savestate::load().is_some();

        let mut items: Vec<Item> = Vec::new();
        let mut y = BTN_Y0;
        let mut push = |kind: MenuItem, label_key: &'static str, y: &mut i32| {
            items.push(Item {
                kind,
                rect: mxcfb_rect {
                    top: *y as u32,
                    left: BTN_X as u32,
                    width: BTN_W,
                    height: BTN_H,
                },
                label_key,
            });
            *y += BTN_H as i32 + BTN_GAP;
        };
        push(MenuItem::PlayAi, "menu.play_ai", &mut y);
        push(MenuItem::PlayLocal, "menu.play_local", &mut y);
        if savestate_exists {
            push(MenuItem::Continue, "menu.continue", &mut y);
        }
        push(MenuItem::Settings, "menu.settings", &mut y);
        push(MenuItem::Exit, "menu.exit", &mut y);

        Self {
            items,
            pending: None,
            needs_redraw: true,
            started_at: Instant::now(),
        }
    }

    fn handle_pick(&mut self, item: MenuItem) {
        let cfg = config::get();
        let next: Transition = match item {
            MenuItem::PlayAi => {
                let mode = config::make_game_mode_pvai(&cfg);
                // Wipe stale savestate so AI mode starts fresh.
                savestate::delete();
                Transition::Replace(Box::new(GameScene::with_mode(mode)))
            }
            MenuItem::PlayLocal => {
                savestate::delete();
                Transition::Replace(Box::new(GameScene::with_mode(GameMode::Pvp)))
            }
            MenuItem::Continue => {
                if let Some(state) = savestate::load() {
                    let mut b = crate::board::Board::new();
                    state.restore_into(&mut b);
                    Transition::Replace(Box::new(GameScene::resume(state.mode.into(), b)))
                } else {
                    return; // savestate vanished — stay on menu
                }
            }
            MenuItem::Settings => Transition::Replace(Box::new(SettingsScene::new())),
            MenuItem::Exit => Transition::Exit,
        };
        self.pending = Some(next);
    }
}

impl Scene for MainMenuScene {
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
            for it in &self.items {
                if x >= it.rect.left as i32
                    && x < (it.rect.left as i32 + it.rect.width as i32)
                    && y >= it.rect.top as i32
                    && y < (it.rect.top as i32 + it.rect.height as i32)
                {
                    let kind = it.kind;
                    self.handle_pick(kind);
                    return;
                }
            }
        }
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        if !self.needs_redraw {
            return;
        }
        canvas.clear();
        // Title
        canvas.draw_text(540, 280, t("menu.title"), 100.0);
        // Buttons
        for it in &self.items {
            let r = &it.rect;
            canvas.fill_rect(r.left as i32, r.top as i32, r.width, r.height, color::WHITE);
            canvas.draw_rect(r.left as i32, r.top as i32, r.width, r.height, 4);
            canvas.draw_text(
                r.left as i32 + 60,
                r.top as i32 + 75,
                t(it.label_key),
                52.0,
            );
        }
        canvas.full_refresh();
        self.needs_redraw = false;
    }

    fn take_transition(&mut self) -> Option<Transition> {
        self.pending.take()
    }
}

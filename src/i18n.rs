//! Bilingual UI string lookup.
//!
//! Note: rendering CJK glyphs requires libremarkable to be using a font with
//! Chinese coverage (e.g. LXGW WenKai). The default embedded font is Latin-only.
//! On rM2 we ship LXGW into ~/.fonts; future work may load it directly via
//! rusttype to bypass libremarkable's hardcoded font.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Zh,
    En,
}

static CURRENT: Mutex<Lang> = Mutex::new(Lang::En);

pub fn set_lang(l: Lang) {
    *CURRENT.lock().unwrap() = l;
}

pub fn current() -> Lang {
    *CURRENT.lock().unwrap()
}

/// Lookup a UI string by key. Falls back to the key itself if missing.
/// Lifetime of the input flows through so that the fallback `key` is valid.
pub fn t<'a>(key: &'a str) -> &'a str {
    let lang = current();
    match (lang, key) {
        // ---- Main menu ----
        (Lang::Zh, "menu.title") => "五子棋",
        (Lang::En, "menu.title") => "Gomoku",
        (Lang::Zh, "menu.play_ai") => "人机对战",
        (Lang::En, "menu.play_ai") => "Play vs AI",
        (Lang::Zh, "menu.play_local") => "本地双人",
        (Lang::En, "menu.play_local") => "Local 2P",
        (Lang::Zh, "menu.continue") => "继续棋局",
        (Lang::En, "menu.continue") => "Continue",
        (Lang::Zh, "menu.settings") => "设置",
        (Lang::En, "menu.settings") => "Settings",
        (Lang::Zh, "menu.exit") => "退出",
        (Lang::En, "menu.exit") => "Exit",

        // ---- Settings ----
        (Lang::Zh, "settings.title") => "设置",
        (Lang::En, "settings.title") => "Settings",
        (Lang::Zh, "settings.difficulty") => "难度",
        (Lang::En, "settings.difficulty") => "Difficulty",
        (Lang::Zh, "settings.easy") => "简单",
        (Lang::En, "settings.easy") => "Easy",
        (Lang::Zh, "settings.medium") => "中等",
        (Lang::En, "settings.medium") => "Medium",
        (Lang::Zh, "settings.hard") => "困难",
        (Lang::En, "settings.hard") => "Hard",
        (Lang::Zh, "settings.language") => "语言",
        (Lang::En, "settings.language") => "Language",
        (Lang::Zh, "settings.lang_zh") => "中文",
        (Lang::En, "settings.lang_zh") => "中文",
        (Lang::Zh, "settings.lang_en") => "English",
        (Lang::En, "settings.lang_en") => "English",
        (Lang::Zh, "settings.ai_side") => "AI 执子",
        (Lang::En, "settings.ai_side") => "AI plays",
        (Lang::Zh, "settings.black") => "黑棋",
        (Lang::En, "settings.black") => "Black",
        (Lang::Zh, "settings.white") => "白棋",
        (Lang::En, "settings.white") => "White",
        (Lang::Zh, "settings.back") => "返回",
        (Lang::En, "settings.back") => "Back",

        // ---- Game ----
        (Lang::Zh, "game.undo") => "悔棋",
        (Lang::En, "game.undo") => "Undo",
        (Lang::Zh, "game.save_quit") => "保存退出",
        (Lang::En, "game.save_quit") => "Save & Quit",
        (Lang::Zh, "game.thinking") => "AI 思考中…",
        (Lang::En, "game.thinking") => "AI thinking…",
        (Lang::Zh, "game.win_black") => "黑棋胜",
        (Lang::En, "game.win_black") => "Black Wins",
        (Lang::Zh, "game.win_white") => "白棋胜",
        (Lang::En, "game.win_white") => "White Wins",
        (Lang::Zh, "game.tap_again") => "点击重开一局",
        (Lang::En, "game.tap_again") => "Tap to play again",

        _ => key,
    }
}

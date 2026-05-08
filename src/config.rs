//! Persistent app configuration: language, difficulty, AI color preference.
//!
//! Stored at `/home/root/.config/gomoku-rm/config.yml`. Loaded once at startup
//! into the global Mutex; mutated by SettingsScene; saved on each change.

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::board::Color;
use crate::i18n::{self, Lang};
use crate::savestate::SColor;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Search depth used by the Alpha-Beta engine.
    pub fn depth(self) -> u8 {
        match self {
            Difficulty::Easy => 2,
            Difficulty::Medium => 4,
            Difficulty::Hard => 6,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub language: Lang,
    pub difficulty: Difficulty,
    pub ai_side: SColor,
    pub time_budget_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Lang::En,
            difficulty: Difficulty::Medium,
            ai_side: SColor::White, // Black (human) plays first
            time_budget_ms: 3000,
        }
    }
}

pub fn config_path() -> PathBuf {
    PathBuf::from("/home/root/.config/gomoku-rm/config.yml")
}

fn try_load_from_disk() -> Option<Config> {
    let bytes = fs::read(config_path()).ok()?;
    serde_yaml::from_slice(&bytes).ok()
}

fn save_to_disk(c: &Config) -> anyhow::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_yaml::to_string(c)?)?;
    Ok(())
}

static GLOBAL: Mutex<Option<Config>> = Mutex::new(None);

/// Initialize from disk (or default), then apply language to the i18n module.
pub fn init() {
    let cfg = try_load_from_disk().unwrap_or_default();
    i18n::set_lang(cfg.language);
    *GLOBAL.lock().unwrap() = Some(cfg);
}

pub fn get() -> Config {
    GLOBAL
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default()
}

/// Mutate config, persist to disk, sync language to i18n.
pub fn update<F: FnOnce(&mut Config)>(f: F) {
    let mut g = GLOBAL.lock().unwrap();
    let cfg = g.get_or_insert_with(Config::default);
    f(cfg);
    i18n::set_lang(cfg.language);
    if let Err(e) = save_to_disk(cfg) {
        log::error!("config save failed: {e:?}");
    }
}

/// Convenience: derive a fresh GameMode from current config.
pub fn make_game_mode_pvai(cfg: &Config) -> crate::scene::game::GameMode {
    let ai: Color = cfg.ai_side.into();
    crate::scene::game::GameMode::PvAi {
        ai,
        depth: cfg.difficulty.depth(),
        time_budget_ms: cfg.time_budget_ms,
    }
}

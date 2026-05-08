//! Scene system + transition state machine.

pub mod game;
pub mod menu;
pub mod settings;

pub use game::GameScene;
pub use menu::MainMenuScene;
pub use settings::SettingsScene;

use crate::canvas::Canvas;
use libremarkable::input::InputEvent;

/// What a scene wants the main loop to do next.
pub enum Transition {
    /// Replace current scene with a new one.
    Replace(Box<dyn Scene>),
    /// Exit the application.
    Exit,
}

pub trait Scene {
    fn on_input(&mut self, event: InputEvent);
    fn draw(&mut self, canvas: &mut Canvas);
    /// Polled by the main loop after each frame. If Some, scene is swapped.
    fn take_transition(&mut self) -> Option<Transition> {
        None
    }
}

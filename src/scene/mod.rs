//! Scene system: each scene owns input + draw responsibility for one screen.

pub mod game;

pub use game::GameScene;

use crate::canvas::Canvas;
use libremarkable::input::InputEvent;

pub trait Scene {
    fn on_input(&mut self, event: InputEvent);
    fn draw(&mut self, canvas: &mut Canvas);
    /// Returns true when this scene wishes to terminate the app.
    fn done(&self) -> bool {
        false
    }
}

mod board;
mod canvas;
mod engine;
mod scene;

use crate::canvas::Canvas;
use crate::scene::{GameScene, Scene};
use libremarkable::input::{ev::EvDevContext, InputDevice, InputEvent};
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    log::info!("gomoku-rm starting (M2: PvP touch placement)");

    let mut canvas = Canvas::new();
    let mut current: Box<dyn Scene> = Box::new(GameScene::new());

    let (tx, rx) = channel::<InputEvent>();
    EvDevContext::new(InputDevice::Multitouch, tx).start();

    const FPS: u64 = 30;
    const FRAME: Duration = Duration::from_millis(1000 / FPS);

    loop {
        let frame_start = Instant::now();
        for event in rx.try_iter() {
            current.on_input(event);
        }
        current.draw(&mut canvas);
        if current.done() {
            break;
        }
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME {
            sleep(FRAME - elapsed);
        }
    }

    log::info!("gomoku-rm exiting");
}

use eframe::{egui::Vec2, epi};
use lib::Gui;

fn main() {
    env_logger::init();
    let app = Box::new(Gui::new());
    eframe::run_native(
        app,
        epi::NativeOptions {
            initial_window_size: Some(Vec2 {
                x: 800f32,
                y: 600f32,
            }),
            ..Default::default()
        },
    );
}

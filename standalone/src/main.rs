use eframe::{egui::Vec2};
use lib::Gui;

fn main() {
    env_logger::init();
    eframe::run_native(
        "Brimstone",
        eframe::NativeOptions {
            initial_window_size: Some(Vec2 {
                x: 800f32,
                y: 600f32,
            }),
            ..Default::default()
        },
        Box::new(|_ctx| Box::new(Gui::new()))
    );
}

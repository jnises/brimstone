use lib::Gui;

fn main() -> eframe::Result<()> {
    env_logger::init();
    eframe::run_native(
        "Brimstone",
        eframe::NativeOptions::default(),
        Box::new(|_ctx| Box::<Gui>::default()),
    )
}

use palette::Srgb;

pub(crate) trait Designer {
    fn show_ui(&mut self, ui: &mut eframe::egui::Ui) -> bool;
    fn render(&self, size: (usize, usize), buf: &mut [Srgb]);
}

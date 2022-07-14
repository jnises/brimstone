mod designer;
mod gamut_mapping;
mod hue_gradient;
mod linear_gradient;
mod utils;
use crate::designer::Designer;
use eframe::{egui, App};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use palette::Srgb;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const IMG_SIZE: usize = 512;

#[derive(EnumIter, Debug, PartialEq, Default, Copy, Clone)]
enum DesignerType {
    Linear,
    #[default]
    Hue,
}

impl DesignerType {
    fn make(&self) -> Box<dyn Designer> {
        match self {
            DesignerType::Linear => Box::new(linear_gradient::Gradient::new()),
            DesignerType::Hue => Box::new(hue_gradient::Gradient::new()),
        }
    }
}

fn make_texture_from_params(
    ctx: &eframe::egui::Context,
    designer: &dyn Designer,
) -> egui::TextureHandle {
    // TODO don't create intermediate buffer somehow?
    let mut buf = vec![Srgb::default(); IMG_SIZE * IMG_SIZE];
    designer.render((IMG_SIZE, IMG_SIZE), &mut buf);
    let u8buf: Vec<u8> = buf
        .iter()
        .flat_map(|p| {
            let q = p.into_format();
            [q.red, q.green, q.blue, u8::MAX]
        })
        .collect();
    ctx.load_texture(
        "gradient",
        egui::ColorImage::from_rgba_unmultiplied([IMG_SIZE, IMG_SIZE], u8buf.as_ref()),
    )
}

fn save_image_from_params<P: AsRef<std::path::Path>>(designer: &dyn Designer, path: P) {
    let mut buf = vec![Srgb::default(); IMG_SIZE * IMG_SIZE];
    designer.render((IMG_SIZE, IMG_SIZE), &mut buf);
    let u16buf: Vec<u16> = buf
        .iter()
        .flat_map(|p| {
            let q = p.into_format();
            [q.red, q.green, q.blue, u16::MAX]
        })
        .collect();
    if let Err(e) = image::ImageBuffer::<image::Rgba<u16>, Vec<u16>>::from_vec(
        IMG_SIZE as u32,
        IMG_SIZE as u32,
        u16buf,
    )
    .unwrap()
    .save(path)
    {
        MessageDialog::new()
            .set_type(MessageType::Error)
            .set_title("Error saving image")
            .set_text(&e.to_string())
            .show_alert()
            .unwrap();
    }
}

pub struct Gui {
    // TODO keep hold of old designers to not loose params when switching
    current_designer: (DesignerType, Box<dyn Designer>),
    texture: Option<egui::TextureHandle>,
}

impl Default for Gui {
    fn default() -> Self {
        let dtype = DesignerType::default();
        let designer = dtype.make();
        Self {
            current_designer: (dtype, designer),
            texture: None,
        }
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_source("designer")
                        .selected_text(format!("{:?}", self.current_designer.0))
                        .show_ui(ui, |ui| {
                            let mut selected_designer = self.current_designer.0;
                            for i in DesignerType::iter() {
                                ui.selectable_value(&mut selected_designer, i, format!("{:?}", i));
                            }
                            if selected_designer != self.current_designer.0 {
                                let new_designer = selected_designer.make();
                                self.current_designer = (selected_designer, new_designer);
                                self.texture = None;
                            }
                        });
                    ui.separator();
                    if ui.add(egui::Button::new("💾")).clicked() {
                        if let Ok(Some(path)) = FileDialog::new()
                            .add_filter("PNG Image", &["png"])
                            .show_save_single_file()
                        {
                            save_image_from_params(self.current_designer.1.as_ref(), path);
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.set_min_width(250.);
                    if self.current_designer.1.show_ui(ui) || self.texture.is_none() {
                        let tex = make_texture_from_params(ctx, self.current_designer.1.as_ref());
                        self.texture = Some(tex);
                    }
                    let texture = self.texture.as_ref().unwrap();
                    ui.image(texture, texture.size_vec2());
                });
            });
        });
    }
}

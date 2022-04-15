use eframe::{
    egui::{self, Vec2},
    epaint::{Color32},
    epi,
};

#[derive(Clone, PartialEq)]
struct Params {
    gray: u8,
}

impl Default for Params {
    fn default() -> Self {
        Self { gray: 128 }
    }
}

fn make_texture_from_params(ctx: &eframe::egui::Context, params: &Params) -> egui::TextureHandle {
    ctx.load_texture(
        "gradient",
        egui::ColorImage::new([512, 512], Color32::from_gray(params.gray)),
    )
}

struct Gui {
    texture: Option<(Params, egui::TextureHandle)>,
}

impl Gui {
    fn new() -> Self {
        Self { texture: None }
    }
}

impl epi::App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &epi::Frame) {
        let mut newparams = if let Some((params, _)) = &self.texture {
            params.clone()
        } else {
            Params::default()
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(egui::Slider::new(&mut newparams.gray, 0..=255).text("Gray"));
            match &self.texture {
                Some((params, _)) if params == &newparams => {}
                _ => {
                    let tex = make_texture_from_params(ctx, &newparams);
                    self.texture = Some((newparams, tex));
                }
            }
            let texture = &self.texture.as_ref().unwrap().1;
            ui.image(texture, texture.size_vec2());
        });
    }

    fn name(&self) -> &str {
        "Brimstone"
    }
}

fn main() {
    env_logger::init();
    let app = Box::new(Gui::new());
    eframe::run_native(
        app,
        epi::NativeOptions {
            initial_window_size: Some(Vec2 {
                x: 600f32,
                y: 800f32,
            }),
            ..Default::default()
        },
    );
}

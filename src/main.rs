use eframe::{
    egui::{self, Vec2},
    epaint::Color32,
    epi,
};
use palette::{FromColor, IntoColor, Oklab, Srgb, Clamp};

const IMG_SIZE: usize = 512;

#[derive(Clone, PartialEq)]
struct Params {
    l: f32,
    a: f32,
    b: f32,
}

impl Default for Params {
    fn default() -> Self {
        let l = (Oklab::<f32>::max_l() + Oklab::<f32>::min_l()) / 2.;
        let a = (Oklab::<f32>::max_a() + Oklab::<f32>::min_a()) / 2.;
        let b = (Oklab::<f32>::max_b() + Oklab::<f32>::min_b()) / 2.;
        Self { l, a, b }
    }
}

fn make_texture_from_params(ctx: &eframe::egui::Context, params: &Params) -> egui::TextureHandle {
    // create the buffer in a more efficient way
    let mut buf = vec![0u8; IMG_SIZE * IMG_SIZE * 4];
    for y in 0..IMG_SIZE {
        let normy = y as f32 / IMG_SIZE as f32;
        for x in 0..IMG_SIZE {
            let normx = x as f32 / IMG_SIZE as f32;
            let lab = palette::Oklab::new(params.l, normx * 2. - 1., normy * 2. - 1.).clamp();
            let rgb = Srgb::from_color(lab).into_format();
            let base = 4 * (x + (IMG_SIZE * y));
            buf[base] = rgb.red;
            buf[base + 1] = rgb.green;
            buf[base + 2] = rgb.blue;
            buf[base + 3] = 0xff;
        }
    }
    ctx.load_texture(
        "gradient",
        egui::ColorImage::from_rgba_unmultiplied([IMG_SIZE, IMG_SIZE], buf.as_ref()),
        // egui::ColorImage::new(
        //     [IMG_SIZE, IMG_SIZE],
        //     Color32::from_rgb(0, 0, 0),//color.red, color.green, color.blue),
        // ),
    )
    // let lab = palette::Oklab::new(params.l, params.a, params.b).clamp();
    // let color = Srgb::from_color(lab).clamp().into_format();
    // ctx.load_texture(
    //     "gradient",
    //     egui::ColorImage::new(
    //         [IMG_SIZE, IMG_SIZE],
    //         Color32::from_rgb(color.red, color.green, color.blue),
    //     ),
    // )
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
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &epi::Frame) {
        let mut newparams = if let Some((params, _)) = &self.texture {
            params.clone()
        } else {
            Params::default()
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add(
                        egui::Slider::new(&mut newparams.l, Oklab::min_l()..=Oklab::max_l())
                            .text("L"),
                    );
                    ui.add(
                        egui::Slider::new(&mut newparams.a, Oklab::min_a()..=Oklab::max_a())
                            .text("a"),
                    );
                    ui.add(
                        egui::Slider::new(&mut newparams.b, Oklab::min_b()..=Oklab::max_b())
                            .text("b"),
                    );
                });
                ui.vertical(|ui| {
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
            });
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
                x: 800f32,
                y: 600f32,
            }),
            ..Default::default()
        },
    );
}

mod gamut_mapping;
use eframe::{
    egui::{self, Vec2},
    epaint::Color32,
    epi,
};
use palette::{convert::FromColorUnclamped, Clamp, FromColor, IntoColor, Oklab, Srgb};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

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

fn oklab_to_srgb(lab: &palette::Oklab, map: bool) -> Srgb<f32> {
    if map {
        let linear = gamut_mapping::oklab_to_linear_srgb(gamut_mapping::OKLab {
            l: lab.l,
            a: lab.a,
            b: lab.b,
        });
        let mapped = gamut_mapping::gamut_clip_adaptive_l0_0_5(linear);
        //let mapped = gamut_mapping::gamut_clip_adaptive_L0_L_cusp(linear);
        //let mapped = gamut_mapping::gamut_clip_preserve_chroma(linear);
        //let mapped = gamut_mapping::gamut_clip_project_to_0_5(linear);
        //let mapped = gamut_mapping::gamut_clip_project_to_L_cusp(linear);
        Srgb::from_linear(palette::LinSrgb::new(mapped.r, mapped.g, mapped.b))
    } else {
        let rgb_unclamped = Srgb::from_color_unclamped(*lab);
        if rgb_unclamped.is_within_bounds() {
            rgb_unclamped.clamp()
        } else {
            Srgb::new(0f32, 0f32, 0f32)
        }
    }
}

fn make_texture_from_params(ctx: &eframe::egui::Context, params: &Params) -> egui::TextureHandle {
    let mut buf = vec![0u8; IMG_SIZE * IMG_SIZE * 4];
    buf.par_chunks_exact_mut(IMG_SIZE * 4)
        .enumerate()
        .for_each(|(y, row)| {
            let normy = y as f32 / IMG_SIZE as f32;
            row.chunks_exact_mut(4).enumerate().for_each(|(x, pixel)| {
                assert!(pixel.len() == 4);
                let normx = x as f32 / IMG_SIZE as f32;
                let lab = palette::Oklab::new(params.l, normx * 2. - 1., normy * 2. - 1.);
                let rgb = oklab_to_srgb(&lab, true).into_format();
                // let rgb_unclamped = Srgb::from_color_unclamped(lab);
                // let rgb = if rgb_unclamped.is_within_bounds() {
                //     rgb_unclamped.clamp().into_format()
                // } else {
                //     Srgb::new(0u8, 0u8, 0u8)
                // };
                pixel[0] = rgb.red;
                pixel[1] = rgb.green;
                pixel[2] = rgb.blue;
                pixel[3] = 0xff;
            });
        });
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

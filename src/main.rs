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
    l0: f32,
    ld: f32,
    a0: f32,
    ad: f32,
    b0: f32,
    bd: f32,
}

impl Default for Params {
    fn default() -> Self {
        let l0 = (Oklab::<f32>::max_l() + Oklab::<f32>::min_l()) / 2.;
        let a0 = (Oklab::<f32>::max_a() + Oklab::<f32>::min_a()) / 2.;
        let b0 = (Oklab::<f32>::max_b() + Oklab::<f32>::min_b()) / 2.;
        Self { l0, a0, b0, ld: 0., ad: 0., bd: 0. }
    }
}

fn oklab_to_srgb_clipped(lab: &palette::Oklab) -> Srgb<f32> {
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
}

fn oklab_to_srgb(lab: &palette::Oklab) -> Srgb<f32> {
    let rgb_unclamped = Srgb::from_color_unclamped(*lab);
    if rgb_unclamped.is_within_bounds() {
        rgb_unclamped.clamp()
    } else {
        Srgb::new(0f32, 0f32, 0f32)
    }
}

fn make_buf<F>(func: F) -> Vec<u8>
where
    F: Fn(f32, f32) -> [u8; 4] + Sync,
{
    let mut buf = vec![0u8; IMG_SIZE * IMG_SIZE * 4];
    buf.par_chunks_exact_mut(IMG_SIZE * 4)
        .enumerate()
        .for_each(|(y, row)| {
            let normy = y as f32 / IMG_SIZE as f32;
            row.chunks_exact_mut(4).enumerate().for_each(|(x, pixel)| {
                debug_assert!(pixel.len() == 4);
                let normx = x as f32 / IMG_SIZE as f32;
                let p = func(normx, normy);
                pixel.clone_from_slice(&p);
            });
        });
    buf
}

fn make_lightness_map(l: f32) -> impl Fn(f32, f32) -> [u8; 4] {
    move |normx, normy| {
        let lab = palette::Oklab::new(l, normx * 2. - 1., normy * 2. - 1.);
        let rgb = oklab_to_srgb_clipped(&lab).into_format();
        [rgb.red, rgb.green, rgb.blue, 0xff]
    }
}

fn make_linear_gradient(l0: f32, ld: f32, a0: f32, ad: f32, b0: f32, bd: f32) -> impl Fn(f32, f32) -> [u8; 4] {
    move |x, _| {
        let l = l0 + x * ld;
        let a = a0 + x * ad;
        let b = b0 + x * bd;
        let lab = palette::Oklab::new(l, a, b);
        let rgb = oklab_to_srgb_clipped(&lab).into_format();
        [rgb.red, rgb.green, rgb.blue, 0xff]
    }
}

fn make_texture_from_params(ctx: &eframe::egui::Context, params: &Params) -> egui::TextureHandle {
    let buf = make_buf(make_linear_gradient(params.l0, params.ld, params.a0, params.ad, params.b0, params.bd));
    ctx.load_texture(
        "gradient",
        egui::ColorImage::from_rgba_unmultiplied([IMG_SIZE, IMG_SIZE], buf.as_ref()),
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
                        egui::Slider::new(&mut newparams.l0, Oklab::min_l()..=Oklab::max_l())
                            .text("L0"),
                    );
                    ui.add(
                        egui::Slider::new(&mut newparams.ld, -1f32..=1.)
                            .text("Ld"),
                    );
                    ui.add(
                        egui::Slider::new(&mut newparams.a0, Oklab::min_a()..=Oklab::max_a())
                            .text("a0"),
                    );
                    ui.add(
                        egui::Slider::new(&mut newparams.ad, -1f32..=1.)
                            .text("ad"),
                    );
                    ui.add(
                        egui::Slider::new(&mut newparams.b0, Oklab::min_b()..=Oklab::max_b())
                            .text("b0"),
                    );
                    ui.add(
                        egui::Slider::new(&mut newparams.bd, -1f32..=1.)
                            .text("bd"),
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

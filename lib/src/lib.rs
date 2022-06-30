mod gamut_mapping;
use eframe::{
    egui::{self, Ui},
    App,
};
use glam::{vec3, Vec3};
use native_dialog::{FileDialog, MessageDialog, MessageType};
use palette::{convert::FromColorUnclamped, Clamp, Component, FromComponent, Oklab, Srgb};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

const IMG_SIZE: usize = 512;

fn vec3_to_oklab(vec: Vec3) -> Oklab {
    Oklab::new(vec.x, vec.y, vec.z)
}

fn oklab_to_vec3(lab: Oklab) -> Vec3 {
    vec3(lab.l, lab.a, lab.b)
}

const NEUTRAL_LAB: Oklab = Oklab {
    l: 0.5,
    a: 0.,
    b: 0.,
    // TODO use these once they are const
    // l: (Oklab::<f32>::max_l() + Oklab::<f32>::min_l()) / 2.,
    // a: (Oklab::<f32>::max_a() + Oklab::<f32>::min_a()) / 2.,
    // b: (Oklab::<f32>::max_b() + Oklab::<f32>::min_b()) / 2.,
};

#[derive(Clone, PartialEq)]
struct Params {
    center: Oklab,
    // TODO change the ab parts to rotation,scale?
    x_slope: Oklab,
    y_slope: Oklab,
    extend: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            center: NEUTRAL_LAB,
            x_slope: Oklab::default(),
            y_slope: Oklab::default(),
            extend: true,
        }
    }
}

fn oklab_to_srgb_clipped(lab: &palette::Oklab) -> Srgb<f32> {
    let linear = gamut_mapping::oklab_to_linear_srgb(gamut_mapping::OKLab {
        l: lab.l,
        a: lab.a,
        b: lab.b,
    });
    // TODO make these selectable in gui
    //let mapped = gamut_mapping::gamut_clip_adaptive_l0_0_5_alpha(linear, 0.);
    let mapped = gamut_mapping::gamut_clip_adaptive_l0_0_5(linear);
    //let mapped = gamut_mapping::gamut_clip_adaptive_L0_L_cusp(linear);
    //let mapped = gamut_mapping::gamut_clip_preserve_chroma(linear);
    //let mapped = gamut_mapping::gamut_clip_project_to_0_5(linear);
    //let mapped = gamut_mapping::gamut_clip_project_to_0_5(linear);
    //let mapped = gamut_mapping::gamut_clip_project_to_l_cusp(linear);
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

fn make_buf<T, F>(func: F) -> Vec<T>
where
    F: Fn(f32, f32) -> Srgb + Sync,
    T: Default + Copy + Send + FromComponent<f32> + Component,
{
    let mut buf = vec![T::default(); IMG_SIZE * IMG_SIZE * 4];
    buf.par_chunks_exact_mut(IMG_SIZE * 4)
        .enumerate()
        .for_each(|(y, row)| {
            let normy = y as f32 / IMG_SIZE as f32;
            row.chunks_exact_mut(4).enumerate().for_each(|(x, pixel)| {
                debug_assert!(pixel.len() == 4);
                let normx = x as f32 / IMG_SIZE as f32;
                let p: Srgb<T> = func(normx, normy).into_format();
                pixel.clone_from_slice(&[p.red, p.green, p.blue, T::max_intensity()]);
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

fn make_linear_gradient(center: Oklab, x_slope: Oklab, clip: bool) -> impl Fn(f32, f32) -> Srgb {
    move |x, _| {
        let xcenter = x - 0.5;
        let lab = vec3_to_oklab(oklab_to_vec3(center) + xcenter * oklab_to_vec3(x_slope));
        if clip {
            oklab_to_srgb_clipped(&lab)
        } else {
            oklab_to_srgb(&lab)
        }
    }
}

fn make_2d_gradient(
    center: Oklab,
    x_slope: Oklab,
    y_slope: Oklab,
    clip: bool,
) -> impl Fn(f32, f32) -> Srgb {
    move |x, y| {
        let xcenter = x - 0.5;
        let ycenter = y - 0.5;
        let lab = vec3_to_oklab(
            oklab_to_vec3(center)
                + xcenter * oklab_to_vec3(x_slope)
                + ycenter * oklab_to_vec3(y_slope),
        );
        if clip {
            oklab_to_srgb_clipped(&lab)
        } else {
            oklab_to_srgb(&lab)
        }
    }
}

fn make_texture_from_params(ctx: &eframe::egui::Context, params: &Params) -> egui::TextureHandle {
    let buf = make_buf(make_2d_gradient(
        params.center,
        params.x_slope,
        params.y_slope,
        params.extend,
    ));
    ctx.load_texture(
        "gradient",
        egui::ColorImage::from_rgba_unmultiplied([IMG_SIZE, IMG_SIZE], buf.as_ref()),
    )
}

fn save_image_from_params<P: AsRef<std::path::Path>>(params: &Params, path: P) {
    let buf = make_buf(make_2d_gradient(
        params.center,
        params.x_slope,
        params.y_slope,
        params.extend,
    ));
    if let Err(e) = image::ImageBuffer::<image::Rgba<u16>, Vec<u16>>::from_vec(
        IMG_SIZE as u32,
        IMG_SIZE as u32,
        buf,
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
    texture: Option<(Params, egui::TextureHandle)>,
}

impl Gui {
    pub fn new() -> Self {
        Self { texture: None }
    }
}

fn lab_slope_gui(ui: &mut Ui, center: &mut Oklab, x_slope: &mut Oklab, y_slope: &mut Oklab) {
    ui.add(egui::Slider::new(&mut center.l, Oklab::min_l()..=Oklab::max_l()).text("L center"));
    ui.add(egui::Slider::new(&mut center.a, Oklab::min_a()..=Oklab::max_a()).text("a center"));
    ui.add(egui::Slider::new(&mut center.b, Oklab::min_b()..=Oklab::max_b()).text("b center"));
    ui.add(egui::Slider::new(&mut x_slope.l, -1f32..=1.).text("L x slope"));
    ui.add(egui::Slider::new(&mut x_slope.b, -1f32..=1.).text("b x slope"));
    ui.add(egui::Slider::new(&mut x_slope.a, -1f32..=1.).text("a x slope"));
    ui.add(egui::Slider::new(&mut y_slope.l, -1f32..=1.).text("L y slope"));
    ui.add(egui::Slider::new(&mut y_slope.b, -1f32..=1.).text("b y slope"));
    ui.add(egui::Slider::new(&mut y_slope.a, -1f32..=1.).text("a y slope"));
    if ui.add(egui::Button::new("reset")).clicked() {
        *center = NEUTRAL_LAB;
        *x_slope = Oklab::new(0., 0., 0.);
        *y_slope = Oklab::new(0., 0., 0.);
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut newparams = if let Some((params, _)) = &self.texture {
            params.clone()
        } else {
            Params::default()
        };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(250.);
                    lab_slope_gui(
                        ui,
                        &mut newparams.center,
                        &mut newparams.x_slope,
                        &mut newparams.y_slope,
                    );
                    ui.checkbox(&mut newparams.extend, "extend");
                });
                ui.vertical(|ui| {
                    match &self.texture {
                        Some((params, _)) if params == &newparams => {}
                        _ => {
                            let tex = make_texture_from_params(ctx, &newparams);
                            self.texture = Some((newparams.clone(), tex));
                        }
                    }
                    let texture = &self.texture.as_ref().unwrap().1;
                    ui.image(texture, texture.size_vec2());
                    if ui.add(egui::Button::new("save")).clicked() {
                        if let Ok(Some(path)) = FileDialog::new()
                            .add_filter("PNG Image", &["png"])
                            .show_save_single_file()
                        {
                            save_image_from_params(&newparams, path);
                        }
                    }
                });
            });
        });
    }
}

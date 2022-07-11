use crate::{designer, gamut_mapping, utils::render_par};
use eframe::egui;
use glam::{vec3, Vec3};
use palette::{convert::FromColorUnclamped, Clamp, Oklab, Srgb};

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

#[derive(PartialEq, Clone)]
pub struct Gradient {
    center: Oklab,
    x_slope: Oklab,
    y_slope: Oklab,
    extend: bool,
}

impl Gradient {
    pub fn new() -> Self {
        Self {
            center: NEUTRAL_LAB,
            x_slope: Oklab::default(),
            y_slope: Oklab::default(),
            extend: true,
        }
    }
}

impl designer::Designer for Gradient {
    fn show_ui(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        let mut c = self.clone();
        let Gradient {
            center,
            x_slope,
            y_slope,
            extend,
        } = &mut c;
        ui.vertical(|ui| {
            ui.add(
                egui::Slider::new(&mut center.l, Oklab::min_l()..=Oklab::max_l()).text("L center"),
            );
            ui.add(
                egui::Slider::new(&mut center.a, Oklab::min_a()..=Oklab::max_a()).text("a center"),
            );
            ui.add(
                egui::Slider::new(&mut center.b, Oklab::min_b()..=Oklab::max_b()).text("b center"),
            );
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
            ui.checkbox(extend, "extend");
        });
        if c != *self {
            *self = c;
            true
        } else {
            false
        }
    }

    fn render(&self, size: (usize, usize), buf: &mut [Srgb]) {
        render_par(size, buf, |x, y| {
            let xcenter = x - 0.5;
            let ycenter = y - 0.5;
            let lab = vec3_to_oklab(
                oklab_to_vec3(self.center)
                    + xcenter * oklab_to_vec3(self.x_slope)
                    + ycenter * oklab_to_vec3(self.y_slope),
            );
            if self.extend {
                oklab_to_srgb_clipped(&lab)
            } else {
                oklab_to_srgb(&lab)
            }
        });
    }
}

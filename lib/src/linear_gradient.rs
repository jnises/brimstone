use crate::{designer, utils::{render_par, NEUTRAL_LAB, vec3_to_oklab, oklab_to_vec3, oklab_to_srgb_clipped, oklab_to_srgb}};
use eframe::egui;
use palette::{Oklab, Srgb};

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

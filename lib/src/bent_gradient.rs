use crate::{
    blur, designer,
    utils::{
        oklab_to_srgb, oklab_to_srgb_clipped, oklab_to_vec3, render_par, resettable_slider,
        vec3_to_oklab, NEUTRAL_LAB,
    },
};
use eframe::egui;
use palette::{convert::FromColorUnclamped, Oklab, Srgb};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

#[derive(PartialEq, Clone)]
pub struct Gradient {
    center: Oklab,
    x_slope: Oklab,
    y_slope: Oklab,
    extend: bool,
    smooth: f32,
}

impl Gradient {
    const CENTER_DEFAULT: Oklab = NEUTRAL_LAB;
    const SMOOTH_DEFAULT: f32 = 0.;
    pub fn new() -> Self {
        Self {
            center: NEUTRAL_LAB,
            x_slope: Oklab::default(),
            y_slope: Oklab::default(),
            extend: true,
            smooth: Self::SMOOTH_DEFAULT,
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
            smooth,
        } = &mut c;
        egui::Grid::new("params").show(ui, |ui| {
            resettable_slider(
                ui,
                &mut center.l,
                "L center",
                Oklab::min_l()..=Oklab::max_l(),
                Self::CENTER_DEFAULT.l,
            );
            ui.end_row();
            resettable_slider(
                ui,
                &mut center.a,
                "a center",
                Oklab::<f32>::min_a() * 2.0..=Oklab::<f32>::max_b() * 2.,
                Self::CENTER_DEFAULT.a,
            );
            ui.end_row();
            resettable_slider(
                ui,
                &mut center.b,
                "b center",
                Oklab::<f32>::min_b() * 2.0..=Oklab::<f32>::max_b() * 2.,
                Self::CENTER_DEFAULT.b,
            );
            ui.end_row();
            // TODO change
            ui.add(egui::Slider::new(&mut x_slope.l, -1f32..=1.).text("L x slope"));
            ui.end_row();
            ui.add(egui::Slider::new(&mut x_slope.b, -1f32..=1.).text("b x slope"));
            ui.end_row();
            ui.add(egui::Slider::new(&mut x_slope.a, -1f32..=1.).text("a x slope"));
            ui.end_row();
            ui.add(egui::Slider::new(&mut y_slope.l, -1f32..=1.).text("L y slope"));
            ui.end_row();
            ui.add(egui::Slider::new(&mut y_slope.b, -1f32..=1.).text("b y slope"));
            ui.end_row();
            ui.add(egui::Slider::new(&mut y_slope.a, -1f32..=1.).text("a y slope"));
            ui.end_row();
            if ui.add(egui::Button::new("reset")).clicked() {
                *center = NEUTRAL_LAB;
                *x_slope = Oklab::new(0., 0., 0.);
                *y_slope = Oklab::new(0., 0., 0.);
            }
            ui.end_row();
            ui.checkbox(extend, "extend");
            ui.end_row();
            // TODO disable when extend is turned on
            ui.add_enabled_ui(*extend, |ui| {
                resettable_slider(ui, smooth, "smooth", 0. ..=100., Self::SMOOTH_DEFAULT)
            });
            ui.end_row();
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
                oklab_to_srgb_clipped(lab)
            } else {
                oklab_to_srgb(&lab)
            }
        });
        if self.smooth > 0. && self.extend {
            // TODO have rayon split the work into bigger chunks to reduce sync?
            let mut labbuf: Vec<_> = buf
                .par_iter()
                .map(|c| palette::Oklab::from_color_unclamped(c.into_linear()))
                .collect();
            blur::gaussian_blur(labbuf.as_mut(), size.0, size.1, self.smooth);
            labbuf
                .par_iter()
                .copied()
                .zip(buf.par_iter_mut())
                .for_each(|(a, b)| *b = oklab_to_srgb_clipped(a));
        }
    }
}

use std::ops::RangeInclusive;

use crate::{
    blur, designer,
    utils::{
        oklab_to_srgb, oklab_to_srgb_clipped, oklab_to_vec3, render_par, resettable_slider,
        resettable_slider_raw, vec3_to_oklab, NEUTRAL_LAB,
    },
};
use eframe::egui::{self, Widget};
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
    const X_SLOPE_DEFAULT: Oklab = Oklab {
        l: 0.,
        a: 0.,
        b: 0.,
    };
    const Y_SLOPE_DEFAULT: Oklab = Oklab {
        l: -1.,
        a: 0.,
        b: 0.,
    };
    pub fn new() -> Self {
        Self {
            center: NEUTRAL_LAB,
            x_slope: Self::X_SLOPE_DEFAULT,
            y_slope: Self::Y_SLOPE_DEFAULT,
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
        ui.vertical(|ui| {
            ui.add(LabUi::new(center, "center").default_value(Self::CENTER_DEFAULT));
            ui.add(
                LabUi::new(x_slope, "x")
                    .default_value(Self::X_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l()),
            );
            ui.add(
                LabUi::new(y_slope, "y")
                    .default_value(Self::Y_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l()),
            );
            ui.checkbox(extend, "extend");
            ui.add_enabled_ui(*extend, |ui| {
                resettable_slider(ui, smooth, "smooth", 0. ..=100., Self::SMOOTH_DEFAULT)
            });
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

// TODO make this show a color picker?
struct LabUi<'a> {
    lab: &'a mut Oklab,
    label: &'a str,
    l_range: RangeInclusive<f32>,
    a_range: RangeInclusive<f32>,
    b_range: RangeInclusive<f32>,
    default_value: Oklab,
}

impl<'a> LabUi<'a> {
    pub fn new(lab: &'a mut Oklab, label: &'a str) -> Self {
        Self {
            lab,
            label,
            l_range: Oklab::min_l()..=Oklab::max_l(),
            a_range: Oklab::<f32>::min_a()..=Oklab::<f32>::max_a(),
            b_range: Oklab::<f32>::min_b()..=Oklab::<f32>::max_b(),
            default_value: NEUTRAL_LAB,
        }
    }

    pub fn l_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.l_range = range;
        self
    }

    pub fn a_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.a_range = range;
        self
    }

    pub fn b_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.b_range = range;
        self
    }

    pub fn default_value(mut self, default_value: Oklab) -> Self {
        self.default_value = default_value;
        self
    }
}

impl<'a> Widget for LabUi<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new(self.label)
            .show(ui, |ui| {
                ui.label(self.label);
                ui.end_row();
                resettable_slider_raw(ui, &mut self.lab.l, "L", self.l_range, self.default_value.l);
                ui.end_row();
                resettable_slider_raw(ui, &mut self.lab.a, "a", self.a_range, self.default_value.a);
                ui.end_row();
                resettable_slider_raw(ui, &mut self.lab.b, "b", self.b_range, self.default_value.b);
                ui.end_row();
            })
            .response
    }
}

use crate::{
    blur, designer,
    lab_ui::LabUi,
    utils::{
        oklab_to_srgb, oklab_to_srgb_clipped, oklab_to_vec3, render_par, resettable_slider,
        vec3_to_oklab, NEUTRAL_LAB,
    },
};
use palette::{convert::FromColorUnclamped, Oklab, Srgb};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

#[derive(PartialEq, Clone)]
pub struct Gradient {
    center: Oklab,
    x_slope: Oklab,
    x2_slope: Oklab,
    x3_slope: Oklab,
    y_slope: Oklab,
    y2_slope: Oklab,
    y3_slope: Oklab,
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
    const X2_SLOPE_DEFAULT: Oklab = Self::X_SLOPE_DEFAULT;
    const X3_SLOPE_DEFAULT: Oklab = Self::X2_SLOPE_DEFAULT;
    const Y_SLOPE_DEFAULT: Oklab = Oklab {
        l: -1.,
        a: 0.,
        b: 0.,
    };
    const Y2_SLOPE_DEFAULT: Oklab = Oklab {
        l: 0.,
        a: 0.,
        b: 0.,
    };
    const Y3_SLOPE_DEFAULT: Oklab = Self::Y2_SLOPE_DEFAULT;
    pub fn new() -> Self {
        Self {
            center: NEUTRAL_LAB,
            x_slope: Self::X_SLOPE_DEFAULT,
            x2_slope: Self::X2_SLOPE_DEFAULT,
            x3_slope: Self::X3_SLOPE_DEFAULT,
            y_slope: Self::Y_SLOPE_DEFAULT,
            y2_slope: Self::Y2_SLOPE_DEFAULT,
            y3_slope: Self::Y3_SLOPE_DEFAULT,
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
            x2_slope,
            x3_slope,
            y_slope,
            y2_slope,
            y3_slope,
            extend,
            smooth,
        } = &mut c;
        ui.vertical(|ui| {
            ui.add(LabUi::new(center, "center").default_value(Self::CENTER_DEFAULT));
            let slope2_range = 2. * Oklab::<f32>::min_a()..=2. * Oklab::<f32>::max_a();
            let slope3_range = 2. * Oklab::<f32>::min_a()..=2. * Oklab::<f32>::max_a();
            ui.add(
                LabUi::new(x_slope, "x")
                    .default_value(Self::X_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l()),
            );
            ui.add(
                LabUi::new(x2_slope, "x2")
                    .default_value(Self::X2_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l())
                    .a_range(slope2_range.clone())
                    .b_range(slope2_range.clone()),
            );
            ui.add(
                LabUi::new(x3_slope, "x3")
                    .default_value(Self::X3_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l())
                    .a_range(slope3_range.clone())
                    .b_range(slope3_range.clone()),
            );
            ui.add(
                LabUi::new(y_slope, "y")
                    .default_value(Self::Y_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l()),
            );
            ui.add(
                LabUi::new(y2_slope, "y2")
                    .default_value(Self::Y2_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l())
                    .a_range(slope2_range.clone())
                    .b_range(slope2_range),
            );
            ui.add(
                LabUi::new(y3_slope, "y3")
                    .default_value(Self::Y3_SLOPE_DEFAULT)
                    .l_range(-Oklab::<f32>::max_l()..=Oklab::max_l())
                    .a_range(slope3_range.clone())
                    .b_range(slope3_range),
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
                    + xcenter.powi(2) * oklab_to_vec3(self.x2_slope)
                    + xcenter.powi(3) * oklab_to_vec3(self.x3_slope)
                    + ycenter * oklab_to_vec3(self.y_slope)
                    + ycenter.powi(2) * oklab_to_vec3(self.y2_slope)
                    + ycenter.powi(3) * oklab_to_vec3(self.y3_slope),
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

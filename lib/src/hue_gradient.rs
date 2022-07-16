use crate::{
    blur, designer,
    utils::{
        oklab_to_srgb, oklab_to_srgb_clipped, oklab_to_vec3, render_par, resettable_slider,
        vec3_to_oklab, NEUTRAL_LAB,
    },
};
use glam::{vec2, Vec2, Vec3};
use palette::{convert::FromColorUnclamped, Oklab, Srgb};
use rayon::{
    iter::{
        IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator,
        ParallelIterator,
    },
    slice::ParallelSliceMut,
};
use std::f32::consts::PI;

#[derive(PartialEq, Clone)]
pub struct Gradient {
    center: Oklab,
    rotation: f32,
    phase: f32,
    saturation: f32,
    saturation_non_midtone: f32,
    twist: f32,
    twist_v: f32,
    extend: bool,
    smooth: f32,
}

impl Gradient {
    const CENTER_DEFAULT: Oklab = NEUTRAL_LAB;
    const ROTATION_DEFAULT: f32 = 2. * PI;
    const SATURATION_DEFAULT: f32 = 0.5;
    const SATURATION_NON_MIDTONE_DEFAULT: f32 = 0.;
    const PHASE_DEFAULT: f32 = 0.;
    const TWIST_DEFAULT: f32 = 0.;
    const TWIST_V_DEFAULT: f32 = 0.;
    const SMOOTH_DEFAULT: f32 = 0.;
    pub fn new() -> Self {
        Self {
            center: Self::CENTER_DEFAULT,
            rotation: Self::ROTATION_DEFAULT,
            phase: Self::PHASE_DEFAULT,
            saturation: Self::SATURATION_DEFAULT,
            saturation_non_midtone: Self::SATURATION_NON_MIDTONE_DEFAULT,
            twist: Self::TWIST_DEFAULT,
            twist_v: Self::TWIST_V_DEFAULT,
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
            rotation,
            phase,
            saturation,
            saturation_non_midtone,
            twist,
            twist_v,
            extend,
            smooth,
        } = &mut c;
        ui.vertical(|ui| {
            resettable_slider(
                ui,
                &mut center.l,
                "L center",
                Oklab::min_l()..=Oklab::max_l(),
                Self::CENTER_DEFAULT.l,
            );
            resettable_slider(
                ui,
                &mut center.a,
                "a center",
                Oklab::<f32>::min_a() * 2.0..=Oklab::<f32>::max_b() * 2.,
                Self::CENTER_DEFAULT.a,
            );
            resettable_slider(
                ui,
                &mut center.b,
                "b center",
                Oklab::<f32>::min_b() * 2.0..=Oklab::<f32>::max_b() * 2.,
                Self::CENTER_DEFAULT.b,
            );
            resettable_slider(
                ui,
                rotation,
                "rotation",
                0. ..=PI * 2.,
                Self::ROTATION_DEFAULT,
            );
            resettable_slider(ui, phase, "phase", -PI..=PI, Self::PHASE_DEFAULT);
            resettable_slider(ui, twist, "twist", -PI * 5.0..=PI * 5., Self::TWIST_DEFAULT);
            resettable_slider(
                ui,
                twist_v,
                "twist v",
                -PI * 5.0..=PI * 5.,
                Self::TWIST_DEFAULT,
            );
            resettable_slider(
                ui,
                saturation,
                "saturation",
                0. ..=1.,
                Self::SATURATION_DEFAULT,
            );
            resettable_slider(
                ui,
                saturation_non_midtone,
                "saturation !midtone",
                0. ..=1.,
                Self::SATURATION_NON_MIDTONE_DEFAULT,
            );
            ui.checkbox(extend, "extend");
            resettable_slider(ui, smooth, "smooth", 0. ..=100., Self::SMOOTH_DEFAULT);
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
            let xcenter = 2. * (x - 0.5);
            let ycenter = 2. * (y - 0.5);
            let lightness = self.center.l - ycenter * 0.5;
            let twist = self.twist + y * self.twist_v;
            let rot =
                Vec2::from_angle(xcenter * 0.5 * self.rotation + self.phase + ycenter * twist);
            let midtone_mask = ((lightness - NEUTRAL_LAB.l).abs() * 2.).powi(2);
            let saturation = (self.saturation
                * (1. - (1. - self.saturation_non_midtone) * midtone_mask))
                .max(0.);
            let chroma = vec2(rot.x, rot.y) * saturation + vec2(self.center.a, self.center.b);
            let lab = Oklab::new(lightness, chroma.x, chroma.y);
            if self.extend {
                oklab_to_srgb_clipped(lab)
            } else {
                oklab_to_srgb(&lab)
            }
        });
        if self.smooth > 0. {
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

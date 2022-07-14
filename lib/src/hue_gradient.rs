use crate::{
    designer,
    utils::{
        oklab_to_srgb, oklab_to_srgb_clipped, oklab_to_vec3, render_par, resettable_slider,
        vec3_to_oklab, NEUTRAL_LAB,
    },
};
use glam::{vec2, vec3, Vec2};
use palette::{Oklab, Srgb};
use std::f32::consts::PI;

#[derive(PartialEq, Clone)]
pub struct Gradient {
    center: Oklab,
    rotation: f32,
    phase: f32,
    saturation: f32,
    saturation_non_midtone: f32,
    twist: f32,
    extend: bool,
}

impl Gradient {
    const CENTER_DEFAULT: Oklab = NEUTRAL_LAB;
    const ROTATION_DEFAULT: f32 = 2. * PI;
    const SATURATION_DEFAULT: f32 = 0.5;
    const SATURATION_NON_MIDTONE_DEFAULT: f32 = 0.;
    const PHASE_DEFAULT: f32 = 0.;
    const TWIST_DEFAULT: f32 = 0.;
    pub fn new() -> Self {
        Self {
            center: Self::CENTER_DEFAULT,
            rotation: Self::ROTATION_DEFAULT,
            phase: Self::PHASE_DEFAULT,
            saturation: Self::SATURATION_DEFAULT,
            saturation_non_midtone: Self::SATURATION_NON_MIDTONE_DEFAULT,
            twist: Self::TWIST_DEFAULT,
            extend: true,
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
            extend,
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
                Oklab::min_a()..=Oklab::max_b(),
                Self::CENTER_DEFAULT.a,
            );
            resettable_slider(
                ui,
                &mut center.b,
                "b center",
                Oklab::min_b()..=Oklab::max_b(),
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
            resettable_slider(ui, twist, "twist", -PI * 5. ..= PI * 5., Self::PHASE_DEFAULT);
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
            let rot = Vec2::from_angle(xcenter * 0.5 * self.rotation + self.phase + ycenter * self.twist);
            let chroma = vec2(rot.x, rot.y) + vec2(self.center.a, self.center.b);
            let midtone_mask = ((lightness - NEUTRAL_LAB.l).abs() * 2.).powi(2);
            let saturation = (self.saturation
                * (1. - (1. - self.saturation_non_midtone) * midtone_mask))
                .max(0.);
            let chroma2 = chroma * saturation;
            let lab = Oklab::new(lightness, chroma2.x, chroma2.y);
            if self.extend {
                oklab_to_srgb_clipped(&lab)
            } else {
                oklab_to_srgb(&lab)
            }
        });
    }
}

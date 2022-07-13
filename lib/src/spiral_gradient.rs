use crate::{
    designer,
    utils::{
        oklab_to_srgb, oklab_to_srgb_clipped, oklab_to_vec3, render_par, resettable_slider,
        vec3_to_oklab, NEUTRAL_LAB,
    },
};
use eframe::egui;
use glam::{vec3, Vec2};
use palette::{float::Float, Oklab, Srgb};

#[derive(PartialEq, Clone)]
pub struct Gradient {
    center: Oklab,
    rotation: f32,
    saturation: f32,
    saturation_midtone: f32,
    extend: bool,
}

impl Gradient {
    const CENTER_DEFAULT: Oklab = NEUTRAL_LAB;
    const ROTATION_DEFAULT: f32 = 1.;
    const SATURATION_DEFAULT: f32 = 0.5;
    const SATURATION_MIDTONE_DEFAULT: f32 = 0.;
    pub fn new() -> Self {
        Self {
            center: Self::CENTER_DEFAULT,
            rotation: Self::ROTATION_DEFAULT,
            saturation: Self::SATURATION_DEFAULT,
            saturation_midtone: Self::SATURATION_MIDTONE_DEFAULT,
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
            saturation,
            saturation_midtone,
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
            resettable_slider(ui, rotation, "rotation", 0. ..=1., Self::ROTATION_DEFAULT);
            resettable_slider(
                ui,
                saturation,
                "saturation",
                0. ..=1.,
                Self::SATURATION_DEFAULT,
            );
            resettable_slider(
                ui,
                saturation_midtone,
                "saturation midtone",
                0. ..=1.,
                Self::SATURATION_MIDTONE_DEFAULT,
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
            let xcenter = x - 0.5;
            let ycenter = y - 0.5;
            let rot = Vec2::from_angle(xcenter * self.rotation * std::f32::consts::PI * 2.);
            let chroma = vec3(0., rot.x, rot.y);
            let midtone = (ycenter.abs() * 2.).sqrt();
            let saturation = (self.saturation * (1. - self.saturation_midtone * midtone)).max(0.);
            let lab = vec3_to_oklab(
                oklab_to_vec3(self.center)
                    + saturation * chroma
                    + ycenter * vec3(-1., 0., 0.),
            );
            if self.extend {
                oklab_to_srgb_clipped(&lab)
            } else {
                oklab_to_srgb(&lab)
            }
        });
    }
}

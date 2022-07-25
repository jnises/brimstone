use std::f32::consts::PI;

use crate::{
    blur, designer,
    lab_ui::LabUi,
    utils::{
        oklab_to_srgb, oklab_to_srgb_clipped, oklab_to_vec3, render_par, resettable_slider,
        vec3_to_oklab,
    },
};
use glam::{vec3, Mat2};
use num_bigint::BigUint;
use palette::{convert::FromColorUnclamped, Oklab, Srgb};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

#[derive(PartialEq, Clone)]
pub struct Gradient {
    offset: Oklab,
    scale: Oklab,
    rotation: f32,
    levels: u32,
    extend: bool,
    smooth: f32,
}

impl Gradient {
    const OFFSET_DEFAULT: Oklab = Oklab {
        l: 0.,
        a: 0.,
        b: 0.,
    };
    const SCALE_DEFAULT: Oklab = Oklab {
        l: 1.,
        a: 1.,
        b: 1.,
    };
    const SMOOTH_DEFAULT: f32 = 0.;
    const LEVELS_DEFAULT: u32 = 3;
    const ROTATION_DEFAULT: f32 = 0.;
    pub fn new() -> Self {
        Self {
            offset: Self::OFFSET_DEFAULT,
            scale: Self::SCALE_DEFAULT,
            levels: Self::LEVELS_DEFAULT,
            rotation: Self::ROTATION_DEFAULT,
            extend: true,
            smooth: Self::SMOOTH_DEFAULT,
        }
    }
}

impl designer::Designer for Gradient {
    fn show_ui(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        let mut c = self.clone();
        let Gradient {
            offset,
            scale,
            smooth,
            rotation,
            extend,
            levels,
        } = &mut c;
        ui.vertical(|ui| {
            ui.add(
                LabUi::new(offset, "offset")
                    .default_value(Self::OFFSET_DEFAULT)
                    .l_range(-1.0..=1.0),
            );
            let scale_range = 0.01..=2.0;
            ui.add(
                LabUi::new(scale, "scale")
                    .default_value(Self::SCALE_DEFAULT)
                    .l_range(scale_range.clone())
                    .a_range(scale_range.clone())
                    .b_range(scale_range),
            );
            resettable_slider(
                ui,
                rotation,
                "rotation",
                0.0..=PI * 2.0,
                Self::ROTATION_DEFAULT,
            );
            resettable_slider(ui, levels, "levels", 1..=9, Self::LEVELS_DEFAULT);
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
        assert!(size.0 == size.1);
        assert!(size.0.is_power_of_two());
        let h_bits = self.levels + 1;
        let h_size = 2_u32.pow(h_bits);
        let maxhid = h_size.pow(2) - 1;
        render_par(size, buf, |x, y| {
            let hid_f = maxhid as f32 * x;
            let hid_lower = hid_f as u64;
            debug_assert!(hid_lower <= maxhid as u64);
            let f = hid_f.fract();
            let p2_lower = hilbert::Point::new_from_hilbert_index(
                0,
                &BigUint::from(hid_lower),
                h_bits as usize,
                2,
            );
            let v2_lower = glam::Vec2::from_slice(
                p2_lower
                    .get_coordinates()
                    .iter()
                    .map(|&a| (a as f32 / h_size as f32 - 0.5) * 2.0)
                    .collect::<Vec<_>>()
                    .as_ref(),
            );
            let p2_upper = hilbert::Point::new_from_hilbert_index(
                0,
                &BigUint::from(hid_lower + 1),
                h_bits as usize,
                2,
            );
            let v2_upper = glam::Vec2::from_slice(
                p2_upper
                    .get_coordinates()
                    .iter()
                    .map(|&a| (a as f32 / h_size as f32 - 0.5) * 2.0)
                    .collect::<Vec<_>>()
                    .as_ref(),
            );
            let mut v2 = v2_lower.lerp(v2_upper, f as f32);
            v2 = Mat2::from_angle(self.rotation) * v2;
            let mut v3 = vec3(1. - y, v2.x, v2.y);
            v3 *= oklab_to_vec3(self.scale);
            v3 += oklab_to_vec3(self.offset);

            let lab = vec3_to_oklab(v3);
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

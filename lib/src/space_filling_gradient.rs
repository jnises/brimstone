use crate::{
    blur, designer,
    lab_ui::LabUi,
    rotator::Rotator,
    utils::{
        oklab_to_srgb_clipped, oklab_to_vec3, render_par_usize, resettable_slider, vec3_to_oklab,
    },
};
use eframe::egui;
use glam::{vec3, Quat, Vec3};
use num_bigint::BigUint;
use palette::{convert::FromColorUnclamped, Oklab, Srgb};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

#[derive(PartialEq, Clone)]
pub struct Gradient {
    offset: Oklab,
    scale: Oklab,
    // TODO add rotation
    rotation: Quat,
    levels: u32,
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
    const ROTATION_DEFAULT: Quat = Quat::IDENTITY;
    pub fn new() -> Self {
        Self {
            offset: Self::OFFSET_DEFAULT,
            scale: Self::SCALE_DEFAULT,
            rotation: Self::ROTATION_DEFAULT,
            levels: Self::LEVELS_DEFAULT,
            smooth: Self::SMOOTH_DEFAULT,
        }
    }
}

impl designer::Designer for Gradient {
    fn show_ui(&mut self, ui: &mut eframe::egui::Ui) -> bool {
        const SPACE: f32 = 10.;
        let mut c = self.clone();
        let Gradient {
            offset,
            scale,
            rotation,
            smooth,
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
            ui.add_space(SPACE);
            ui.label("rotation");
            ui.horizontal(|ui| {
                ui.add(Rotator::new(rotation));
                if ui
                    .add_enabled(*rotation != Self::ROTATION_DEFAULT, egui::Button::new("⟲"))
                    .clicked()
                {
                    *rotation = Self::ROTATION_DEFAULT;
                }
            });
            // TODO rotation
            resettable_slider(ui, levels, "levels", 1..=9, Self::LEVELS_DEFAULT);
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
        assert!(size.0 == size.1);
        assert!(size.0.is_power_of_two());
        let bits_2d = usize::BITS - size.0.leading_zeros() - 1;
        let maxid_2d = size.0.pow(2) - 1;
        let bits_3d = self.levels + 1;
        let size_3d = 2_u32.pow(bits_3d);
        let maxid_3d = size_3d.pow(3) - 1;
        render_par_usize(size, buf, |x, y| {
            let hid_2d =
                hilbert::Point::new(0, &[x as u32, y as u32]).hilbert_transform(bits_2d as usize);
            let t = u64::try_from(hid_2d).unwrap() as f64 / maxid_2d as f64;
            debug_assert!(t <= 1.);
            let hid_3d_f = maxid_3d as f64 * t;
            let hid_lower_3d = hid_3d_f as u64;
            debug_assert!(hid_lower_3d <= maxid_3d as u64);
            let f = hid_3d_f.fract();
            let p3_lower = hilbert::Point::new_from_hilbert_index(
                0,
                &BigUint::from(hid_lower_3d),
                bits_3d as usize,
                3,
            );
            let v3_lower = glam::Vec3::from_slice(
                p3_lower
                    .get_coordinates()
                    .iter()
                    .map(|&a| a as f32 / size_3d as f32)
                    .collect::<Vec<_>>()
                    .as_ref(),
            );
            let p3_upper = hilbert::Point::new_from_hilbert_index(
                0,
                &BigUint::from(hid_lower_3d + 1),
                bits_3d as usize,
                3,
            );
            let v3_upper = glam::Vec3::from_slice(
                p3_upper
                    .get_coordinates()
                    .iter()
                    .map(|&a| a as f32 / size_3d as f32)
                    .collect::<Vec<_>>()
                    .as_ref(),
            );
            let mut v3 = v3_lower.lerp(v3_upper, f as f32);
            v3 -= Vec3::splat(0.5);
            v3 = self.rotation * v3;
            v3 *= vec3(1., 2., 2.);
            //v3 += vec3(-0.5, -1., -1.);
            v3 *= oklab_to_vec3(self.scale);
            v3.x += 0.5;
            v3 += oklab_to_vec3(self.offset);

            let lab = vec3_to_oklab(v3);
            // TODO do we need clipped? in case the curve goes outside gamut?
            oklab_to_srgb_clipped(lab)
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

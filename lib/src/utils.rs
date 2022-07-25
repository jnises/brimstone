use std::ops::RangeInclusive;

use eframe::{egui::{self, Ui}, emath};
use glam::{vec3, Vec3};
use palette::{convert::FromColorUnclamped, Clamp, Component, FromComponent, Oklab, Srgb};
use rayon::{
    iter::{IndexedParallelIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::gamut_mapping::{self};

pub fn render_par<F, T>(size: (usize, usize), buf: &mut [Srgb<T>], func: F)
where
    F: Fn(f32, f32) -> Srgb + Sync,
    T: Default + Copy + Send + FromComponent<f32> + Component,
{
    render_par_usize(size, buf, |x, y| func(x as f32 / size.0 as f32, y as f32 / size.1 as f32));
}

pub fn render_par_usize<F, T>(size: (usize, usize), buf: &mut [Srgb<T>], func: F)
where
    F: Fn(usize, usize) -> Srgb + Sync,
    T: Default + Copy + Send + FromComponent<f32> + Component,
{
    assert!(buf.len() == size.0 * size.1);
    buf.par_chunks_exact_mut(size.0)
        .enumerate()
        .for_each(|(y, row)| {
            row.iter_mut().enumerate().for_each(|(x, pixel)| {
                let p: Srgb<T> = func(x, y).into_format();
                *pixel = p;
            });
        });
}

pub fn vec3_to_oklab(vec: Vec3) -> Oklab {
    Oklab::new(vec.x, vec.y, vec.z)
}

pub fn oklab_to_vec3(lab: Oklab) -> Vec3 {
    vec3(lab.l, lab.a, lab.b)
}

pub const NEUTRAL_LAB: Oklab = Oklab {
    l: 0.5,
    a: 0.,
    b: 0.,
    // TODO use these once they are const
    // l: (Oklab::<f32>::max_l() + Oklab::<f32>::min_l()) / 2.,
    // a: (Oklab::<f32>::max_a() + Oklab::<f32>::min_a()) / 2.,
    // b: (Oklab::<f32>::max_b() + Oklab::<f32>::min_b()) / 2.,
};

pub fn oklab_to_srgb_clipped(lab: palette::Oklab) -> Srgb<f32> {
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

pub fn oklab_to_srgb(lab: &palette::Oklab) -> Srgb<f32> {
    let rgb_unclamped = Srgb::from_color_unclamped(*lab);
    if rgb_unclamped.is_within_bounds() {
        rgb_unclamped.clamp()
    } else {
        Srgb::new(0f32, 0f32, 0f32)
    }
}

pub fn resettable_slider_raw<T: emath::Numeric>(
    ui: &mut Ui,
    value: &mut T,
    text: &str,
    range: RangeInclusive<T>,
    default_value: T,
) {
    debug_assert!(range.contains(&default_value));
    ui.add(egui::Slider::new(value, range).text(text));
    if ui
        .add_enabled(*value != default_value, egui::Button::new("⟲"))
        .clicked()
    {
        *value = default_value;
    }
}

pub fn resettable_slider<T: emath::Numeric>(
    ui: &mut Ui,
    value: &mut T,
    text: &str,
    range: RangeInclusive<T>,
    default_value: T,
) {
    debug_assert!(range.contains(&default_value));
    ui.horizontal(|ui| resettable_slider_raw(ui, value, text, range, default_value));
}

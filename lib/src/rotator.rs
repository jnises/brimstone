use std::cmp::Ordering;

use eframe::{
    egui::{Sense, Widget},
    emath,
    epaint::{Color32, Stroke},
};
use glam::{Quat, Vec3};
use palette::convert::FromColorUnclamped;

pub struct Rotator<'a> {
    quat: &'a mut Quat,
}

impl<'a> Rotator<'a> {
    pub fn new(quat: &'a mut Quat) -> Self {
        Self { quat }
    }
}

impl<'a> Widget for Rotator<'a> {
    fn ui(self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        // TODO when do we need persistent ids?
        //let id = ui.make_persistent_id("rotator");
        const SIZE: f32 = 100.0;
        let (response, painter) =
            ui.allocate_painter(emath::vec2(SIZE, SIZE), Sense::click_and_drag());
        let id = response.id;
        // TODO is this correct?
        let on = if let Some(pos) = response.interact_pointer_pos() {
            let data = *ui.data().get_temp_mut_or_insert_with(id, || State {
                start_rot: *self.quat,
                start_pos: pos,
            });
            let d = pos - data.start_pos;
            *self.quat = Quat::from_scaled_axis(glam::vec3(d.y, -d.x, 0.) * 0.01) * data.start_rot;
            true
        } else {
            ui.data().remove::<State>(id);
            false
        };
        if ui.is_rect_visible(response.rect) {
            let visuals = ui.style().interact_selectable(&response, on);
            // TODO use some constant?
            let rect = response.rect.shrink(2.);
            painter.circle(
                rect.center(),
                rect.width() / 2.,
                visuals.bg_fill,
                visuals.fg_stroke,
            );
            let mut axis = [(Vec3::X, 0.), (Vec3::NEG_Y, 120.), (Vec3::NEG_Z, 240.)]
                .into_iter()
                .map(|(v, h)| (v, h, *self.quat * v))
                .collect::<Vec<_>>();
            axis.sort_by(|(_, _, Vec3 { z: az, .. }), (_, _, Vec3 { z: bz, .. })| {
                bz.partial_cmp(az).unwrap_or(Ordering::Equal)
            });
            for (_, h, rv) in axis {
                let depth = 1. - ((rv.z + 1.) / 2.).clamp(0., 1.);
                let c = palette::Srgb::from_color_unclamped(palette::Oklch {
                    l: 0.4 + depth * 0.4,
                    chroma: 0.3 + depth * 0.2,
                    hue: h.into(),
                })
                .into_format();
                let p =
                    rect.center() + emath::vec2(rv.x, rv.y) / (1.7 - depth) * rect.width() / 2.1;
                painter.line_segment(
                    [rect.center(), p],
                    Stroke {
                        width: 1.,
                        color: Color32::from_rgb(c.red, c.green, c.blue),
                    },
                );
            }
        }
        response
    }
}

#[derive(Clone, Copy, Debug)]
struct State {
    start_rot: Quat,
    start_pos: emath::Pos2,
}

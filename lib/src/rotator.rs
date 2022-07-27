use eframe::{
    egui::{Context, Id, Sense, Widget},
    emath::{self, pos2, Pos2},
    epaint::Shape,
};
use glam::{vec3, Quat, Vec2, Vec3};
use once_cell::sync::Lazy;

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
        // TODO do we need a persistent id?
        let id = ui.make_persistent_id("rotator");
        const SIZE: f32 = 100.0;
        let (response, painter) =
            ui.allocate_painter(emath::vec2(SIZE, SIZE), Sense::click_and_drag());
        // TODO is this correct?
        let on = if let Some(pos) = response.interact_pointer_pos() {
            let data = ui
                .data()
                .get_temp_mut_or_insert_with(id, || State {
                    start_rot: *self.quat,
                    start_pos: pos,
                })
                .clone();
            let d = pos - data.start_pos;
            // TODO fix this
            *self.quat = Quat::from_scaled_axis(glam::vec3(d.y, -d.x, 0.) * 0.01) * data.start_rot;
            true
        } else {
            ui.data().remove::<State>(id);
            false
        };
        // TODO
        if ui.is_rect_visible(response.rect) {
            //let how_on = ui.ctx().animate_bool(response.id, on);
            let visuals = ui.style().interact_selectable(&response, on);
            // TODO use some constant?
            let rect = response.rect.shrink(2.);
            painter.circle(
                rect.center(),
                rect.width() / 2.,
                visuals.bg_fill,
                visuals.fg_stroke,
            );
            for t in ICOSPHERE.triangles.iter() {
                // TODO use fixed size array
                let rotated: Vec<_> = t
                    .iter()
                    .map(|&v| {
                        let r = *self.quat * v;
                        rect.center() + emath::vec2(r.x, r.y) * rect.width() / 2.
                    })
                    .collect();
                // TODO backface culling
                painter.line_segment([rotated[0], rotated[1]], visuals.fg_stroke);
                painter.line_segment([rotated[0], rotated[2]], visuals.fg_stroke);
                painter.line_segment([rotated[1], rotated[2]], visuals.fg_stroke);
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

struct IcoSphere {
    // TODO do a vertices, indices thing instead
    triangles: Vec<[Vec3; 3]>,
}

impl IcoSphere {
    fn new(divisions: u32) -> Self {
        let v = [
            vec3(f32::sqrt(8. / 9.), 0., -1. / 3.),
            vec3(-f32::sqrt(2. / 9.), f32::sqrt(2. / 3.), -1. / 3.),
            vec3(-f32::sqrt(2. / 9.), -f32::sqrt(2. / 3.), -1. / 3.),
            vec3(0., 0., 1.),
        ];
        for i in v {
            debug_assert!(i.is_normalized());
        }
        // TODO ccw
        let mut triangles = vec![
            [v[0], v[1], v[2]],
            [v[0], v[2], v[3]],
            [v[0], v[3], v[1]],
            [v[1], v[2], v[3]],
        ];
        for _ in 0..divisions {
            let prev: Vec<_> = triangles.drain(..).collect();
            for t in prev {
                let nv = [
                    ((t[0] + t[1]) / 2.).normalize(),
                    ((t[0] + t[2]) / 2.).normalize(),
                    ((t[1] + t[2]) / 2.).normalize(),
                ];
                triangles.push([t[0], nv[0], nv[1]]);
                triangles.push([t[1], nv[0], nv[2]]);
                triangles.push([t[2], nv[1], nv[2]]);
                triangles.push([nv[0], nv[1], nv[2]]);
            }
        }
        Self { triangles }
    }
}

static ICOSPHERE: Lazy<IcoSphere> = Lazy::new(|| IcoSphere::new(2));

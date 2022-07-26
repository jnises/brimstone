use eframe::{egui::{Widget, Sense, Context, Id}, emath};
use glam::{Quat, Vec2};


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
        let id = ui.make_persistent_id("rotator");
        const SIZE: f32 = 100.0;
        let (response, painter) = ui.allocate_painter(emath::vec2(SIZE, SIZE), Sense::click_and_drag());
        // TODO is this correct?
        let on = if let Some(pos) = response.interact_pointer_pos() {
            let data = ui.data().get_temp_mut_or_insert_with(id, || State { start_rot: *self.quat, start_pos: pos }).clone();
            let d = pos - data.start_pos;
            // TODO fix this
            *self.quat = data.start_rot * Quat::from_scaled_axis(glam::vec3(d.x, d.y, 0.) * 0.01);
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
            painter.circle(rect.center(), rect.width() / 2., visuals.bg_fill, visuals.fg_stroke);
            // TODO draw something
        }
        response
    }
}

#[derive(Clone, Copy, Debug)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
// #[cfg_attr(feature = "serde", serde(default))]
struct State {
    start_rot: Quat,
    start_pos: emath::Pos2,
}

// impl Default for State {
//     fn default() -> Self {
//         Self {
//             offset: Vec2::ZERO,
//             show_scroll: [false; 2],
//             vel: Vec2::ZERO,
//             scroll_start_offset_from_top_left: [None; 2],
//             scroll_stuck_to_end: [true; 2],
//         }
//     }
// }

// impl State {
//     pub fn load(ctx: &Context, id: Id) -> Option<Self> {
//         ctx.data().get_persisted(id)
//     }

//     pub fn store(self, ctx: &Context, id: Id) {
//         ctx.data().insert_persisted(id, self);
//     }

//     pub fn remove(self, ctx: &Context, id: Id) {
//         ctx.data().remove::<State>(id);
//     }
// }

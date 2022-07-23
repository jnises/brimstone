use std::ops::RangeInclusive;

use eframe::egui::{self, Widget};
use palette::Oklab;

use crate::utils::{resettable_slider_raw, NEUTRAL_LAB};

// TODO make this show a color picker?
pub struct LabUi<'a> {
    lab: &'a mut Oklab,
    label: &'a str,
    l_range: RangeInclusive<f32>,
    a_range: RangeInclusive<f32>,
    b_range: RangeInclusive<f32>,
    default_value: Oklab,
}

impl<'a> LabUi<'a> {
    pub fn new(lab: &'a mut Oklab, label: &'a str) -> Self {
        Self {
            lab,
            label,
            l_range: Oklab::min_l()..=Oklab::max_l(),
            a_range: Oklab::<f32>::min_a()..=Oklab::<f32>::max_a(),
            b_range: Oklab::<f32>::min_b()..=Oklab::<f32>::max_b(),
            default_value: NEUTRAL_LAB,
        }
    }

    pub fn l_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.l_range = range;
        self
    }

    #[allow(dead_code)]
    pub fn a_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.a_range = range;
        self
    }

    #[allow(dead_code)]
    pub fn b_range(mut self, range: RangeInclusive<f32>) -> Self {
        self.b_range = range;
        self
    }

    pub fn default_value(mut self, default_value: Oklab) -> Self {
        self.default_value = default_value;
        self
    }
}

impl<'a> Widget for LabUi<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::Grid::new(self.label)
            .show(ui, |ui| {
                ui.label(self.label);
                ui.end_row();
                resettable_slider_raw(ui, &mut self.lab.l, "L", self.l_range, self.default_value.l);
                ui.end_row();
                resettable_slider_raw(ui, &mut self.lab.a, "a", self.a_range, self.default_value.a);
                ui.end_row();
                resettable_slider_raw(ui, &mut self.lab.b, "b", self.b_range, self.default_value.b);
                ui.end_row();
            })
            .response
    }
}

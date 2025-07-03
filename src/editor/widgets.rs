use egui::*;
use nih_plug::{
    params::{BoolParam, FloatParam, IntParam, Param},
    prelude::ParamSetter,
};

use crate::utils;

pub fn create_toggle_button(ui: &mut Ui, param: &BoolParam, setter: &ParamSetter) -> Response {
    let mut value = param.value();
    let response = ui.toggle_value(&mut value, param.name());

    if response.changed() {
        setter.set_parameter(param, value);
    }
    response
}

pub fn create_checkbox(ui: &mut Ui, param: &BoolParam, setter: &ParamSetter) -> Response {
    ui.vertical(|ui| {
        let mut value = param.value();
        let response = ui.checkbox(&mut value, param.name());

        if response.changed() {
            setter.set_parameter(param, value);
        }
        response
    })
    .response
}

pub fn create_slider(
    ui: &mut Ui,
    param: &FloatParam,
    setter: &ParamSetter,
    orientation: SliderOrientation,
) -> Response {
    let ui_closure = |ui: &mut Ui| {
        ui.label(param.name());

        let mut value = param.modulated_normalized_value();
        let response = ui.add(
            Slider::new(&mut value, 0.0..=1.0)
                .show_value(false)
                .orientation(orientation)
                .handle_shape(egui::style::HandleShape::Circle),
        );

        if response.changed() {
            setter.set_parameter_normalized(param, value);
        }

        // Show formatted value
        ui.label(param.to_string());

        response
    };

    match orientation {
        SliderOrientation::Vertical => ui.vertical(ui_closure).inner,
        SliderOrientation::Horizontal => ui.horizontal(ui_closure).inner,
    }
}

pub fn create_combo_box(ui: &mut Ui, param: &IntParam, setter: &ParamSetter) -> Response {
    ui.horizontal(|ui| {
        for root in 0..12 {
            let checked = param.value() == root;
            let response = ui.selectable_label(checked, utils::semitones_to_note(root));
            if response.clicked() {
                setter.set_parameter(param, root);
            }
        }
    })
    .response
}

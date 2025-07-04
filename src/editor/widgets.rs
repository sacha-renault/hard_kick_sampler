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
    scroll_step: f32,
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

        if response.hovered()
            && ui.input(|i| i.pointer.button_double_clicked(PointerButton::Primary))
        {
            setter.set_parameter(param, param.default_plain_value());
        }

        if response.hovered() {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta > 0.0 {
                setter.set_parameter_normalized(param, value + scroll_step);
            } else if scroll_delta < 0.0 {
                setter.set_parameter_normalized(param, value - scroll_step);
            }
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

#[derive(Default, Clone)]
struct DragState {
    is_dragging: bool,
    start_pos: Pos2,
    start_value: i32,
}

pub fn create_integer_input(ui: &mut Ui, param: &IntParam, setter: &ParamSetter) -> Response {
    let current_value = param.value();
    let min_value = -24;
    let max_value = 24;

    // Create a unique ID for this parameter
    let id = ui.next_auto_id();

    // Get the current drag state
    let mut drag_state = ui
        .ctx()
        .memory_mut(|mem| mem.data.get_temp::<DragState>(id).unwrap_or_default());

    // Display the current value
    let text = format!("{}", current_value);
    let desired_size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    // Handle interaction
    if response.drag_started() {
        drag_state.is_dragging = true;
        drag_state.start_value = current_value;
        drag_state.start_pos = response.interact_pointer_pos().unwrap_or_default();
    }

    if response.dragged() && drag_state.is_dragging {
        if let Some(current_pos) = response.interact_pointer_pos() {
            // Calculate vertical delta (negative because screen Y increases downward)
            let delta_y = drag_state.start_pos.y - current_pos.y;

            // Sensitivity: pixels per integer step
            let sensitivity = 2.0;
            let value_delta = (delta_y / sensitivity) as i32;

            let new_value = (drag_state.start_value + value_delta).clamp(min_value, max_value);

            // Only update if the value actually changed
            if new_value != current_value {
                setter.set_parameter(param, new_value);
            }
        }
    }

    if response.double_clicked() {
        setter.set_parameter(param, param.default_plain_value());
    }

    if response.drag_stopped() || !ui.input(|i| i.pointer.any_down()) {
        drag_state.is_dragging = false;
    }

    if response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
        if scroll_delta > 0.0 {
            setter.set_parameter(param, current_value + 1);
        } else if scroll_delta < 0.0 {
            setter.set_parameter(param, current_value - 1);
        }
    }

    // Visual styling
    let visuals = ui.style().interact(&response);
    let bg_color = if drag_state.is_dragging {
        Color32::from_rgb(80, 80, 120)
    } else if response.hovered() {
        Color32::from_rgb(60, 60, 80)
    } else {
        Color32::from_rgb(40, 40, 60)
    };

    // Store the drag state
    ui.ctx().memory_mut(|mem| {
        mem.data.insert_temp(id, drag_state);
    });

    // Draw background
    ui.painter().rect_filled(rect, 3.0, bg_color);

    // Draw border
    ui.painter().rect_stroke(
        rect,
        3.0,
        Stroke::new(1.0, visuals.fg_stroke.color),
        StrokeKind::Inside,
    );

    // Draw text
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::default(),
        visuals.text_color(),
    );

    response
}

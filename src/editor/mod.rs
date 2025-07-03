#[allow(dead_code)]
mod widgets;

use std::path::PathBuf;
use std::sync::Arc;

use egui::*;
use nih_plug::prelude::ParamSetter;
use nih_plug::{editor::Editor, prelude::AsyncExecutor};
use nih_plug_egui::{create_egui_editor, EguiState};

use crate::params::{HardKickSamplerParams, SampleWrapperParams, MAX_SAMPLES};
use crate::plugin::HardKickSampler;
use crate::tasks::{TaskRequests, TaskResults};

const SPACE_AMOUT: f32 = 15_f32;

fn get_current_tab(ctx: &Context) -> usize {
    ctx.data(|data| data.get_temp::<usize>(Id::new("tab")).clone().unwrap_or(0))
}

fn set_current_tab(ctx: &Context, current_tab: usize) {
    ctx.data_mut(|data| {
        data.insert_temp(Id::new("tab"), current_tab);
    });
}

fn get_sample_name(sample_params: &SampleWrapperParams) -> Option<String> {
    sample_params.sample_path.read().ok().and_then(|guard| {
        guard.as_ref().and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        })
    })
}

fn get_sample_path(sample_params: &SampleWrapperParams) -> Option<String> {
    sample_params.sample_path.read().ok().and_then(|guard| {
        guard
            .as_ref()
            .and_then(|path| path.to_str().map(String::from))
    })
}

fn handle_file_drop(
    ctx: &egui::Context,
    async_executor: &AsyncExecutor<HardKickSampler>,
    current_tab: usize,
) {
    if let Some(file) = ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path.clone())) {
        if let Some(path) = file {
            async_executor.execute_background(TaskRequests::LoadFile(current_tab, path));
        }
    }
}

// fn render_file_controls(
//     ui: &mut egui::Ui,
//     async_executor: &AsyncExecutor<HardKickSampler>,
//     params: &HardKickSamplerParams,
//     current_tab: usize,
// ) {
//     ui.horizontal(|ui| {
//         ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
//             render_header_buttons(ui, async_executor, params, current_tab);
//         });
//     });
// }

fn render_file_controls(
    ui: &mut egui::Ui,
    async_executor: &AsyncExecutor<HardKickSampler>,
    params: &HardKickSamplerParams,
    current_tab: usize,
) {
    let current_file_path = get_sample_path(&params.samples[current_tab]);
    let current_file_name = get_sample_name(&params.samples[current_tab]);

    ui.horizontal(|ui| {
        // Label on the left
        ui.label(current_file_name.clone().unwrap_or_default());

        // Push buttons to the right
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("📁").clicked() {
                async_executor.execute_background(TaskRequests::OpenFilePicker(current_tab));
            }

            if ui
                .add_enabled(current_file_name.clone().is_some(), Button::new("Delete"))
                .clicked()
            {
                async_executor.execute_background(TaskRequests::TransfertTask(
                    TaskResults::ClearSample(current_tab),
                ));
            }

            if ui
                .add_enabled(current_file_path.is_some(), Button::new(">"))
                .clicked()
            {
                if let Some(file) = current_file_path.clone() {
                    async_executor
                        .execute_background(TaskRequests::LoadNextFile(current_tab, file.clone()));
                }
            }

            if ui
                .add_enabled(current_file_path.is_some(), Button::new("<"))
                .clicked()
            {
                if let Some(file) = current_file_path {
                    async_executor.execute_background(TaskRequests::LoadPreviousFile(
                        current_tab,
                        file.clone(),
                    ));
                }
            }
        });
    });
}

fn render_tabs(ui: &mut egui::Ui, current_tab: usize) -> usize {
    let mut new_tab = current_tab;

    ui.horizontal(|ui| {
        for tab in 0..MAX_SAMPLES {
            if ui
                .selectable_label(current_tab == tab, format!("Sample {}", tab + 1))
                .clicked()
            {
                new_tab = tab;
            }
        }
    });

    new_tab
}

fn render_sample_controls(
    ui: &mut egui::Ui,
    params: &HardKickSamplerParams,
    current_tab: usize,
    setter: &ParamSetter,
) {
    let sample_params = &params.samples[current_tab];

    render_mute_control(ui, sample_params, setter);
    ui.add_space(SPACE_AMOUT);

    render_adsr_controls(ui, sample_params, setter);
    ui.add_space(SPACE_AMOUT);

    render_tonal_controls(ui, sample_params, setter);
    ui.add_space(SPACE_AMOUT);

    render_semitone_control(ui, sample_params, setter);
    ui.add_space(SPACE_AMOUT);

    render_gain_control(ui, sample_params, setter);
}

fn render_mute_control(
    ui: &mut egui::Ui,
    sample_params: &SampleWrapperParams,
    setter: &ParamSetter,
) {
    ui.horizontal(|ui| {
        widgets::create_checkbox(ui, &sample_params.muted, setter);
    });
}

fn render_adsr_controls(
    ui: &mut egui::Ui,
    sample_params: &SampleWrapperParams,
    setter: &ParamSetter,
) {
    ui.label("ADSR");
    ui.horizontal(|ui| {
        let orientation = SliderOrientation::Vertical;
        widgets::create_slider(ui, &sample_params.attack, setter, orientation);
        widgets::create_slider(ui, &sample_params.decay, setter, orientation);
        widgets::create_slider(ui, &sample_params.sustain, setter, orientation);
        widgets::create_slider(ui, &sample_params.release, setter, orientation);
    });
}

fn render_tonal_controls(
    ui: &mut egui::Ui,
    sample_params: &SampleWrapperParams,
    setter: &ParamSetter,
) {
    ui.horizontal(|ui| {
        widgets::create_checkbox(ui, &sample_params.is_tonal, setter);
        if sample_params.is_tonal.value() {
            widgets::create_combo_box(ui, &sample_params.root_note, setter);
        }
    });
}

fn render_semitone_control(
    ui: &mut egui::Ui,
    sample_params: &SampleWrapperParams,
    setter: &ParamSetter,
) {
    widgets::create_integer_input(ui, &sample_params.semitone_offset, setter);
}

fn render_gain_control(
    ui: &mut egui::Ui,
    sample_params: &SampleWrapperParams,
    setter: &ParamSetter,
) {
    widgets::create_slider(
        ui,
        &sample_params.gain,
        setter,
        SliderOrientation::Horizontal,
    );
}

pub fn create_editor(
    params: Arc<HardKickSamplerParams>,
    async_executor: AsyncExecutor<HardKickSampler>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        EguiState::from_size(800, 600),
        params.clone(),
        |_ctx, _params| {},
        move |ctx, setter, params| {
            let mut current_tab = get_current_tab(ctx);

            handle_file_drop(ctx, &async_executor, current_tab);

            CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.label(RichText::new("Hard Kick Sampler").size(36.0).strong());
                    ui.add_space(SPACE_AMOUT);

                    current_tab = render_tabs(ui, current_tab);
                    ui.add_space(SPACE_AMOUT);

                    render_sample_controls(ui, params, current_tab, setter);

                    // Simple waveform placeholder at the bottom
                    ui.add_space(SPACE_AMOUT);
                    render_file_controls(ui, &async_executor, params, current_tab);
                    let rect = ui
                        .allocate_response(
                            egui::Vec2::new(ui.available_width(), 150.0),
                            Sense::hover(),
                        )
                        .rect;

                    // Draw a simple rectangle
                    ui.painter().rect_filled(
                        rect,
                        CornerRadius::same(4),
                        egui::Color32::from_gray(30),
                    );

                    // Draw border
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::same(4),
                        egui::Stroke::new(1.0, egui::Color32::from_gray(100)),
                        StrokeKind::Middle,
                    );

                    // Add placeholder text
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Waveform will go here",
                        egui::FontId::proportional(16.0),
                        egui::Color32::from_gray(150),
                    );
                })
            });

            set_current_tab(ctx, current_tab);
        },
    )
}

mod knob;
#[allow(dead_code)]
mod theme;
mod waveform;
#[allow(dead_code)]
mod widgets;

use egui::*;
use nih_plug::prelude::ParamSetter;
use nih_plug::{editor::Editor, prelude::AsyncExecutor};
use nih_plug_egui::{create_egui_editor, EguiState};

use crate::editor::waveform::render_waveform_stereo;
use crate::params::{HardKickSamplerParams, SampleWrapperParams, MAX_SAMPLES};
use crate::plugin::HardKickSampler;
use crate::shared_states::SharedStates;
use crate::tasks::{TaskRequests, TaskResults};

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

fn render_panel<R>(
    ui: &mut Ui,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    Frame::new()
        .fill(theme::BACKGROUND_COLOR)
        .stroke(Stroke::new(theme::STANDARD_STROKE, theme::BORDER_COLOR))
        .corner_radius(theme::STANDARD_ROUNDING)
        .inner_margin(Margin::same(theme::PANEL_PADDING as i8))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(title)
                        .strong()
                        .color(theme::TEXT_COLOR_ACCENT),
                );
                ui.add_space(4.0);
                add_contents(ui)
            })
            .inner
        })
}

fn render_adsr_panel(ui: &mut Ui, sample_params: &SampleWrapperParams, setter: &ParamSetter) {
    render_panel(ui, "ADSR", |ui| {
        ui.horizontal(|ui| {
            ui.columns(4, |columns| {
                widgets::create_knob(&mut columns[0], &sample_params.attack, setter, 0.1);
                widgets::create_knob(&mut columns[1], &sample_params.decay, setter, 0.1);
                widgets::create_knob(&mut columns[2], &sample_params.sustain, setter, 0.1);
                widgets::create_knob(&mut columns[3], &sample_params.release, setter, 0.1);
            });
        });
    });
}

fn render_time_control_panel(
    ui: &mut Ui,
    sample_params: &SampleWrapperParams,
    setter: &ParamSetter,
) {
    render_panel(ui, "Time Control", |ui| {
        ui.horizontal(|ui| {
            ui.columns(2, |columns| {
                widgets::create_knob(&mut columns[0], &sample_params.trim_start, setter, 0.1);
                widgets::create_knob(&mut columns[1], &sample_params.delay_start, setter, 0.1);
            });
        });
    });
}

fn render_tonal_panel(ui: &mut Ui, sample_params: &SampleWrapperParams, setter: &ParamSetter) {
    render_panel(ui, "Tonal", |ui| {
        ui.vertical(|ui| {
            widgets::create_checkbox(ui, &sample_params.is_tonal, setter);

            // We don't need the thing to be tonal, we just disable root
            // note when the value isn't checked
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Root Note:");
                widgets::create_combo_box(
                    ui,
                    &sample_params.root_note,
                    setter,
                    sample_params.is_tonal.value(),
                );
            });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Semi:");
                widgets::create_integer_input(ui, &sample_params.semitone_offset, setter);
            });
        });
    });
}

fn render_gain_panel(ui: &mut Ui, sample_params: &SampleWrapperParams, setter: &ParamSetter) {
    render_panel(ui, "Gain", |ui| {
        // Add horizontal layout to match other panels
        ui.horizontal(|ui| {
            ui.vertical_centered(|ui| {
                widgets::create_knob(ui, &sample_params.gain, setter, 0.025);
            });
        });
    });
}

fn render_control_panels(ui: &mut Ui, sample_params: &SampleWrapperParams, setter: &ParamSetter) {
    ui.vertical(|ui| {
        // Tonal Panel - full width at top
        render_tonal_panel(ui, sample_params, setter);

        ui.add_space(theme::PANEL_SPACING);

        // ADSR and Gain panels - horizontal layout below
        ui.horizontal_top(|ui| {
            // Use horizontal_top for top alignment
            // ADSR Panel - 40% width
            let total_spacing = theme::PANEL_SPACING * 2.0;
            let available_width = ui.available_width() - total_spacing;
            let panel_height = 120.0;

            // Calculate widths based on available space after spacing
            let adsr_width = available_width * 0.4;
            let time_width = available_width * 0.4;
            let gain_width = available_width * 0.2;

            ui.allocate_ui_with_layout(
                Vec2::new(adsr_width, panel_height),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ui.set_min_size(Vec2::new(adsr_width, panel_height));
                    render_adsr_panel(ui, sample_params, setter)
                },
            );

            ui.add_space(theme::PANEL_SPACING);

            // Add a placeholder panel
            ui.allocate_ui_with_layout(
                Vec2::new(time_width, panel_height),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ui.set_min_size(Vec2::new(adsr_width, panel_height));
                    render_time_control_panel(ui, sample_params, setter)
                },
            );

            ui.add_space(theme::PANEL_SPACING);

            // Gain Panel - 20% width
            ui.allocate_ui_with_layout(
                Vec2::new(gain_width, panel_height),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ui.set_min_size(Vec2::new(gain_width, panel_height));
                    render_gain_panel(ui, sample_params, setter)
                },
            );
        });
    });
}

fn render_sample_info_strip(
    ui: &mut Ui,
    async_executor: &AsyncExecutor<HardKickSampler>,
    params: &HardKickSamplerParams,
    current_tab: usize,
    setter: &ParamSetter,
) {
    let sample_params = &params.samples[current_tab];
    let current_file_path = get_sample_path(sample_params);
    let current_file_name = get_sample_name(sample_params);

    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), 30.0), // Set explicit height too
        Layout::left_to_right(Align::Center),
        |ui| {
            Frame::new()
                .fill(theme::BACKGROUND_COLOR_FOCUSED)
                .stroke(Stroke::new(theme::STANDARD_STROKE, theme::BORDER_COLOR))
                .corner_radius(theme::SMALL_ROUNDING)
                .inner_margin(Margin::same(6))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width() - theme::PANEL_SPACING * 2.);
                    ui.horizontal_centered(|ui| {
                        // Mute checkbox on the left
                        widgets::create_checkbox(ui, &sample_params.muted, setter);

                        ui.add_space(10.0);

                        // Sample name
                        ui.label(
                            RichText::new(
                                current_file_name.unwrap_or("No sample loaded".to_string()),
                            )
                            .color(theme::TEXT_COLOR),
                        );

                        // Push file controls to the right
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("Delete").clicked() {
                                async_executor.execute_background(TaskRequests::TransfertTask(
                                    TaskResults::ClearSample(current_tab),
                                ));
                            }

                            if ui
                                .add_enabled(current_file_path.is_some(), Button::new(">"))
                                .clicked()
                            {
                                if let Some(file) = current_file_path.clone() {
                                    async_executor.execute_background(TaskRequests::LoadNextFile(
                                        current_tab,
                                        file.clone(),
                                    ));
                                }
                            }

                            if ui
                                .add_enabled(current_file_path.is_some(), Button::new("<"))
                                .clicked()
                            {
                                if let Some(file) = current_file_path {
                                    async_executor.execute_background(
                                        TaskRequests::LoadPreviousFile(current_tab, file.clone()),
                                    );
                                }
                            }

                            if ui.button("📁").clicked() {
                                async_executor
                                    .execute_background(TaskRequests::OpenFilePicker(current_tab));
                            }
                        });
                    });
                });
        },
    );
}

fn render_waveform_display(
    ui: &mut Ui,
    waveform_data: Option<&Vec<f32>>,
    num_channels: usize,
    params: &SampleWrapperParams,
) {
    // Render image if needed
    match waveform_data {
        Some(data) if !data.is_empty() => {
            let height_per_channel =
                (ui.available_height() - theme::SPACE_AMOUNT) / num_channels as f32;
            let trim_start = params.trim_start.value();
            for channel_index in 0..num_channels {
                let rect = paint_rect(ui, height_per_channel, ui.available_width());
                ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
                    render_waveform_stereo(
                        ui,
                        data,
                        channel_index,
                        num_channels,
                        trim_start,
                        44100.,
                    )
                });
            }
        }
        _ => {
            let rect = paint_rect(
                ui,
                ui.available_height() - theme::SPACE_AMOUNT,
                ui.available_width(),
            );
            ui.allocate_new_ui(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::centered_and_justified(
                        egui::Direction::LeftToRight,
                    )),
                |ui| ui.label("No waveform data"),
            );
        }
    }
}

fn paint_rect(ui: &mut Ui, height: f32, width: f32) -> Rect {
    let rect = ui
        .allocate_response(Vec2::new(width, height), Sense::hover())
        .rect;

    // Draw waveform background
    ui.painter()
        .rect_filled(rect, theme::STANDARD_ROUNDING, Color32::from_gray(25));

    // Draw border
    ui.painter().rect_stroke(
        rect,
        theme::STANDARD_ROUNDING,
        Stroke::new(theme::STANDARD_STROKE, theme::BORDER_COLOR),
        StrokeKind::Middle,
    );
    rect
}

fn render_tabs(ui: &mut egui::Ui, current_tab: usize) -> usize {
    let mut new_tab = current_tab;

    ui.horizontal(|ui| {
        for tab in (0..MAX_SAMPLES).rev() {
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

fn show_with_margin<R>(
    ui: &mut Ui,
    margins: (f32, f32),
    layout: Layout,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Response {
    ui.allocate_ui_with_layout(
        Vec2::new(
            ui.available_width() - margins.0 * 2.,
            ui.available_height() - margins.1 * 2.,
        ),
        layout,
        add_contents,
    )
    .response
}

pub fn create_editor(
    states: SharedStates,
    async_executor: AsyncExecutor<HardKickSampler>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        EguiState::from_size(800, 600),
        states,
        |_ctx, _params| {},
        move |ctx, setter, states| {
            let mut current_tab = get_current_tab(ctx);
            let params = states.params.clone();

            handle_file_drop(ctx, &async_executor, current_tab);
            theme::apply_theme(ctx);

            let current_sample_params = &params.samples[current_tab];

            CentralPanel::default().show(ctx, |ui| {
                show_with_margin(
                    ui,
                    (theme::PANEL_PADDING, theme::PANEL_PADDING),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        // Header with title and tabs
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Hard Kick Sampler").size(24.0).strong());

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                current_tab = render_tabs(ui, current_tab);
                            });
                        });

                        ui.add_space(theme::SPACE_AMOUNT);

                        // Control panels in the middle
                        render_control_panels(ui, current_sample_params, setter);

                        ui.add_space(theme::SPACE_AMOUNT);

                        // Sample info strip above waveform
                        render_sample_info_strip(ui, &async_executor, &params, current_tab, setter);

                        ui.add_space(8.0);

                        // Waveform display takes remaining space
                        let waveform_data = states.wave_readers[current_tab].read().ok();
                        render_waveform_display(
                            ui,
                            waveform_data.as_deref(),
                            2,
                            current_sample_params,
                        );
                    },
                );
            });

            set_current_tab(ctx, current_tab);
        },
    )
}

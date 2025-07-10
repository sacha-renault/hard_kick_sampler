#[allow(dead_code)]
mod theme;
mod waveform;
#[allow(dead_code)]
mod widgets;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use egui::*;
use nih_plug::prelude::ParamSetter;
use nih_plug::{editor::Editor, prelude::AsyncExecutor};
use nih_plug_egui::{create_egui_editor, EguiState};

use crate::editor::waveform::{render_waveform_stereo, PlotData};
use crate::params::{BlendGroup, HardKickSamplerParams, SamplePlayerParams, MAX_SAMPLES};
use crate::plugin::HardKickSampler;
use crate::shared_states::SharedStates;
use crate::tasks::{AudioData, TaskRequests, TaskResults};
use crate::utils;

const PANEL_HEIGHT: f32 = 135.;

fn get_current_tab(ctx: &Context) -> usize {
    ctx.data(|data| data.get_temp::<usize>(Id::new("tab")).unwrap_or(0))
}

fn set_current_tab(ctx: &Context, current_tab: usize) {
    ctx.data_mut(|data| {
        data.insert_temp(Id::new("tab"), current_tab);
    });
}

fn get_sample_name(sample_params: &SamplePlayerParams) -> Option<String> {
    sample_params.sample_path.read().ok().and_then(|guard| {
        guard.as_ref().and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(String::from)
        })
    })
}

fn get_sample_path(sample_params: &SamplePlayerParams) -> Option<String> {
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
    if let Some(Some(path)) = ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path.clone())) {
        async_executor.execute_background(TaskRequests::LoadFile(current_tab, path));
    }
}

fn render_panel<R>(
    ui: &mut Ui,
    title: &str,
    width: f32,
    height: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Response {
    // Remove padding and stroke to the inner width
    let corrected_width = width - theme::PANEL_PADDING * 2. - theme::STANDARD_STROKE * 2.;
    let corrected_height = height - theme::PANEL_PADDING * 2. - theme::STANDARD_STROKE * 2.;
    let panel_rect = Rect::from_min_size(
        ui.cursor().min,
        Vec2::new(corrected_width, corrected_height),
    );

    ui.allocate_new_ui(UiBuilder::new().max_rect(panel_rect), |ui| {
        Frame::new()
            .fill(theme::BACKGROUND_COLOR_FOCUSED)
            .stroke(Stroke::new(theme::STANDARD_STROKE, theme::BORDER_COLOR))
            .corner_radius(theme::STANDARD_ROUNDING)
            .inner_margin(Margin::same(theme::PANEL_PADDING as i8))
            .show(ui, |ui| {
                ui.set_width(corrected_width);
                ui.set_height(corrected_height);

                ui.vertical(|ui| {
                    // Title header
                    ui.horizontal(|ui| {
                        ui.set_height(24.0); // Fixed header height
                        ui.label(
                            RichText::new(title)
                                .size(14.0)
                                .strong()
                                .color(theme::TEXT_COLOR_ACCENT),
                        );
                    });

                    // Content area - takes remaining space
                    ui.allocate_new_ui(
                        UiBuilder::new().max_rect(Rect::from_min_size(
                            ui.cursor().min,
                            Vec2::new(ui.available_width(), ui.available_height()),
                        )),
                        add_contents,
                    );
                });
            });
    })
    .response
}

fn render_sample_info_strip(
    ui: &mut Ui,
    async_executor: &AsyncExecutor<HardKickSampler>,
    params: Arc<HardKickSamplerParams>,
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
                        let mut value = sample_params.muted.value();
                        if ui.checkbox(&mut value, "Muted").clicked() {
                            setter.set_parameter(&sample_params.muted, value);
                        }

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

                            let next_file = current_file_path
                                .clone()
                                .and_then(|file| utils::get_next_file_in_directory_wrap(&file));
                            if ui
                                .add_enabled(next_file.is_some(), Button::new(">"))
                                .clicked()
                            {
                                if let Some(file) = next_file {
                                    async_executor.execute_background(TaskRequests::LoadFile(
                                        current_tab,
                                        file,
                                    ));
                                }
                            }

                            let previous_file = current_file_path
                                .clone()
                                .and_then(|file| utils::get_previous_file_in_directory_wrap(&file));
                            if ui
                                .add_enabled(previous_file.is_some(), Button::new("<"))
                                .clicked()
                            {
                                if let Some(file) = previous_file {
                                    async_executor.execute_background(TaskRequests::LoadFile(
                                        current_tab,
                                        file,
                                    ));
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
    shared_data: Option<&AudioData>,
    params: &SamplePlayerParams,
    current_position: Arc<AtomicU64>,
) {
    // Render image if needed
    match shared_data {
        Some(shared_data) if !shared_data.data.is_empty() => {
            // Get the number of channels
            let num_channels = shared_data.spec.channels as usize;

            // Get ui size available
            let height_per_channel =
                (ui.available_height() - theme::SPACE_AMOUNT) / num_channels as f32;

            // sample specific stuff
            let trim_start = params.trim_start.value();
            let delay_start = params.delay_start.value();
            let position = current_position.load(Ordering::Relaxed);

            // Data to be displayed
            let line_data = PlotData::new(
                &shared_data.data,
                trim_start,
                delay_start,
                shared_data.spec.sample_rate as f32,
                shared_data.spec.channels as usize,
                position,
            );

            // iterate for all the channels available
            for channel_index in 0..num_channels {
                // Paint rect
                let rect = paint_rect(ui, height_per_channel, ui.available_width());

                ui.allocate_new_ui(UiBuilder::new().max_rect(rect), |ui| {
                    render_waveform_stereo(ui, channel_index, &line_data);
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
    // Current tab
    let mut new_tab = current_tab;

    ui.horizontal(|ui| {
        ui.columns(MAX_SAMPLES, |ui| {
            for (tab, sub_ui) in ui.iter_mut().enumerate() {
                if sub_ui
                    .selectable_label(current_tab == tab, format!("Sample {}", tab + 1))
                    .clicked()
                {
                    new_tab = tab;
                }
            }
        })
    });

    new_tab
}

fn render_control_tonal_blend(
    ui: &mut Ui,
    global_params: Arc<HardKickSamplerParams>,
    sample_params: &SamplePlayerParams,
    setter: &ParamSetter,
) {
    let width = ui.available_width() - 2. * 8.;
    ui.horizontal(|ui| {
        let (tonal_width, sample_width, global_width) = (0.5, 0.15, 0.35);
        render_panel(ui, "Tonal", width * tonal_width, PANEL_HEIGHT, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Semi:");
                    widgets::create_integer_input(ui, &sample_params.semitone_offset, setter);
                });

                ui.add_space(8.0);

                widgets::create_toggle_button(ui, &sample_params.is_tonal, setter);

                ui.add_space(8.0);
                widgets::create_combo_box(
                    ui,
                    &sample_params.root_note,
                    setter,
                    sample_params.is_tonal.value(),
                );
            });
        });
        render_panel(
            ui,
            "Sample Blend",
            width * sample_width,
            PANEL_HEIGHT,
            |ui| {
                let current_group = sample_params.blend_group.value();
                if ui
                    .radio(current_group == BlendGroup::None, "None")
                    .clicked()
                {
                    setter.set_parameter(&sample_params.blend_group, BlendGroup::None);
                }
                if ui
                    .radio(current_group == BlendGroup::Start, "Start")
                    .clicked()
                {
                    setter.set_parameter(&sample_params.blend_group, BlendGroup::Start);
                }
                if ui.radio(current_group == BlendGroup::End, "End").clicked() {
                    setter.set_parameter(&sample_params.blend_group, BlendGroup::End);
                }
            },
        );
        render_panel(
            ui,
            "Global Blend Options",
            width * global_width,
            PANEL_HEIGHT,
            |ui| {
                ui.horizontal(|ui| {
                    ui.columns(2, |columns| {
                        widgets::create_knob(
                            &mut columns[0],
                            &global_params.blend_time,
                            setter,
                            0.1,
                        );
                        widgets::create_knob(
                            &mut columns[1],
                            &global_params.blend_transition,
                            setter,
                            0.1,
                        );
                    });
                });
            },
        );
        // ui.label(format!("{}", ui.style().spacing.item_spacing.x))
    });
}

fn render_control_adsr_time_gain(ui: &mut Ui, params: &SamplePlayerParams, setter: &ParamSetter) {
    let width = ui.available_width() - 2. * 8.;
    ui.horizontal(|ui| {
        render_panel(ui, "Adsr", width * 0.4, PANEL_HEIGHT, |ui| {
            ui.horizontal(|ui| {
                ui.columns(4, |columns| {
                    widgets::create_knob(&mut columns[0], &params.attack, setter, 0.1);
                    widgets::create_knob(&mut columns[1], &params.decay, setter, 0.1);
                    widgets::create_knob(&mut columns[2], &params.sustain, setter, 0.1);
                    widgets::create_knob(&mut columns[3], &params.release, setter, 0.1);
                });
            });
        });
        render_panel(ui, "Time Control", width * 0.35, PANEL_HEIGHT, |ui| {
            ui.horizontal(|ui| {
                ui.columns(2, |columns| {
                    widgets::create_knob(&mut columns[0], &params.trim_start, setter, 0.1);
                    widgets::create_knob(&mut columns[1], &params.delay_start, setter, 0.1);
                });
            });
        });
        render_panel(ui, "Gain", width * 0.25, PANEL_HEIGHT, |ui| {
            ui.horizontal(|ui| {
                ui.vertical_centered(|ui| {
                    widgets::create_knob(ui, &params.gain, setter, 0.025);
                });
            });
        });
        // ui.label(format!("{}", ui.style().spacing.item_spacing.x))
    });
}

fn show_with_margin<R>(
    ui: &mut Ui,
    margins: (f32, f32),
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Response {
    let available_rect = ui.available_rect_before_wrap();
    let margin_rect = Rect::from_min_size(
        available_rect.min + Vec2::new(margins.0, margins.1),
        available_rect.size() - Vec2::new(margins.0 * 2., margins.1 * 2.),
    );

    ui.allocate_new_ui(UiBuilder::new().max_rect(margin_rect), add_contents)
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
            let current_position = states.positions[current_tab].clone();
            let guard_option = states.shared_buffer[current_tab].read().ok();
            let shared_data: Option<&AudioData> = guard_option
                .as_ref() // Option<&RwLockReadGuard<Option<AudioData>>>
                .and_then(|guard| guard.as_ref());

            CentralPanel::default().show(ctx, |ui| {
                show_with_margin(ui, (theme::PANEL_PADDING, theme::PANEL_PADDING), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hard Kick Sampler").size(36.0).strong());
                    });

                    current_tab = render_tabs(ui, current_tab);

                    ui.separator();

                    // Render the first row of controls
                    render_control_tonal_blend(ui, params.clone(), current_sample_params, setter);
                    render_control_adsr_time_gain(ui, current_sample_params, setter);

                    // Sample info strip above waveform
                    render_sample_info_strip(
                        ui,
                        &async_executor,
                        params.clone(),
                        current_tab,
                        setter,
                    );

                    // Waveform display takes remaining space
                    render_waveform_display(
                        ui,
                        shared_data,
                        current_sample_params,
                        current_position,
                    );
                });
            });

            set_current_tab(ctx, current_tab);
        },
    )
}

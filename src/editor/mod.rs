#[allow(dead_code)]
mod theme;
#[allow(dead_code)]
mod widgets;

use egui::*;
use nih_plug::prelude::ParamSetter;
use nih_plug::{editor::Editor, prelude::AsyncExecutor};
use nih_plug_egui::{create_egui_editor, EguiState};

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
            let orientation = SliderOrientation::Vertical;
            widgets::create_slider(ui, &sample_params.attack, setter, orientation, 0.1);
            widgets::create_slider(ui, &sample_params.decay, setter, orientation, 0.1);
            widgets::create_slider(ui, &sample_params.sustain, setter, orientation, 0.1);
            widgets::create_slider(ui, &sample_params.release, setter, orientation, 0.1);
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
        // Vertical gain slider to save space
        widgets::create_slider(
            ui,
            &sample_params.gain,
            setter,
            SliderOrientation::Vertical,
            0.025,
        );
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
            // ADSR Panel - 80% width
            let available_width = ui.available_width() - theme::PANEL_SPACING;
            let panel_height = 120.0;
            let adsr_width = available_width * 0.8;

            ui.allocate_ui_with_layout(
                Vec2::new(adsr_width, panel_height),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ui.set_min_size(Vec2::new(adsr_width, panel_height));
                    render_adsr_panel(ui, sample_params, setter)
                },
            );

            ui.add_space(theme::PANEL_SPACING);

            // Gain Panel - 20% width
            let gain_width = available_width * 0.2;
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
        Vec2::new(ui.available_width() - 2. * theme::PANEL_SPACING, 30.0), // Set explicit height too
        Layout::left_to_right(Align::Center),
        |ui| {
            Frame::new()
                .fill(theme::BACKGROUND_COLOR_FOCUSED)
                .stroke(Stroke::new(theme::STANDARD_STROKE, theme::BORDER_COLOR))
                .corner_radius(theme::SMALL_ROUNDING)
                .inner_margin(Margin::same(6))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width() - theme::PANEL_SPACING * 2.);
                    ui.horizontal(|ui| {
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

fn render_waveform_display(ui: &mut Ui) {
    let available_height = ui.available_height() - theme::PANEL_SPACING;
    let rect = ui
        .allocate_response(
            Vec2::new(
                ui.available_width() - theme::PANEL_SPACING * 2.,
                available_height,
            ),
            Sense::hover(),
        )
        .rect;

    // Draw waveform background
    ui.painter()
        .rect_filled(rect, theme::STANDARD_ROUNDING, Color32::from_gray(25));

    // Draw border
    ui.painter().rect_stroke(
        rect,
        theme::STANDARD_ROUNDING,
        Stroke::new(theme::STANDARD_STROKE, theme::BORDER_COLOR),
        StrokeKind::Inside,
    );

    // Add placeholder text
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        "Waveform Display",
        theme::FONT_LARGE,
        theme::TEXT_COLOR_DISABLED,
    );
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

            CentralPanel::default().show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Header with title and tabs
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hard Kick Sampler").size(24.0).strong());

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            current_tab = render_tabs(ui, current_tab);
                        });
                    });

                    ui.add_space(theme::SPACE_AMOUNT);

                    // Control panels in the middle
                    render_control_panels(ui, &params.samples[current_tab], setter);

                    ui.add_space(theme::SPACE_AMOUNT);

                    // Sample info strip above waveform
                    render_sample_info_strip(ui, &async_executor, &params, current_tab, setter);

                    ui.add_space(8.0);

                    // Waveform display takes remaining space
                    let _ = match states.wave_readers[current_tab].read() {
                        Ok(data) => Some(data),
                        Err(_) => None,
                    }; // Can now use data to render the wave ! :)

                    render_waveform_display(ui);
                });
            });

            set_current_tab(ctx, current_tab);
        },
    )
}

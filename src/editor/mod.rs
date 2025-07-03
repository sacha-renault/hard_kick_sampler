#[allow(dead_code)]
mod widgets;

use std::path::PathBuf;
use std::sync::Arc;

use egui::*;
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

fn get_sample_name(sample_params: &SampleWrapperParams) -> String {
    sample_params
        .sample_path
        .read()
        .ok()
        .and_then(|guard| {
            guard.as_ref().and_then(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(String::from)
            })
        })
        .unwrap_or_else(|| "No sample loaded".to_string())
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
            // Get the current editor state
            let mut current_tab = get_current_tab(ctx);
            let sample_params = &params.samples[current_tab];

            // Enable drag and drop
            if let Some(file) = ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path.clone())) {
                let path = file.clone().unwrap_or(PathBuf::from(""));
                async_executor.execute_background(TaskRequests::LoadFile(5, path));
            }

            CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    ui.add_space(SPACE_AMOUT);

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Hard Kick Sampler").size(18.0).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.button("📁").clicked() {
                                async_executor
                                    .execute_background(TaskRequests::OpenFilePicker(current_tab));
                            }
                            if ui.button("Delete").clicked() {
                                async_executor.execute_background(TaskRequests::TransfertTask(
                                    TaskResults::ClearSample(current_tab),
                                ));
                            }
                        });
                    });

                    ui.add_space(SPACE_AMOUT);

                    // Tabs
                    ui.horizontal(|ui| {
                        for tab in 0..MAX_SAMPLES {
                            if ui
                                .selectable_label(current_tab == tab, format!("Sample {}", tab + 1))
                                .clicked()
                            {
                                current_tab = tab;
                            }
                        }
                    });

                    ui.add_space(SPACE_AMOUT);

                    // Some params
                    ui.horizontal(|ui| {
                        widgets::create_toggle_button(ui, &sample_params.muted, setter);
                    });

                    ui.add_space(SPACE_AMOUT);

                    ui.label("ADSR");
                    ui.horizontal(|ui| {
                        let orientation = SliderOrientation::Vertical;
                        widgets::create_slider(ui, &sample_params.attack, setter, orientation);
                        widgets::create_slider(ui, &sample_params.decay, setter, orientation);
                        widgets::create_slider(ui, &sample_params.sustain, setter, orientation);
                        widgets::create_slider(ui, &sample_params.release, setter, orientation);
                    });

                    ui.add_space(SPACE_AMOUT);

                    ui.horizontal(|ui| {
                        widgets::create_toggle_button(ui, &sample_params.is_tonal, setter);
                        if sample_params.is_tonal.value() {
                            widgets::create_combo_box(ui, &sample_params.root_note, setter);
                        }
                    });

                    // Show what is loaded
                    ui.label(get_sample_name(sample_params));
                })
            });

            // Insert data at the end
            set_current_tab(ctx, current_tab);
        },
    )
}

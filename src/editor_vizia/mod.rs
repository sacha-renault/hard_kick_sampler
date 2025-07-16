mod customs;
mod style;
mod widgets;

use std::path::PathBuf;
use std::sync::Arc;

use cyma::prelude::*;
use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::RawParamEvent;
use nih_plug_vizia::{create_vizia_editor, ViziaState};

// use crate::editor::waveform::WavePlot;
use crate::params::{SamplePlayerParams, MAX_SAMPLES};
use crate::plugin::HardKickSampler;
use crate::shared_states::SharedStates;
use crate::tasks::TaskRequests;
use crate::utils;
use style::*;

pub enum AppEvent {
    SelectSample(usize),
    FileLoading(usize, PathBuf),
}

#[derive(Lens)]
pub struct Data {
    states: Arc<SharedStates>,
    selected_sample: usize,
    executor: AsyncExecutor<HardKickSampler>,
}

impl Model for Data {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SelectSample(index) => {
                self.selected_sample = *index;
            }
            AppEvent::FileLoading(index, path) => {
                self.executor
                    .execute_background(TaskRequests::LoadFile(*index, path.clone()));

                // Check if the sample is tonal
                // We also check the current value of the root note to set it
                let root = match utils::get_root_note_from_filename(
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .into(),
                ) {
                    Some(root) => root,
                    _ => 0,
                };
                // Get the param
                let param = &get_param(&self.states, self.selected_sample).root_note;
                let ptr = param.as_ptr();
                let normalized = param.preview_normalized(root);
                cx.emit(RawParamEvent::BeginSetParameter(ptr));
                cx.emit(RawParamEvent::SetParameterNormalized(ptr, normalized));
                cx.emit(RawParamEvent::EndSetParameter(ptr));
            }
        });
    }
}

pub fn get_param(st: &Arc<SharedStates>, index: usize) -> &SamplePlayerParams {
    &st.params.samples[index]
}

fn create_title_section(cx: &mut Context) {
    // Title - this doesn't need to change
    Label::new(cx, "Hard Kick Sampler")
        .color(TEXT_COLOR_ACCENT)
        .class("title");
}

fn create_sample_tabs(cx: &mut Context) {
    // Tabs - OUTSIDE the binding so they keep their event handlers
    HStack::new(cx, |cx| {
        for index in 0..MAX_SAMPLES {
            let txt = format!("Sample {}", index + 1);
            Button::new(
                cx,
                move |cx| cx.emit(AppEvent::SelectSample(index)), // Add the event handler!
                |cx| Label::new(cx, &txt).cursor(CursorIcon::Hand),
            )
            .hoverable(true)
            .class("tab")
            .toggle_class(
                "selected",
                Data::selected_sample.map(move |selected| *selected == index),
            );
        }
    })
    .width(Stretch(1.0))
    .height(Auto);
}

fn create_first_panel_row(cx: &mut Context, index: usize) {
    // First panel row - equal height
    HStack::new(cx, |cx| {
        widgets::WidgetPanel::vnew(cx, "Tonal", |cx| {
            widgets::ParamDragNumber::new(cx, Data::states, move |st| {
                &get_param(st, index).semitone_offset
            });
            widgets::ParamSwitch::new(cx, Data::states, move |st| &get_param(st, index).is_tonal);
            widgets::ParamDragNumber::new(cx, Data::states, move |st| {
                &get_param(st, index).root_note
            })
            .class("root-note-select")
            .disabled(Data::states.map(move |st| get_param(st, index).is_tonal.value() == false));
        })
        .width(Stretch(0.3));
        widgets::WidgetPanel::new(cx, "Pitch Algorithm", |cx| {
            widgets::ParamRadio::vertical(
                cx,
                Data::states,
                move |st| &get_param(st, index).pitch_shift_kind,
                false,
            );
        })
        .width(Stretch(0.2));
        widgets::WidgetPanel::new(cx, "Blend Group", |cx| {
            widgets::ParamRadio::vertical(
                cx,
                Data::states,
                move |st| &get_param(st, index).blend_group,
                false,
            );
        })
        .width(Stretch(0.2));
        widgets::WidgetPanel::new(cx, "Global Blend Param", |cx| {
            widgets::ParamKnob::new_left_align(cx, Data::states, move |st| &st.params.blend_time);
            widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                &st.params.blend_transition
            });
        })
        .width(Stretch(0.3));
    })
    .col_between(Units::Pixels(PANEL_SPACING))
    .height(Stretch(1.0)); // Equal height distribution
}

fn create_second_panel_row(cx: &mut Context, index: usize) {
    // Second panel row - equal height
    HStack::new(cx, |cx| {
        widgets::WidgetPanel::new(cx, "ADSR", |cx| {
            widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                &get_param(st, index).attack
            });
            widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                &get_param(st, index).decay
            });
            widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                &get_param(st, index).sustain
            });
            widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                &get_param(st, index).release
            });
        })
        .width(Stretch(0.5));
        widgets::WidgetPanel::new(cx, "Time Control", |cx| {
            widgets::ParamKnob::new(
                cx,
                Data::states,
                move |st| &get_param(st, index).start_offset,
                true,
            );
        })
        .width(Stretch(0.25));
        widgets::WidgetPanel::new(cx, "Gain", |cx| {
            widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                &get_param(st, index).gain
            });
        })
        .width(Stretch(0.25));
    })
    .col_between(Units::Pixels(PANEL_SPACING))
    .height(Stretch(1.0)); // Equal height distribution
}

fn create_sample_info_strip(cx: &mut Context, index: usize) {
    // Get the lens of current file
    let file_path = Data::states.map(move |st| {
        get_param(st, index)
            .sample_path
            .read()
            .ok()
            .and_then(|guard| {
                guard
                    .as_ref()
                    .and_then(|path| path.to_str().map(String::from))
            })
    });
    let file_name = Data::states.map(move |st| {
        get_param(st, index)
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
    });

    // The bar for selecting sample ... etc
    HStack::new(cx, |cx| {
        // Button for mute / unmute
        widgets::ParamSwitch::new(cx, Data::states, move |st| &get_param(st, index).muted)
            .width(Stretch(1.0));

        // Sample Name
        Label::new(
            cx,
            file_name.map(|v| v.clone().unwrap_or_else(|| "No sample loaded".into())),
        )
        .width(Stretch(1.0));

        // Btn group
        create_button_group(cx, index, file_path);
    })
    .width(Stretch(1.0))
    .height(Auto)
    .class("widget-panel")
    .class("sample-info-strip");
}

fn create_button_group(
    cx: &mut Context,
    index: usize,
    file_path: impl Lens<Target = Option<String>>,
) {
    HStack::new(cx, |cx| {
        let next_file = file_path.map(|path| {
            path.clone()
                .map(|path| utils::get_next_file_in_directory_wrap(&path))
                .flatten()
        });
        let previous_file = file_path.map(|path| {
            path.clone()
                .map(|path| utils::get_previous_file_in_directory_wrap(&path))
                .flatten()
        });
        Button::new(
            cx,
            move |cx| {
                cx.spawn(move |proxy: &mut ContextProxy| {
                    let path_opt = rfd::FileDialog::new()
                        .add_filter("audio", &["wav"])
                        .pick_file();
                    if let Some(path) = path_opt {
                        // We send a message > load audio
                        let _ = proxy.emit(AppEvent::FileLoading(index, path));
                    }
                });
            },
            |cx| Label::new(cx, "📂"),
        );
        Button::new(
            cx,
            move |cx| {
                if let Some(path) = previous_file.get(cx) {
                    cx.emit(AppEvent::FileLoading(index, path));
                }
            },
            |cx| Label::new(cx, "<"),
        )
        .disabled(previous_file.map(|v| v.is_none()));
        Button::new(
            cx,
            move |cx| {
                if let Some(path) = next_file.get(cx) {
                    cx.emit(AppEvent::FileLoading(index, path));
                }
            },
            |cx| Label::new(cx, ">"),
        )
        .disabled(next_file.map(|v| v.is_none()));
        Button::new(cx, |_| {}, |cx| Label::new(cx, "🗑️"));
    })
    .col_between(Pixels(2.0))
    .height(Auto)
    .child_left(Stretch(1.0))
    .width(Stretch(1.0));
}

fn create_waveform_section(cx: &mut Context, index: usize) {
    // Make a special length for the waveshape
    let binding_lens = Data::states.map(move |st| {
        let param = get_param(st, index);
        (
            param.sample_path.read().ok().and_then(|guard| {
                guard
                    .as_ref()
                    .and_then(|path| path.to_str().map(String::from))
            }),
            param.start_offset.value(),
        )
    });

    // Create a binding so the entire wave isn't always redrawn
    Binding::new(cx, binding_lens, move |cx, new_value| {
        // The display for waves
        VStack::new(cx, |cx| {
            let buffer = Data::states.get(cx).get_buffer_copy(index);
            if let Some(data) = buffer {
                ZStack::new(cx, |cx| {
                    // background canvas
                    Grid::new(
                        cx,
                        ValueScaling::Linear,
                        (-1., 1.),
                        vec![0.],
                        Orientation::Horizontal,
                    );

                    // Waveform canvas
                    // TODO
                    // DO something better here!
                    let num_channel = data.spec.channels as usize;
                    let final_data = data.data.into_iter().step_by(num_channel);
                    let silent = 44100. * new_value.get(cx).1;
                    let silent_vec = vec![0.; silent as usize];
                    widgets::Waveform::new(cx, silent_vec.into_iter().chain(final_data).collect())
                        .outline_width(Pixels(1.0));

                    // Position canvas

                    // Something else ...
                });
            }
        })
        .class("waveform-vizualizer")
        .height(Stretch(1.0));
    });
}

fn create_third_panel_row(cx: &mut Context, index: usize) {
    // Third panel row - equal height
    VStack::new(cx, |cx| {
        create_sample_info_strip(cx, index);
        create_waveform_section(cx, index);
    })
    .row_between(Units::Pixels(PANEL_SPACING))
    .height(Stretch(2.0));
}

fn create_parameter_panels(cx: &mut Context) {
    // Only the parameter panels should be inside the binding
    Binding::new(cx, Data::selected_sample, |cx, selected_idx| {
        let index = selected_idx.get(cx);

        // Wrap all three HStacks in a VStack with a constrained height
        VStack::new(cx, |cx| {
            create_first_panel_row(cx, index);
            create_second_panel_row(cx, index);
            create_third_panel_row(cx, index);
        })
        .row_between(Units::Pixels(PANEL_SPACING))
        .height(Stretch(1.0)); // This VStack should take remaining space
    });
}

struct CssString(String);

impl IntoCssStr for CssString {
    fn get_style(&self) -> Result<String, std::io::Error> {
        Ok(self.0.clone())
    }
}

pub fn create_editor(
    states: Arc<SharedStates>,
    async_executor: AsyncExecutor<HardKickSampler>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(
        ViziaState::new(|| (801, 600)),
        nih_plug_vizia::ViziaTheming::None,
        move |cx, _| {
            let variable_map = css_var_resolver::build_variable_map(THEMES_VAR);
            let css_style =
                css_var_resolver::resolve_css_variables(include_str!("style.css"), &variable_map);
            let css_theme =
                css_var_resolver::resolve_css_variables(include_str!("theme.css"), &variable_map);
            cx.add_stylesheet(CssString(css_style))
                .expect("Coudln't load css file.");
            cx.add_stylesheet(CssString(css_theme))
                .expect("Coudln't load css file.");

            // Build data
            Data {
                states: states.clone(),
                selected_sample: 0,
                executor: async_executor.clone(),
            }
            .build(cx);

            VStack::new(cx, |cx| {
                create_title_section(cx);
                create_sample_tabs(cx);
                create_parameter_panels(cx);
            })
            .child_space(Units::Pixels(MAIN_PADDING))
            .background_color(BACKGROUND_COLOR);
        },
    )
}

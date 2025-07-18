mod customs;
mod events;
mod style;
mod widgets;

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_vizia::vizia::icons::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::RawParamEvent;
use nih_plug_vizia::{create_vizia_editor, ViziaState};

use crate::editor_vizia::events::SetDraggingBlend;
use crate::editor_vizia::widgets::widget_base::ParamWidget;
use crate::params::{SamplePlayerParams, MAX_SAMPLES};
use crate::plugin::HardKickSampler;
use crate::shared_states::SharedStates;
use crate::tasks::{TaskRequests, TaskResults};
use crate::utils;
use style::*;
use widgets::widget_base::*;

pub enum AppEvent {
    SelectSample(usize),
    FileLoading(usize, PathBuf),
    SampleDeleted(usize),
}

#[derive(Lens)]
pub struct Data {
    states: Arc<SharedStates>,
    selected_sample: usize,
    executor: AsyncExecutor<HardKickSampler>,
    is_dragging_blend: bool,
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
                let root = utils::get_root_note_from_filename(
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("")
                        .into(),
                )
                .unwrap_or_default();
                // Get the param
                let param = &get_param(&self.states, self.selected_sample).root_note;
                let ptr = param.as_ptr();
                let normalized = param.preview_normalized(root);
                cx.emit(RawParamEvent::BeginSetParameter(ptr));
                cx.emit(RawParamEvent::SetParameterNormalized(ptr, normalized));
                cx.emit(RawParamEvent::EndSetParameter(ptr));
            }
            AppEvent::SampleDeleted(index) => {
                self.executor
                    .execute_background(TaskRequests::TransfertTask(TaskResults::ClearSample(
                        *index,
                    )));
            }
        });

        event.map(|event: &SetDraggingBlend, meta| {
            self.is_dragging_blend = event.0;
            meta.consume();
        });
    }
}

pub fn get_param(st: &Arc<SharedStates>, index: usize) -> &SamplePlayerParams {
    &st.params.samples[index]
}

fn create_title_section(cx: &mut Context) {
    // Title - this doesn't need to change
    Label::new(cx, "Hard Kick Sampler").class("title");
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
    .col_between(Stretch(1.0))
    .child_bottom(Pixels(PANEL_SPACING))
    .height(Auto);
}

fn create_first_panel_row(cx: &mut Context, index: usize) {
    // First panel row - equal height
    HStack::new(cx, |cx| {
        widgets::WidgetPanel::vnew(cx, "Tonal", |cx| {
            HStack::new(cx, |cx| {
                widgets::ButtonToggle::builder()
                    .with_icon(ICON_WAVE_SAW_TOOL)
                    .build(cx, Data::states, move |st| &get_param(st, index).is_tonal);

                widgets::ParamDragNumber::new(cx, Data::states, move |st| {
                    &get_param(st, index).semitone_offset
                });
            });

            widgets::ParamDragNumber::new(cx, Data::states, move |st| {
                &get_param(st, index).root_note
            })
            .class("root-note-select")
            .disabled(Data::states.map(move |st| !get_param(st, index).is_tonal.value()));
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
            widgets::ParamKnob::builder()
                .on_drag_start(|cx| cx.emit(SetDraggingBlend(true)))
                .on_drag_end(|cx| cx.emit(SetDraggingBlend(false)))
                .build(cx, Data::states, move |st| &st.params.blend_time);
            widgets::ParamKnob::builder()
                .on_drag_start(|cx| cx.emit(SetDraggingBlend(true)))
                .on_drag_end(|cx| cx.emit(SetDraggingBlend(false)))
                .build(cx, Data::states, move |st| &st.params.blend_transition);
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
            widgets::ParamKnob::new(cx, Data::states, move |st| &get_param(st, index).attack);
            widgets::ParamKnob::new(cx, Data::states, move |st| &get_param(st, index).decay);
            widgets::ParamKnob::new(cx, Data::states, move |st| &get_param(st, index).sustain);
            widgets::ParamKnob::new(cx, Data::states, move |st| &get_param(st, index).release);
        })
        .width(Stretch(0.5));
        widgets::WidgetPanel::new(cx, "Time Control", |cx| {
            widgets::ParamKnob::builder()
                .centered()
                .build(cx, Data::states, move |st| {
                    &get_param(st, index).start_offset
                });
        })
        .width(Stretch(0.25));
        widgets::WidgetPanel::new(cx, "Gain", |cx| {
            widgets::ParamKnob::new(cx, Data::states, move |st| &get_param(st, index).gain);
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
        widgets::ButtonToggle::builder()
            .with_icon(ICON_WAVE_SAW_TOOL)
            .build(cx, Data::states, move |st| &get_param(st, index).muted)
            .width(Stretch(1.0))
            .class("mute-toggle");

        // Sample Name
        Label::new(
            cx,
            file_name.map(|v| v.clone().unwrap_or_else(|| "No sample loaded".into())),
        )
        .width(Stretch(1.0))
        .top(Stretch(1.0))
        .bottom(Stretch(1.0));

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
                .and_then(|path| utils::get_next_file_in_directory_wrap(&path))
        });
        let previous_file = file_path.map(|path| {
            path.clone()
                .and_then(|path| utils::get_previous_file_in_directory_wrap(&path))
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
            |cx| Icon::new(cx, ICON_ARROW_BEAR_LEFT),
        )
        .disabled(previous_file.map(|v| v.is_none()));
        Button::new(
            cx,
            move |cx| {
                if let Some(path) = next_file.get(cx) {
                    cx.emit(AppEvent::FileLoading(index, path));
                }
            },
            |cx| Icon::new(cx, ICON_ARROW_BEAR_RIGHT),
        )
        .disabled(next_file.map(|v| v.is_none()));
        Button::new(
            cx,
            move |cx| cx.emit(AppEvent::SampleDeleted(index)),
            |cx| Label::new(cx, "🗑️"),
        )
        .disabled(file_path.map(|file| file.is_none()));
    })
    .col_between(Pixels(2.0))
    .height(Auto)
    .child_left(Stretch(1.0))
    .width(Stretch(1.0));
}

fn create_grid(cx: &mut Context) {
    widgets::StaticGridLines::new(
        cx,
        (1..4).map(|v| v as f32 / 4.).collect(),
        Orientation::Horizontal,
    )
    .class("grid-main");
    widgets::StaticGridLines::new(
        cx,
        (1..3).map(|v| v as f32 / 3.).collect(),
        Orientation::Vertical,
    )
    .class("grid-main");

    widgets::StaticGridLines::new(
        cx,
        (1..5).map(|v| -1. / 8. + v as f32 / 4.).collect(),
        Orientation::Horizontal,
    )
    .class("grid-secondary");
    widgets::StaticGridLines::new(
        cx,
        (1..4).map(|v| -1. / 6. + v as f32 / 3.).collect(),
        Orientation::Vertical,
    )
    .class("grid-secondary");
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
            if let Some(audio_data) = buffer {
                ZStack::new(cx, |cx| {
                    // background canvas
                    create_grid(cx);

                    // First, we have to know how many frame we wanna display
                    let bpm = Data::states.get(cx).host_bpm.load(Ordering::Relaxed);
                    let sr = audio_data.spec.sample_rate as f32;
                    let (_, start_offset) = new_value.get(cx);

                    // calc sum
                    let num_frames = customs::get_num_displayed_frames(1.5, sr, bpm);
                    let num_channels = audio_data.spec.channels as usize;

                    // Waveform canvas
                    // TODO
                    // DO something better here!
                    let final_data = customs::get_waveform(
                        &audio_data.data,
                        num_frames,
                        num_channels,
                        0,
                        start_offset,
                        sr,
                    );

                    // Make waveform
                    let disabled_binding =
                        Data::states.map(move |st| get_param(st, index).muted.value());
                    Binding::new(cx, disabled_binding, move |cx, disabled| {
                        let disabled = disabled.get(cx);
                        widgets::StaticWavePlot::new(cx, final_data.clone())
                            .disabled(disabled)
                            .class("waveform-canvas");
                    });

                    // Time indicator
                    customs::neon_indicator(
                        cx,
                        Data::states.map(move |st| {
                            st.positions[index].load(Ordering::Relaxed) as f32 / num_frames as f32
                        }),
                    );

                    // A Container that has button !
                    HStack::new(cx, |cx| {
                        Icon::new(cx, ICON_123);
                        Icon::new(cx, ICON_123);
                        Icon::new(cx, ICON_123);
                    })
                    .left(Stretch(1.0))
                    .top(Pixels(PANEL_PADDING))
                    .right(Pixels(PANEL_PADDING))
                    .width(Auto)
                    .height(Auto)
                    .visibility(Data::is_dragging_blend)
                    .class("widget-panel");
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
                is_dragging_blend: false,
            }
            .build(cx);

            VStack::new(cx, |cx| {
                create_title_section(cx);
                create_sample_tabs(cx);
                create_parameter_panels(cx);
            })
            .child_space(Pixels(MAIN_PADDING));
        },
    )
}

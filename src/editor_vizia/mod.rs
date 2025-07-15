mod style;
mod widgets;

use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{create_vizia_editor, ViziaState};

// use crate::editor::waveform::WavePlot;
use crate::params::{SamplePlayerParams, MAX_SAMPLES};
use crate::plugin::{HardKickSampler, DEFAULT_BPM};
use crate::shared_states::SharedStates;
use crate::tasks::{AudioData, TaskRequests, TaskResults};
use crate::utils;
use style::*;

pub enum AppEvent {
    SelectSample(usize),
}

#[derive(Lens)]
pub struct Data {
    states: Arc<SharedStates>,
    selected_sample: usize,
}

impl Model for Data {
    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _| match app_event {
            AppEvent::SelectSample(index) => {
                self.selected_sample = *index;
            }
        });
    }
}

pub fn get_param(st: &Arc<SharedStates>, index: usize) -> &SamplePlayerParams {
    &st.params.samples[index]
}

pub fn create_editor(
    states: Arc<SharedStates>,
    _async_executor: AsyncExecutor<HardKickSampler>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(
        ViziaState::new(|| (801, 600)),
        nih_plug_vizia::ViziaTheming::None,
        move |cx, _| {
            cx.add_stylesheet(include_style!("src/editor_vizia/style.css"))
                .expect("Coudln't load css file.");
            cx.add_stylesheet(include_style!("src/editor_vizia/theme.css"))
                .expect("Coudln't load css file.");

            // Build data
            Data {
                states: states.clone(),
                selected_sample: 0,
            }
            .build(cx);

            VStack::new(cx, |cx| {
                // Title - this doesn't need to change
                Label::new(cx, "Hard Kick Sampler")
                    .color(TEXT_COLOR_ACCENT)
                    .class("title");

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

                // Only the parameter panels should be inside the binding
                Binding::new(cx, Data::selected_sample, |cx, selected_idx| {
                    let index = selected_idx.get(cx);

                    // Wrap all three HStacks in a VStack with a constrained height
                    VStack::new(cx, |cx| {
                        // First panel row - equal height
                        HStack::new(cx, |cx| {
                            widgets::WidgetPanel::vnew(cx, "Tonal", |cx| {
                                widgets::ParamDragNumber::new(cx, Data::states, move |st| {
                                    &get_param(st, index).semitone_offset
                                });
                                widgets::ParamSwitch::new(cx, Data::states, move |st| {
                                    &get_param(st, index).is_tonal
                                });
                                widgets::ParamDragNumber::new(cx, Data::states, move |st| {
                                    &get_param(st, index).root_note
                                })
                                .class("root-note-select")
                                .disabled(
                                    Data::states.map(move |st| {
                                        get_param(st, index).is_tonal.value() == false
                                    }),
                                );
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
                                widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                                    &st.params.blend_time
                                });
                                widgets::ParamKnob::new_left_align(cx, Data::states, move |st| {
                                    &st.params.blend_transition
                                });
                            })
                            .width(Stretch(0.3));
                        })
                        .col_between(Units::Pixels(PANEL_SPACING))
                        .height(Stretch(1.0)); // Equal height distribution

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

                        // Third panel row - equal height
                        VStack::new(cx, |cx| {
                            // The bar for selecting sample ... etc
                            HStack::new(cx, |cx| {
                                // Button for mute / unmute
                                widgets::ParamSwitch::new(cx, Data::states, move |st| {
                                    &get_param(st, index).muted
                                })
                                .width(Stretch(1.0));

                                // Sample Name
                                Label::new(
                                    cx,
                                    Data::states.map(move |st| {
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
                                            .unwrap_or_else(|| "No sample loaded".to_string())
                                    }),
                                )
                                .width(Stretch(1.0));

                                // Btn group
                                HStack::new(cx, |cx| {
                                    Button::new(cx, |_| {}, |cx| Label::new(cx, "📂"));
                                    Button::new(cx, |_| {}, |cx| Label::new(cx, "<"));
                                    Button::new(cx, |_| {}, |cx| Label::new(cx, ">"));
                                    Button::new(cx, |_| {}, |cx| Label::new(cx, "🗑️"));
                                })
                                .col_between(Pixels(2.0))
                                .height(Auto)
                                .child_left(Stretch(1.0))
                                .width(Stretch(1.0));
                            })
                            .width(Stretch(1.0))
                            .height(Auto)
                            .class("widget-panel")
                            .bottom(Stretch(1.0))
                            .top(Stretch(1.0));

                            // The display for waves
                            HStack::new(cx, |_| {}).height(Stretch(1.0));
                        })
                        .row_between(Units::Pixels(PANEL_SPACING))
                        .height(Stretch(1.0));
                    })
                    .row_between(Units::Pixels(PANEL_SPACING))
                    .height(Stretch(1.0)); // This VStack should take remaining space
                });
            })
            .child_space(Units::Pixels(MAIN_PADDING))
            .background_color(BACKGROUND_COLOR);
        },
    )
}

mod style;
mod widgets;

use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;
use nih_plug_vizia::widgets::ParamSlider;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState};

// use crate::editor::waveform::WavePlot;
use crate::params::{BlendGroup, HardKickSamplerParams, SamplePlayerParams, MAX_SAMPLES};
use crate::pitch_shift::PitchShiftKind;
use crate::plugin::{HardKickSampler, DEFAULT_BPM};
use crate::shared_states::SharedStates;
use crate::tasks::{AudioData, TaskRequests, TaskResults};
use crate::utils;
use style::*;

#[derive(Lens)]
pub struct Data {
    states: Arc<SharedStates>,
}

impl Model for Data {}

impl Data {
    pub fn params(&self) -> Arc<HardKickSamplerParams> {
        self.states.params.clone()
    }

    pub fn player_params(&self, index: usize) -> &SamplePlayerParams {
        &self.states.params.samples[index]
    }
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

            // Build data
            Data {
                states: states.clone(),
            }
            .build(cx);

            VStack::new(cx, |cx| {
                // Title (and logo later)
                Label::new(cx, "Hard Kick Sampler")
                    .color(TEXT_COLOR_ACCENT)
                    .class("title");

                // Tabs to select samples
                HStack::new(cx, |cx| {
                    for index in 0..MAX_SAMPLES {
                        let txt = format!("Sample {}", index + 1);
                        Button::new(
                            cx,
                            |_| {},
                            |cx| Label::new(cx, &txt).cursor(CursorIcon::Hand),
                        )
                        .hoverable(true)
                        .class("tab");
                    }
                });

                // Pannels
                HStack::new(cx, |cx| {
                    widgets::WidgetPanel::new(cx, "Tonal", |cx| {});
                    widgets::WidgetPanel::new(cx, "Blend Group", |cx| {});
                    widgets::WidgetPanel::new(cx, "Global Blend Param", |cx| {});
                });

                HStack::new(cx, |cx| {
                    widgets::WidgetPanel::new(cx, "ADSR", |cx| {});
                    widgets::WidgetPanel::new(cx, "Time Control", |cx| {});
                    widgets::WidgetPanel::new(cx, "Gain", |cx| {
                        widgets::ParamKnob::new(cx, Data::states, |st| &st.params.gain);
                    });
                });

                HStack::new(cx, |cx| {});
            })
            .background_color(BACKGROUND_COLOR);
        },
    )
}

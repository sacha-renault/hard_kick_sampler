use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use nih_plug::prelude::*;

use crate::utils;

pub const MAX_SAMPLES: usize = 8;

#[derive(Params)]
pub struct SampleWrapperParams {
    #[persist = "sample_path"]
    pub sample_path: Arc<RwLock<Option<PathBuf>>>,

    #[id = "muted"]
    pub muted: BoolParam,

    #[id = "is_tonal"]
    pub is_tonal: BoolParam,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "root_note"]
    pub root_note: IntParam,

    #[id = "semitone_offset"]
    pub semitone_offset: IntParam,

    // ADSR Envelope Parameters
    #[id = "attack"]
    pub attack: FloatParam,

    #[id = "decay"]
    pub decay: FloatParam,

    #[id = "sustain"]
    pub sustain: FloatParam,

    #[id = "release"]
    pub release: FloatParam,
}

impl Default for SampleWrapperParams {
    fn default() -> Self {
        Self {
            sample_path: Arc::new(RwLock::new(None)),

            muted: BoolParam::new("Muted", false).with_value_to_string(Arc::new(|value| {
                if value {
                    String::from("Muted")
                } else {
                    String::from("Mute")
                }
            })),

            is_tonal: BoolParam::new("Tonal", true),

            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            root_note: IntParam::new("Root Note", 0, IntRange::Linear { min: 0, max: 11 })
                .with_value_to_string(Arc::new(utils::semitones_to_note)),

            semitone_offset: IntParam::new(
                "Semitone Offset",
                0,
                IntRange::Linear { min: -24, max: 24 },
            )
            .with_unit(" semitones"),

            // ADSR Parameters
            attack: FloatParam::new(
                "Attack",
                0.0, // 0ms default! I want the kick to go brrrrr
                FloatRange::Skewed {
                    min: 0.,                               // 0ms minimum
                    max: 5.0,                              // 5s maximum
                    factor: FloatRange::skew_factor(-2.0), // Exponential curve
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            decay: FloatParam::new(
                "Decay",
                0.1, // 100ms default
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            sustain: FloatParam::new(
                "Sustain",
                1.0, // default to 100%, we don't want the sample to be modified unless the user specify it
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            release: FloatParam::new(
                "Release",
                0.010, // 10ms default to short release to avoid end clic
                FloatRange::Skewed {
                    min: 0.001,
                    max: 9.999, // Longer release times are useful
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
        }
    }
}

#[derive(Params)]
pub struct HardKickSamplerParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,

    #[nested(array, group = "Samples")]
    pub samples: [SampleWrapperParams; MAX_SAMPLES],
}

impl Default for HardKickSamplerParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            samples: [(); MAX_SAMPLES].map(|_| SampleWrapperParams::default()),
        }
    }
}

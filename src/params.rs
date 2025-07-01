use std::sync::{Arc, RwLock};

use nih_plug::prelude::*;

pub const MAX_SAMPLES: usize = 8;

#[derive(Params)]
pub struct SampleWrapperParams {
    #[persist = "sample_path"]
    pub sample_path: Arc<RwLock<Option<String>>>,

    #[id = "muted"]
    pub muted: BoolParam,

    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for SampleWrapperParams {
    fn default() -> Self {
        Self {
            sample_path: Arc::new(RwLock::new(None)),
            muted: BoolParam::new("Muted", false),
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            ),
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

mod adsr;
mod editor;
mod params;
mod plugin;
mod sample_wrapper;
mod shared_states;
mod tasks;
mod utils;

use nih_plug::prelude::*;
use plugin::HardKickSampler;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

impl ClapPlugin for HardKickSampler {
    const CLAP_ID: &'static str = "com.your-domain.hard-kick-sampler";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Sample kick that goes brrrrr");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
        ClapFeature::Sampler,
    ];
}

nih_export_clap!(HardKickSampler);

impl Vst3Plugin for HardKickSampler {
    const VST3_CLASS_ID: [u8; 16] = *b"HardKickSampler!";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
        Vst3SubCategory::Drum,
        Vst3SubCategory::Sampler,
    ];
}

// We will not export vst3 right away, i have to figure out
// what the GPL license implies
nih_export_vst3!(HardKickSampler);

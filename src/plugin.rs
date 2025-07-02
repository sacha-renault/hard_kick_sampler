use nih_plug::prelude::*;
use std::num::NonZero;
use std::sync::Arc;

use crate::params::{HardKickSamplerParams, MAX_SAMPLES};
use crate::sample_wrapper::SampleWrapper;

pub struct HardKickSampler {
    // Params of the plugin
    params: Arc<HardKickSamplerParams>,

    // Sample wrapper
    samples: Vec<SampleWrapper>,
}

impl Default for HardKickSampler {
    fn default() -> Self {
        let params = Arc::new(HardKickSamplerParams::default());
        let samples = (0..MAX_SAMPLES)
            .map(|index| SampleWrapper::new(params.clone(), index))
            .collect();
        Self {
            params: params.clone(),
            samples: samples,
        }
    }
}

impl HardKickSampler {
    /// Process midi events
    fn handle_context(&mut self, context: &mut impl ProcessContext<Self>) {
        // Process MIDI events
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    // Trigger a sample
                    self.start_sample(note, velocity);
                }
                NoteEvent::NoteOff { .. } => {
                    // Stop a sample
                    self.stop_sample();
                }
                _ => {}
            }
        }
    }

    /// Trigger the samples to play for all the ones that are loaded
    fn start_sample(&mut self, note: u8, velocity: f32) {
        for sample in self.samples.iter_mut() {
            sample.start_playing(note, velocity);
        }
    }

    /// Just stop playing, we don't have to specify the notes
    /// because we don't handle multi notes playing in the same
    /// time anyway
    fn stop_sample(&mut self) {
        for sample in self.samples.iter_mut() {
            sample.stop_playing();
        }
    }
}

impl Plugin for HardKickSampler {
    const NAME: &'static str = "Hard Kick Sampler";
    const VENDOR: &'static str = "Sacha RENAULT";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "contact@sacharenault.ovh";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Get number of channel
        let num_channel = audio_io_layout
            .main_output_channels
            .unwrap_or(const { NonZero::new(2).unwrap() })
            .get();

        for sample_wrapper in self.samples.iter_mut() {
            sample_wrapper.change_sample_rate_output(buffer_config.sample_rate);
            sample_wrapper.change_channel_number(num_channel as usize);
        }

        // Load some random shit samples
        let _ = self.samples[0].load_audio_file(r"C:\Program Files\Image-Line\FL Studio 21\Data\Patches\Packs\Custom pack\OPS\OPS - Euphoric Hardstyle Kick Expansion (Vol. 1)\Kick Build Folder\Punches\OPS_ECHKE1_PUNCH_5.wav");
        let _ = self.samples[1].load_audio_file(r"C:\Program Files\Image-Line\FL Studio 21\Data\Patches\Packs\Custom pack\OPS\OPS - Euphoric Hardstyle Kick Expansion (Vol. 1)\Kick Build Folder\Crunches\OPS_ECHKE1_CRUNCH_12_G.wav");
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Handle the context
        self.handle_context(context);

        // Audio processing
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();

            // We initialize a value to false to say to the sample wrapper that this might be or not
            // The first channel. What's will happens basically is that on the first sample
            // sample wrapper calls smoother.next() and all the subsequant calls will be made
            // On previous_value() so that it doesn't call next `channel_number` time per sample
            let mut channel_index = 0;

            for sample in channel_samples {
                *sample = 0.;

                // each sample provide its next value
                // Sum all playing samples
                for sample_wrapper in &mut self.samples {
                    *sample += sample_wrapper.next(channel_index);
                }

                // apply gain
                *sample *= gain;

                // set first channel to false
                channel_index += 1;
            }
        }

        ProcessStatus::Normal
    }
}

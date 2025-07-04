use nih_plug::prelude::*;
use std::num::NonZero;
use std::sync::Arc;

use crate::editor::create_editor;
use crate::params::{HardKickSamplerParams, MAX_SAMPLES};
use crate::sample_wrapper::SampleWrapper;
use crate::shared_states::SharedStates;
use crate::tasks::{TaskRequests, TaskResults};
use crate::utils;

pub struct HardKickSampler {
    // Params of the plugin
    params: Arc<HardKickSamplerParams>,

    // Sample wrapper
    sample_wrappers: Vec<SampleWrapper>,

    // The task receiver
    receiver: Option<std::sync::mpsc::Receiver<TaskResults>>,
}

impl Default for HardKickSampler {
    fn default() -> Self {
        let params = Arc::new(HardKickSamplerParams::default());
        let sample_wrappers = (0..MAX_SAMPLES)
            .map(|index| SampleWrapper::new(params.clone(), index))
            .collect();
        Self {
            params: params.clone(),
            sample_wrappers,
            receiver: None,
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
        for sample in self.sample_wrappers.iter_mut() {
            sample.start_playing(note, velocity);
        }
    }

    /// Just stop playing, we don't have to specify the notes
    /// because we don't handle multi notes playing in the same
    /// time anyway
    fn stop_sample(&mut self) {
        for sample in self.sample_wrappers.iter_mut() {
            sample.stop_playing();
        }
    }

    fn handle_messages(&mut self) {
        // Get the receiver
        let receiver = match &self.receiver {
            Some(receiver) => receiver,
            None => return,
        };

        // Handle events
        while let Ok(task) = receiver.try_recv() {
            match task {
                TaskResults::LoadedFile(index, path, data) => {
                    self.sample_wrappers
                        .get_mut(index)
                        .map(|sample| sample.load_and_set_audio_file(&path, data));
                }
                TaskResults::ClearSample(index) => {
                    self.sample_wrappers
                        .get_mut(index)
                        .map(|sample| sample.clear_sample());
                }
            };
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
    type BackgroundTask = TaskRequests;

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

        // init a bool that knows if everything went well
        let mut success = true;

        for (index, sample_wrapper) in self.sample_wrappers.iter_mut().enumerate() {
            sample_wrapper.cleanup_wrapper();
            sample_wrapper.change_sample_rate_output(buffer_config.sample_rate);
            sample_wrapper.change_channel_number(num_channel as usize);

            // Load the file that is saved in the preset!
            if let Err(e) = sample_wrapper.load_preset_sample() {
                nih_error!("Failed to load sample for wrapper {}: {}", index, e);
                success = false;
            }
        }
        success
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
        for sample_wrapper in self.sample_wrappers.iter_mut() {
            sample_wrapper.reset();
        }
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Handle messages
        self.handle_messages();

        // Handle the context
        self.handle_context(context);

        // Audio processing
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();

            for (channel_index, sample) in channel_samples.into_iter().enumerate() {
                *sample = 0.;

                // each sample provide its next value
                // Sum all playing samples
                for sample_wrapper in &mut self.sample_wrappers {
                    *sample += sample_wrapper.next(channel_index);
                }

                // apply gain
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let state = SharedStates {
            params: self.params.clone(),
            wave_readers: self
                .sample_wrappers
                .iter()
                .map(|s| s.get_wave_reader())
                .collect(),
        };
        create_editor(state, async_executor)
    }

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        let (sender, receiver) = std::sync::mpsc::channel();
        self.receiver = Some(receiver);

        Box::new(move |task| match task {
            TaskRequests::TransfertTask(task) => {
                // Actually load the file
                let _ = sender.send(task);
            }
            TaskRequests::LoadFile(index, path) => {
                // Actually load the file
                if let Ok(audio_data) = utils::load_audio_file(&path) {
                    let _ = sender.send(TaskResults::LoadedFile(index, path, audio_data));
                }
            }
            TaskRequests::OpenFilePicker(index) => {
                let path_opt = rfd::FileDialog::new()
                    .add_filter("audio", &["wav"])
                    .pick_file();
                if let Some(path) = path_opt {
                    if let Ok(audio_data) = utils::load_audio_file(&path) {
                        let _ = sender.send(TaskResults::LoadedFile(index, path, audio_data));
                    }
                }
            }
            TaskRequests::LoadNextFile(index, current_path) => {
                if let Some(file) = utils::get_next_file_in_directory_wrap(&current_path) {
                    if let Ok(audio_data) = utils::load_audio_file(&file) {
                        let _ = sender.send(TaskResults::LoadedFile(index, file, audio_data));
                    }
                }
            }
            TaskRequests::LoadPreviousFile(index, current_path) => {
                if let Some(file) = utils::get_previous_file_in_directory_wrap(&current_path) {
                    if let Ok(audio_data) = utils::load_audio_file(&file) {
                        let _ = sender.send(TaskResults::LoadedFile(index, file, audio_data));
                    }
                }
            }
        })
    }
}

use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use nih_plug::buffer::Buffer;
use nih_plug::nih_error;

use crate::adsr::MultiChannelAdsr;
use crate::params::{HardKickSamplerParams, SamplePlayerParams};
use crate::pitch_shift::PitchShiftKind;
use crate::tasks::AudioData;
use crate::utils::{self, SharedAudioData};

/// MIDI note number for middle C (C3), used as the base note for pitch calculations
const BASE_NOTE: u8 = 60;

/// Default sample rate used for initialization
const DEFAULT_SAMPLE_RATE: f32 = 48000.;

/// Number of semitone in one octave
const SEMITONE_PER_OCTAVE: f32 = 12.;

/// A multi-channel audio sample player with pitch shifting, ADSR envelope, and real-time parameter control.
///
/// `SampleWrapper` handles loading and playback of audio samples with support for:
/// - Multi-channel audio (mono, stereo, surround)
/// - Real-time pitch shifting based on MIDI notes
/// - ADSR envelope shaping
/// - Smooth parameter interpolation
/// - Sample rate conversion
pub struct SamplePlayer {
    /// A ref to the params
    params: Arc<HardKickSamplerParams>,

    /// Known which index it is
    index: usize,

    /// Holds the values of the sample
    buffer: Option<Vec<f32>>,

    /// The target sample rate (i.e. the sample rate of the host)
    host_sample_rate: f32,

    /// Sample rate of the sample itself, not the process sr
    sample_rate: f32,

    /// Current trigerred note
    midi_note: Option<i8>,

    /// Number of output channels
    num_channel: usize,

    /// The adsr envelope
    adsr: MultiChannelAdsr,

    // HERE ARE THE DATA THAT ARE SHARED WITH THE GUI
    /// A copy of the buffer that the GUI can access for display
    shared_buffer: Arc<RwLock<SharedAudioData>>,

    /// A copy of the current position in the sample
    shared_playback_position: Arc<AtomicU64>,
}

impl SamplePlayer {
    /// Returns a reference to the parameters specific to this sample wrapper.
    ///
    /// This is a convenience method to access the sample-specific parameters
    /// from the shared parameter structure.
    fn get_params(&self) -> &SamplePlayerParams {
        &self.params.samples[self.index]
    }

    /// Creates a new sample wrapper for the given parameter index.
    ///
    /// # Arguments
    ///
    /// * `params` - Shared reference to the sampler's parameter structure
    /// * `index` - Index of this sample in the parameters array
    ///
    /// # Panics
    ///
    /// Panics if the index is greater than or equal to the number of available sample slots.
    pub fn new(params: Arc<HardKickSamplerParams>, index: usize) -> Self {
        // Ensure the index is not out of range
        assert!(
            index < params.samples.len(),
            "Sample index {} is out of bounds (max: {})",
            index,
            params.samples.len()
        );

        Self {
            params,
            index,
            buffer: None,
            sample_rate: 0.,
            host_sample_rate: DEFAULT_SAMPLE_RATE,
            midi_note: None,
            num_channel: 0,
            adsr: MultiChannelAdsr::new(DEFAULT_SAMPLE_RATE),

            // THINGS FOR GUI
            shared_buffer: Arc::new(RwLock::new(SharedAudioData::default())),
            shared_playback_position: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Starts playing the sample with the specified MIDI note.
    ///
    /// The note number is used for pitch calculation if the sample is set to tonal mode.
    /// Only triggers if a sample buffer is loaded.
    ///
    /// # Arguments
    ///
    /// * `note` - MIDI note number (0-127, where 60 is middle C)
    /// * `_velocity` - Note velocity (0.0-1.0, currently unused)
    pub fn start_playing(&mut self, note: u8, _velocity: f32) {
        // Only trigger if we have a buffer loaded
        if self.buffer.is_some() {
            // Calculate semitone difference from base note
            let semitone_offset = note as i8 - BASE_NOTE as i8;

            // Set the note that is currently playing
            self.midi_note = Some(semitone_offset);

            // Trigger the adsr
            self.adsr.note_on();
        }
    }

    /// Stops the sample playback by triggering the ADSR release phase.
    ///
    /// The sample will continue playing through its release envelope
    /// before becoming silent.
    pub fn stop_playing(&mut self) {
        self.adsr.note_off();
    }

    /// Updates the target sample rate for proper pitch calculation.
    ///
    /// This should be called when the host sample rate changes.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - New target sample rate in Hz
    pub fn change_sample_rate_output(&mut self, sample_rate: f32) {
        self.host_sample_rate = sample_rate;
        self.adsr.set_sample_rate(sample_rate);
    }

    /// Sets the number of output channels for proper buffer indexing.
    ///
    /// # Arguments
    ///
    /// * `num_channel` - Number of output channels (1=mono, 2=stereo, etc.)
    pub fn change_channel_number(&mut self, num_channel: usize) {
        self.num_channel = num_channel;
    }

    /// Loads an audio file and sets it as the current sample.
    ///
    /// The file path is stored in the parameters for preset saving/loading.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the audio file to load
    /// * `audio_data` - data of the loaded audio
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the file was loaded successfully
    /// * `Err(...)` if there was an error loading the file or setting the path
    pub fn load_and_set_audio_file(
        &mut self,
        file_path: &Path,
        audio_data: AudioData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Set the buffer and sample rate
        self.sample_rate = audio_data.spec.sample_rate as f32;
        self.write_two_buffers(Some(audio_data.data));

        // If buffer is loaded, we set the sample path
        match self.get_params().sample_path.write() {
            Ok(mut path) => *path = Some(file_path.into()),
            _ => return Err("Couldn't set the file path".into()),
        };

        // Load buffer;
        Ok(())
    }

    pub fn clear_sample(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Reset data
        self.write_two_buffers(None);
        self.sample_rate = 0.;
        self.adsr.reset();
        match self.get_params().sample_path.write() {
            Ok(mut path) => *path = None,
            _ => return Err("Couldn't set the file path".into()),
        };

        Ok(())
    }

    /// Loads the sample from the stored file path in parameters.
    ///
    /// This is used when loading presets or restoring the sampler state.
    /// If no path is stored, this method does nothing.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the sample was loaded successfully or no path was stored
    /// * `Err(...)` if there was an error loading the file
    pub fn load_preset_sample(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get file path
        let file_path = match self.get_params().sample_path.read() {
            Ok(path_guard) => match path_guard.as_ref() {
                Some(path) => path.clone(),
                None => return Ok(()),
            },
            Err(_) => return Err("Error fetching file path".into()),
        };

        // load audio
        let audio = utils::load_audio_file(&file_path)?;

        // Set the buffer and sample rate
        self.sample_rate = audio.spec.sample_rate as f32;
        self.write_two_buffers(Some(audio.data));
        Ok(())
    }

    /// Calculates the current playback rate based on pitch shifting parameters.
    ///
    /// The playback rate is calculated using the formula: `2^(semitone_offset / 12)`
    /// where semitone_offset combines:
    /// - Semitone offset parameter (user fine tuning adjustment)
    /// - MIDI note offset from root note (if tonal mode is enabled)
    ///
    /// If `is_tonal` parameter is false, MIDI note AND the root note has no influence on pitch.
    ///
    /// Returns a multiplier where 1.0 = original speed, 2.0 = double speed (octave up),
    /// 0.5 = half speed (octave down).
    #[inline]
    pub fn get_playback_rate(&self) -> f32 {
        // Cache params
        let params = self.get_params();

        // Parameter offset (user tuning adjustment)
        let param_note_offset = params.semitone_offset.value() as f32;

        // MIDI note offset from root note
        let midi_note_offset = if params.is_tonal.value() {
            let midi_offset = self.midi_note.unwrap_or(0) as f32;
            let root_note = params.root_note.value() as f32;
            midi_offset - root_note
        } else {
            0.
        };

        let final_offset = param_note_offset + midi_note_offset;
        2.0_f32.powf(final_offset / SEMITONE_PER_OCTAVE)
    }

    /// Get the pitch-adjusted playback position for a specific audio channel.
    ///
    /// Applies the current playback rate to the raw playback position and calculates
    /// the appropriate sample index for interleaved audio data.
    ///
    /// # Returns
    ///
    /// A tuple of (sample_index, fractional_part) where:
    /// - `sample_index` is the integer sample position in the interleaved buffer
    /// - `fractional_part` is the sub-sample position for interpolation (0.0 to 1.0)
    #[inline]
    pub fn get_playback_position(&self, process_count: f32, channel_index: usize) -> (usize, f32) {
        let raw_playback_position = process_count * self.get_sr_correction();
        let pitched_position = self.get_playback_rate() * raw_playback_position;

        let frame_index = pitched_position as usize;
        let fraction = pitched_position.fract();
        let sample_index = frame_index * self.num_channel + channel_index;

        (sample_index, fraction)
    }

    /// Returns the sample rate correction factor.
    ///
    /// This accounts for differences between the sample's original sample rate
    /// and the host's sample rate to maintain proper playback timing.
    #[inline]
    pub fn get_sr_correction(&self) -> f32 {
        self.sample_rate / self.host_sample_rate
    }

    /// Completely resets and clears the sample wrapper.
    ///
    /// This removes the loaded sample buffer and resets all playback state.
    /// Use this when changing samples or cleaning up resources.
    pub fn cleanup_wrapper(&mut self) {
        // Clear sample data
        self.write_two_buffers(None);

        // Reset playback state
        self.midi_note = None;

        // Reset ADSR envelope
        self.adsr.reset();
    }

    /// Resets the playback state without clearing the loaded sample.
    ///
    /// This resets the ADSR envelope, clears the current note, and
    /// returns the playback position to the beginning.
    pub fn reset(&mut self) {
        self.adsr.reset();
        self.midi_note = None;
    }

    /// Returns whether this sample is currently muted.
    ///
    /// # Returns
    ///
    /// `true` if the sample is muted, `false` otherwise
    #[inline]
    pub fn is_muted(&self) -> bool {
        self.get_params().muted.value()
    }

    /// Returns whether this sample should produce silence.
    ///
    /// This is a convenience method that combines all conditions that would
    /// result in no audio output. A sample is considered silent if:
    /// - The ADSR envelope is in idle state (not playing)
    /// - The sample is muted via parameters
    /// - No audio buffer is loaded
    #[inline]
    pub fn is_silent(&self) -> bool {
        self.adsr.is_idling() || self.is_muted() || self.buffer.is_none()
    }

    #[inline]
    pub fn get_buffer_if_not_silent(&self) -> Option<&Vec<f32>> {
        if self.is_silent() {
            None
        } else {
            self.buffer.as_ref()
        }
    }

    /// Generates the next audio sample for the specified channel.
    ///
    /// This is the main audio processing method that should be called once per channel
    /// per audio frame. It handles sample interpolation, ADSR envelope application,
    /// and parameter smoothing.
    ///
    /// # Arguments
    ///
    /// * `process_count` - The number of frames processed by the plugin from the start of the note
    /// This value must be corrected if sr of the sample != from the sample of the host.
    /// * `channel_index` - Which channel to generate (0 for left, 1 for right, etc.)
    ///
    /// # Returns
    ///
    /// The audio sample value for this channel, or 0.0 if:
    /// - The sample is muted
    /// - No sample is loaded
    /// - The ADSR envelope is idle
    /// - Playback has reached the end of the sample
    ///
    /// # Performance Notes
    ///
    /// - Parameter loading and playback position updates only occur on channel 0
    /// - This prevents parameter drift and maintains channel synchronization
    /// - Uses linear interpolation for smooth playback at non-integer positions
    #[inline]
    pub fn next(&mut self, process_count: f32, channel_index: usize) -> f32 {
        // Check if we should play first
        let buffer = match self.get_buffer_if_not_silent() {
            Some(buffer) => buffer,
            None => return 0.0,
        };

        // check if it's the first channel of the frame
        // to be processed
        let is_first_channel = channel_index == 0;

        // Cache the params
        let params = self.get_params();

        // Get the sample_index
        let num_frames_delay = params.delay_start.value() * self.sample_rate;
        let (sample_index, fraction) = self.get_playback_position(process_count, channel_index);

        // Might have early return if current value is < 0
        let final_sample_index = match utils::clipping_sub(
            sample_index,
            (num_frames_delay * self.num_channel as f32) as usize,
        ) {
            Some(v) => v,
            None => return 0.,
        };

        // Get current and next frame
        let current_frame = buffer.get(final_sample_index);
        let next_frame = buffer.get(final_sample_index + self.num_channel);

        // depending on the availability of current and next frame, we apply different processing
        let sample_value = match (current_frame, next_frame) {
            // Case were current value and next value are both defined
            // We can interpolate
            (Some(value), Some(value_next)) => utils::interpolate(*value, *value_next, fraction),

            // Current value is define but next is out of range
            (Some(value), None) => *value,

            // Nothing defined, we reset note + adsr
            _ => {
                // Case we are out of bounds
                self.midi_note = None;
                self.adsr.reset();
                return 0.0;
            }
        };

        // Load parameter
        let gain = utils::load_smooth_param(&params.gain.smoothed, is_first_channel);

        // We don't want those param to be any smoothed!
        let attack = params.attack.value();
        let decay = params.decay.value();
        let sustain = params.sustain.value();
        let release = params.release.value();

        // Get the adrs value
        let adrs_envelope = self
            .adsr
            .next_value(attack, decay, sustain, release, is_first_channel);

        sample_value * gain * adrs_envelope
    }

    pub fn process(&mut self, buffer: &mut Buffer, process_count: f32) {
        match self.get_params().pitch_shift_kind.value() {
            PitchShiftKind::Classic => self.process_classic(buffer, process_count),
        }
    }

    fn process_classic(&mut self, buffer: &mut Buffer, process_count: f32) {
        for (count, frame) in buffer
            .iter_samples()
            .enumerate()
            .map(|(i, sample)| (i as f32 + process_count, sample))
        {
            for (channel_index, sample) in frame.into_iter().enumerate() {
                *sample += self.next(count, channel_index);
            }
        }
    }

    /// Writes audio data to both internal and shared buffers.
    ///
    /// The internal buffer write always succeeds and is critical for audio processing.
    /// If the shared buffer write fails (GUI-related), audio processing continues
    /// uninterrupted while the GUI may show stale waveform data.
    #[inline]
    fn write_two_buffers(&mut self, data: Option<Vec<f32>>) {
        self.buffer = data.clone();
        match self.shared_buffer.write() {
            Ok(mut buff) => {
                *buff = SharedAudioData::new(data.unwrap_or_default(), self.sample_rate)
            }
            Err(_) => nih_error!("Couldn't write ..."),
        }
    }

    pub fn get_shared_audio_data(&self) -> Arc<RwLock<SharedAudioData>> {
        self.shared_buffer.clone()
    }

    pub fn get_shared_position(&self) -> Arc<AtomicU64> {
        self.shared_playback_position.clone()
    }

    #[inline]
    pub fn update_shared_position(&mut self, process_count: f32) {
        if !self.is_silent() {
            let raw_playback_position = process_count * self.get_sr_correction();
            let pitched_position = self.get_playback_rate() * raw_playback_position;
            self.shared_playback_position
                .store(pitched_position as u64, Ordering::Relaxed);
        } else {
            self.shared_playback_position.store(0, Ordering::Relaxed);
        }
    }
}

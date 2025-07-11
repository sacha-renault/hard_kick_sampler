use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use nih_plug::buffer::Buffer;
use nih_plug::nih_error;

use crate::adsr::MultiChannelAdsr;
use crate::params::{HardKickSamplerParams, SamplePlayerParams};
use crate::pitch_shift::PitchShiftKind;
use crate::tasks::AudioData;
use crate::utils;

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
    host_channels: usize,

    /// Number of channel of the sample
    sample_channels: usize,

    /// The adsr envelope
    adsr: MultiChannelAdsr,

    // HERE ARE THE DATA THAT ARE SHARED WITH THE GUI
    /// A copy of the buffer that the GUI can access for display
    shared_buffer: Arc<RwLock<Option<AudioData>>>,

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
            host_channels: 0,
            sample_channels: 0,
            adsr: MultiChannelAdsr::new(DEFAULT_SAMPLE_RATE),

            // THINGS FOR GUI
            shared_buffer: Arc::new(RwLock::new(None)),
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
        self.host_channels = num_channel;
    }

    /// Sets the sample file path in the parameters.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to set, or None to clear
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful
    /// * `Err(...)` if the parameter write lock couldn't be acquired
    fn set_sample_path(&self, file_path: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
        let mut path_guard = self
            .get_params()
            .sample_path
            .write()
            .map_err(|_| "Failed to acquire write lock on sample path")?;

        *path_guard = file_path.map(|p| p.to_path_buf());
        Ok(())
    }

    /// Updates both internal and shared audio buffers with new data.
    ///
    /// This method handles updating the internal buffer for audio processing
    /// and the shared buffer for GUI display. If updating the shared buffer fails,
    /// audio processing continues uninterrupted.
    ///
    /// # Arguments
    ///
    /// * `audio_data` - New audio data to set, or None to clear buffers
    fn update_buffers(&mut self, audio_data: Option<AudioData>) {
        // Update internal buffer and metadata
        self.buffer = audio_data.as_ref().map(|data| data.data.clone());
        self.sample_channels = audio_data
            .as_ref()
            .map(|data| data.spec.channels as usize)
            .unwrap_or(0);

        // Update sample rate if we have audio data
        if let Some(ref data) = audio_data {
            self.sample_rate = data.spec.sample_rate as f32;
        }

        // Update shared buffer for GUI (non-critical operation)
        if let Ok(mut shared_guard) = self.shared_buffer.write() {
            *shared_guard = audio_data;
        } else {
            nih_error!("Failed to update shared buffer for GUI - audio processing continues");
        }
    }

    /// Loads an audio file and sets it as the current sample.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the audio file to load
    /// * `audio_data` - Loaded audio data
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful
    /// * `Err(...)` if there was an error setting the file path
    pub fn load_and_set_audio_file(
        &mut self,
        file_path: &Path,
        audio_data: AudioData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Update buffers with new audio data
        self.update_buffers(Some(audio_data));

        // Set the file path in parameters
        self.set_sample_path(Some(file_path))?;

        Ok(())
    }

    /// Clears the current sample and resets the player state.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful
    /// * `Err(...)` if there was an error clearing the file path
    pub fn clear_sample(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear buffers and reset state
        self.update_buffers(None);
        self.sample_rate = 0.;
        self.adsr.reset();

        // Clear the file path
        self.set_sample_path(None)?;

        Ok(())
    }

    /// Loads the sample from the stored file path in parameters.
    ///
    /// This is used when loading presets or restoring the sampler state.
    /// If no path is stored, this method does nothing.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if successful or no path was stored
    /// * `Err(...)` if there was an error loading the file
    pub fn load_preset_sample(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Get the stored file path
        let file_path = {
            let path_guard = self
                .get_params()
                .sample_path
                .read()
                .map_err(|_| "Failed to acquire read lock on sample path")?;

            match path_guard.as_ref() {
                Some(path) => path.clone(),
                None => return Ok(()), // No path stored, nothing to load
            }
        };

        // Load and set the audio data
        let audio_data = utils::load_audio_file(&file_path)?;
        self.update_buffers(Some(audio_data));

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
        utils::get_stretch_playback_position(
            process_count,
            self.get_sr_correction(),
            self.get_playback_rate(),
            self.sample_channels,
            channel_index,
        )
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
        self.update_buffers(None);

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
    ///    this value must be corrected if sr of the sample != from the sample of the host.
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
        let (sample_index, fraction) = self.get_playback_position(process_count, channel_index);
        let offset = -params.start_offset.value() * self.sample_rate * self.sample_channels as f32;

        // Depending on offset, we add or sub
        let final_sample_index = if offset > 0. {
            match utils::clipping_sub(sample_index, offset as usize) {
                Some(v) => v,
                None => return 0.,
            }
        } else {
            (sample_index as f32 - offset) as usize
        };

        // Get current and next frame
        let current_frame = buffer.get(final_sample_index);
        let next_frame = buffer.get(final_sample_index + self.host_channels);

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

        // Get the blend value
        let group = params.blend_group.value();
        let blend_time = self.params.blend_time.value();
        let blend_transition = self.params.blend_transition.value();
        let current_time = process_count / self.host_sample_rate;
        let blend_gain = utils::get_blend_value(group, current_time, blend_time, blend_transition);

        // Get the adrs value
        let adrs_envelope = self
            .adsr
            .next_value(attack, decay, sustain, release, is_first_channel);

        sample_value * gain * adrs_envelope * blend_gain
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

    pub fn get_shared_audio_data(&self) -> Arc<RwLock<Option<AudioData>>> {
        self.shared_buffer.clone()
    }

    pub fn get_shared_position(&self) -> Arc<AtomicU64> {
        self.shared_playback_position.clone()
    }

    #[inline]
    pub fn update_shared_position(&mut self, process_count: f32) {
        // If sample is silent, position is 0
        if self.is_silent() {
            self.shared_playback_position.store(0, Ordering::Relaxed);
            return;
        }

        // Else we can store a value
        let raw_playback_position = process_count * self.get_sr_correction();
        let pitched_position = self.get_playback_rate() * raw_playback_position;
        self.shared_playback_position
            .store(pitched_position as u64, Ordering::Relaxed);
    }
}

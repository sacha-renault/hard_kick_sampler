use std::path::Path;
use std::sync::Arc;

use crate::adsr::MultiChannelAdsr;
use crate::params::{HardKickSamplerParams, SampleWrapperParams};
use crate::tasks::AudioData;
use crate::utils;

/// MIDI note number for middle C (C3), used as the base note for pitch calculations
const BASE_NOTE: u8 = 72;

/// Default sample rate used for initialization
const DEFAULT_SAMPLE_RATE: f32 = 44100.;

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
pub struct SampleWrapper {
    /// A ref to the params
    params: Arc<HardKickSamplerParams>,

    /// Known which index it is
    index: usize,

    /// Holds the values of the sample
    buffer: Option<Vec<f32>>,

    /// The target sample rate (i.e. the sample rate of the host)
    target_sample_rate: f32,

    /// Sample rate of the sample itself, not the process sr
    sample_rate: u32,

    /// Save where we are in the sample
    playback_position: f32,

    /// Current trigerred note
    midi_note: Option<i8>,

    /// Number of output channels
    num_channel: usize,

    /// The adsr envelope
    adsr: MultiChannelAdsr,
}

impl SampleWrapper {
    /// Returns a reference to the parameters specific to this sample wrapper.
    ///
    /// This is a convenience method to access the sample-specific parameters
    /// from the shared parameter structure.
    fn get_params(&self) -> &SampleWrapperParams {
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
            params.samples.len() >= index,
            "Index of the sample is more than the maximum"
        );
        Self {
            params,
            index,
            buffer: None,
            playback_position: 0.,
            sample_rate: 0,
            target_sample_rate: DEFAULT_SAMPLE_RATE,
            midi_note: None,
            num_channel: 0,
            adsr: MultiChannelAdsr::new(DEFAULT_SAMPLE_RATE),
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

            // Reset playback position to start of sample
            self.playback_position = 0.0;

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
        self.target_sample_rate = sample_rate;
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
        self.sample_rate = audio_data.spec.sample_rate;
        self.buffer = Some(audio_data.data);

        // If buffer is loaded, we set the sample path
        match self.get_params().sample_path.write() {
            Ok(mut path) => *path = Some(file_path.into()),
            _ => return Err("Couldn't set the file path".into()),
        };

        // Load buffer;
        Ok(())
    }

    pub fn clear_sample(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.buffer = None;
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
        self.sample_rate = audio.spec.sample_rate;
        self.buffer = Some(audio.data);
        Ok(())
    }

    /// Advances the playback position based on pitch shifting parameters.
    ///
    /// The playback rate is calculated using the formula: `2^(semitone_offset / 12)`
    /// where semitone_offset combines:
    /// - MIDI note offset from C3 (if tonal mode is enabled)
    /// - Root note parameter (original pitch of the sample)
    /// - Semitone offset parameter (additional fine tuning)
    ///
    /// If `is_tonal` parameter is false, MIDI note has no influence on pitch.
    #[inline]
    pub fn increment_playback_position(&mut self) {
        // Parameter offset (user tuning adjustment)
        let param_note_offset = self.get_params().semitone_offset.value() as f32;

        // MIDI note offset from root note
        let midi_note_offset = if self.get_params().is_tonal.value() {
            self.midi_note.unwrap_or(0) as f32
        } else {
            0.
        };

        // Get root note
        let root_note = self.get_params().root_note.value() as f32;

        let final_offset = midi_note_offset + param_note_offset - root_note;
        let playback_rate = 2.0_f32.powf(final_offset / SEMITONE_PER_OCTAVE);
        self.playback_position +=
            playback_rate * (self.sample_rate as f32 / self.target_sample_rate);
    }

    /// Completely resets and clears the sample wrapper.
    ///
    /// This removes the loaded sample buffer and resets all playback state.
    /// Use this when changing samples or cleaning up resources.
    pub fn cleanup_wrapper(&mut self) {
        // Clear sample data
        self.buffer = None;

        // Reset playback state
        self.playback_position = 0.0;
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
        self.playback_position = 0.;
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

    /// Generates the next audio sample for the specified channel.
    ///
    /// This is the main audio processing method that should be called once per channel
    /// per audio frame. It handles sample interpolation, ADSR envelope application,
    /// and parameter smoothing.
    ///
    /// # Arguments
    ///
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
    pub fn next(&mut self, channel_index: usize) -> f32 {
        // Check if we should play first
        if self.is_silent() {
            return 0.0;
        }

        // check if it's the first channel of the frame
        // to be processed
        let is_first_channel = channel_index == 0;

        if let Some(buffer) = self.buffer.as_ref() {
            // Get the sample_index
            let sample_index = self.playback_position as usize * self.num_channel + channel_index;

            // depending on if it'the value is Some or None
            let sample_value = match (
                buffer.get(sample_index),
                buffer.get(sample_index + self.num_channel),
            ) {
                // Case were current value and next value are both defined
                // We can interpolate
                (Some(value), Some(value_next)) => {
                    let fraction = self.playback_position.fract();
                    utils::interpolate(*value, *value_next, fraction)
                }

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

            // Update playback position only on the first channel of the frame
            if is_first_channel {
                self.increment_playback_position();
            }

            // Load parameter
            let gain = utils::load_smooth_param(&self.get_params().gain.smoothed, is_first_channel);

            // We don't want those param to be any smoothed!
            let attack = self.get_params().attack.value();
            let decay = self.get_params().decay.value();
            let sustain = self.get_params().sustain.value();
            let release = self.get_params().release.value();

            // Get the adrs value
            let adrs_enveloppe =
                self.adsr
                    .next_value(attack, decay, sustain, release, is_first_channel);

            sample_value * gain * adrs_enveloppe
        } else {
            0.0
        }
    }
}

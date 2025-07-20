pub mod classic;
pub mod psola;

use nih_plug::prelude::Enum;

#[derive(Debug, PartialEq, Enum)]
pub enum PitchShiftKind {
    Classic,
    Psola,
}

#[derive(Debug, PartialEq)]
pub enum FrameOutput {
    Mono(f32),
    Stereo([f32; 2]),
    Unsupported,
}

impl From<Vec<f32>> for FrameOutput {
    fn from(value: Vec<f32>) -> Self {
        if value.len() == 1 {
            FrameOutput::Mono(value[0])
        } else if value.len() == 2 {
            FrameOutput::Stereo([value[0], value[1]])
        } else {
            FrameOutput::Unsupported
        }
    }
}

/// A trait for audio pitch shifting implementations.
///
/// This trait defines the interface for pitch shifting algorithms that can load audio samples,
/// apply pitch and speed modifications, and generate output frames on demand.
///
/// ## Typical Usage Flow
///
/// 1. **Load**: Call `load_sample()` when a new audio sample is available (e.g., at VST startup or sample change)
/// 2. **Trigger**: Call `trigger()` when a note is played or playback begins
/// 3. **Stream**: Call `get_frame()` repeatedly while the note is held or until the sample ends
/// 4. **Clear**: Call `clear_sample()` when cleaning up or switching samples
pub trait PitchShifter {
    /// Clears the loaded sample and resets the shifter to an unloaded state.
    ///
    /// This method should:
    /// - Free any allocated memory for the current sample
    /// - Reset internal state to the initial condition
    /// - Ensure `ready()` returns `false` after clearing
    ///
    /// Safe to call multiple times or when no sample is loaded.
    fn clear_sample(&mut self);

    /// Loads an audio sample for pitch shifting.
    ///
    /// # Parameters
    ///
    /// * `sample_buffer` - Interleaved audio data (e.g., [L, R, L, R, ...] for stereo)
    /// * `channel_number` - Number of audio channels (1 for mono, 2 for stereo, etc.)
    /// * `sample_rate` - Sample rate of the input audio in Hz
    ///
    /// # Behavior
    ///
    /// - Analyzes the input audio for pitch detection and preprocessing
    /// - May fail silently if pitch detection fails (implementation-dependent)
    /// - After successful loading, the shifter is ready to be triggered
    /// - Calling this method replaces any previously loaded sample
    ///
    /// # Notes
    ///
    /// The sample buffer contains interleaved audio data. For stereo audio:
    /// - `sample_buffer[0]` = first left channel sample
    /// - `sample_buffer[1]` = first right channel sample  
    /// - `sample_buffer[2]` = second left channel sample
    /// - And so on...
    fn load_sample(&mut self, sample_buffer: &[f32], channel_number: usize, sample_rate: f32);

    /// Triggers playback with specified pitch and timing parameters.
    ///
    /// # Parameters
    ///
    /// * `sr_correction` - Sample rate correction factor (original_rate / host_rate)
    /// * `get_semitone_offset` - Number of semitone to shift
    ///
    /// # Sample Rate Correction
    ///
    /// The `sr_correction` parameter accounts for differences between the original sample's
    /// rate and the host's sample rate:
    ///
    /// ```text
    /// sr_correction = sample_rate / host_sample_rate
    ///
    /// Examples:
    /// - Sample at 44.1kHz, host at 48kHz: 44100 / 48000 = 0.91875
    /// - Sample at 48kHz, host at 44.1kHz: 48000 / 44100 = 1.08844
    /// ```
    ///
    /// # Playback Rate
    ///
    /// The `get_semitone_offset` parameter controls pitch shifting:
    ///
    /// ```text
    /// playback_rate = 2^(semitones / 12)
    ///
    /// Examples:
    /// - Up 1 semitone: 2^(1/12) ≈ 1.0595
    /// - Down 1 octave: 2^(-12/12) = 0.5
    /// - Up 1 octave: 2^(12/12) = 2.0
    /// ```
    ///
    /// # Requirements
    ///
    /// - A sample must be loaded before calling this method
    /// - After triggering, `ready()` should return `true` if successful
    /// - Can be called multiple times to retrigger with different parameters
    fn trigger(&mut self, sr_correction: f32, get_semitone_offset: f32);

    /// Returns whether the shifter is ready to generate output frames.
    ///
    /// # Returns
    ///
    /// `true` if:
    /// - A sample has been successfully loaded
    /// - `trigger()` has been called successfully
    /// - The shifter is ready to process `get_frame()` calls
    ///
    /// `false` if:
    /// - No sample is loaded
    /// - `trigger()` hasn't been called yet
    /// - The shifter is in an error state
    fn ready(&self) -> bool;

    /// Retrieves an audio frame at the specified position.
    ///
    /// # Parameters
    ///
    /// * `position` - Sample index position (0.0 = first sample, 1.0 = second sample, etc.)
    ///
    /// # Returns
    ///
    /// * `Some(Vec<f32>)` - Audio frame with one value per channel [ch1, ch2, ...]
    /// * `None` - If position is out of bounds or shifter is not ready
    ///
    /// # Position Indexing
    ///
    /// The position parameter uses sample-based indexing:
    /// - `0.0` returns the first output sample
    /// - `1.0` returns the second output sample
    /// - `1000.5` would interpolate between samples 1000 and 1001 (implementation-dependent)
    ///
    /// # Frame Format
    ///
    /// The returned vector contains one sample per channel:
    /// - Mono: `[sample]`
    /// - Stereo: `[left, right]`
    /// - 5.1: `[L, R, C, LFE, Ls, Rs]`
    fn get_frame(&mut self, position: f32) -> Option<FrameOutput>;

    /// Returns the type/algorithm used by this pitch shifter.
    fn kind(&self) -> PitchShiftKind;

    /// Return the position in frame number of the pitch shifter
    /// since sample started to play
    fn get_position(&self, position: f32) -> f32;
}

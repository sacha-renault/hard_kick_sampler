use std::sync::Arc;

use crate::params::{HardKickSamplerParams, SampleWrapperParams};
use crate::utils;

// C4 is MIDI note 60 (base note)
const BASE_NOTE: u8 = 60;

pub struct SampleWrapper {
    /// A ref to the params
    params: Arc<HardKickSamplerParams>,

    /// Known which index it is
    index: usize,

    /// Holds the values of the sample
    buffer: Option<Vec<f32>>,

    /// The target sample rate (i.e. the sample rate of the output buffer)
    target_sample_rate: f32,

    /// Sample rate of the sample itself, not the process sr
    sample_rate: u32,

    /// Save where we are in the sample
    playback_position: f32,

    /// Current trigerred note
    note_offset: Option<i8>,

    /// Number of output channels
    num_channel: usize,
}

impl SampleWrapper {
    /// Helper function to get the params of this sample
    fn get_params(&self) -> &SampleWrapperParams {
        &self.params.samples[self.index]
    }

    pub fn new(params: Arc<HardKickSamplerParams>, index: usize) -> Self {
        // Ensure the index is not oor
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
            target_sample_rate: 0.,
            note_offset: None,
            num_channel: 0,
        }
    }

    pub fn start_playing(&mut self, note: u8, _velocity: f32) {
        // Only trigger if we have a buffer loaded
        if self.buffer.is_some() {
            // Calculate semitone difference from base note
            let semitone_offset = note as i8 - BASE_NOTE as i8;

            // Set the note that is currently playing
            self.note_offset = Some(semitone_offset);

            // Reset playback position to start of sample
            self.playback_position = 0.0;
        }
    }

    pub fn stop_playing(&mut self) {
        self.playback_position = 0.0;
        self.note_offset = None;
    }

    pub fn change_sample_rate_output(&mut self, sample_rate: f32) {
        self.target_sample_rate = sample_rate;
    }

    pub fn change_channel_number(&mut self, num_channel: usize) {
        self.num_channel = num_channel;
    }

    pub fn load_audio_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // load audio
        let audio = utils::load_audio_file(file_path)?;

        // Set the buffer and sample rate
        self.sample_rate = audio.0;
        self.buffer = Some(audio.1);

        // If buffer is loaded, we set the sample path
        match self.get_params().sample_path.write() {
            Ok(mut path) => *path = Some(String::from(file_path)),
            _ => return Err("Couldn't set the file path".into()),
        };

        // Load buffer;
        Ok(())
    }

    /// Increment the playback position based on:
    /// - Sample rate
    /// - The note the user is playing
    /// - The `param_note_offset` which is a combination of root note selected for the sample + semitone set in param
    /// If the param `is_tonal` isn't set to true, midi note has no influence
    /// TODO
    /// Double check this is okay with interleaved stereo sample
    /// For me, looks like there will be crosstalk between channel and that could be horrible
    pub fn increment_playback_position(&mut self) {
        let param_note_offset =
            self.get_params().semitone_offset.value() + self.get_params().root_note.value();
        let midi_note_offset = if self.get_params().is_tonal.value() {
            self.note_offset.unwrap_or(0)
        } else {
            0
        };
        let playback_rate =
            2.0_f32.powf((midi_note_offset + param_note_offset as i8) as f32 / 12.0);
        self.playback_position +=
            playback_rate * (self.target_sample_rate / self.sample_rate as f32);
    }

    /// Reset entirely the sample wrapper
    pub fn reset(&mut self) {
        let params = self.params.clone();
        let target_sample_rate = self.target_sample_rate;
        *self = Self::new(params, self.index);
        self.target_sample_rate = target_sample_rate;
    }

    pub fn is_muted(&self) -> bool {
        self.get_params().muted.value()
    }

    pub fn next(&mut self, channel_index: usize) -> f32 {
        // Check if we should play first
        if self.note_offset.is_none() || self.is_muted() {
            return 0.0;
        }

        // check if it's the first channel of the frame
        // to be processed
        let is_first_channel = channel_index == 0;

        if let Some(buffer) = self.buffer.as_ref() {
            // Get the sample_index
            let sample_index = self.playback_position as usize * self.num_channel + channel_index;

            // Check bounds before accessing
            if sample_index >= buffer.len() {
                self.note_offset = None;
                return 0.0;
            }

            // Get the sample value
            // TODO
            // Might wanna double check this function and take an additional parameter channels
            // To know how many channels are expected ...
            // let sample_value = utils::get_sample_at(&buffer, self.playback_position, channel_index);
            let sample_value = buffer[sample_index];

            // Load parameter
            let gain = utils::load_smooth_param(&self.get_params().gain.smoothed, is_first_channel);

            // Update playback position only on the first channel of the frame
            if is_first_channel {
                self.increment_playback_position();
            }

            sample_value * gain
        } else {
            0.0
        }
    }
}

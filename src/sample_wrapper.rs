use std::sync::Arc;

use nih_plug::nih_log;

use crate::params::{HardKickSamplerParams, SampleWrapperParams};
use crate::utils;

pub struct SampleWrapper {
    /// A ref to the params
    params: Arc<HardKickSamplerParams>,

    /// Known which index it is
    index: usize,

    /// Holds the values of the sample
    buffer: Option<Vec<f32>>,

    /// Sample rate of the sample itself, not the process sr
    sample_rate: u32,

    /// Know how fast we have to go throught the sample
    /// Depends on the pitch
    playback_rate: f32,

    /// Save where we are in the sample
    playback_position: f32,
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
            playback_rate: 1.,
            playback_position: 0.,
            sample_rate: 0,
        }
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

    pub fn reset(&mut self) {
        // Clear the buffer & put the sample path to none
        self.buffer = None;
        self.sample_rate = 0;
        match self.get_params().sample_path.write() {
            Ok(mut path) => *path = None,
            Err(_) => {} // Do nothing rn, maybe error handling there later ?
        }
    }

    pub fn is_muted(&self) -> bool {
        self.get_params().muted.value()
    }

    pub fn get_buffer_if_playing(&mut self) -> Option<&mut Vec<f32>> {
        if self.is_muted() {
            None
        } else if let Some(buf) = self.buffer.as_mut() {
            if self.playback_position >= buf.len() as f32 {
                None
            } else {
                Some(buf)
            }
        } else {
            None
        }
    }

    pub fn next(&mut self, is_first_channel: bool) -> f32 {
        // early exit is its muted
        if let Some(buffer) = self.get_buffer_if_playing() {
            let gain = utils::load_smooth_param(&self.get_params().gain.smoothed, is_first_channel);
            todo!()
        } else {
            0.
        }
    }
}

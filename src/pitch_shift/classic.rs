use crate::{
    pitch_shift::{PitchShiftKind, PitchShifter},
    utils,
};

pub struct ClassicShifter {
    sample_buffer: Option<Vec<f32>>,
    channel_number: usize,
    sample_rate: f32,
    playback_rate: f32,
    sr_correction: f32,
    is_loaded: bool,
}

impl ClassicShifter {
    pub fn new() -> Self {
        Self {
            sample_buffer: None,
            channel_number: 0,
            sample_rate: 0.0,
            playback_rate: 1.0,
            sr_correction: 1.0,
            is_loaded: false,
        }
    }

    /// Get the pitch-adjusted playback position for a specific audio channel.
    /// Uses the same utility function as the main SamplePlayer for consistency.
    fn get_playback_position(&self, process_count: f32, channel_index: usize) -> (usize, f32) {
        utils::get_stretch_playback_position(
            process_count,
            self.sr_correction,
            self.playback_rate,
            self.channel_number,
            channel_index,
        )
    }

    /// Linear interpolation between two samples
    fn interpolate(current: f32, next: f32, fraction: f32) -> f32 {
        current + (next - current) * fraction
    }
}

impl PitchShifter for ClassicShifter {
    fn clear_sample(&mut self) {
        self.sample_buffer = None;
        self.channel_number = 0;
        self.sample_rate = 0.0;
        self.playback_rate = 1.0;
        self.sr_correction = 1.0;
        self.is_loaded = false;
    }

    fn load_sample(&mut self, sample_buffer: &[f32], channel_number: usize, sample_rate: f32) {
        self.sample_buffer = Some(sample_buffer.to_vec());
        self.channel_number = channel_number;
        self.sample_rate = sample_rate;
        self.is_loaded = true;
    }

    fn trigger(&mut self, sr_correction: f32, playback_rate: f32) {
        self.sr_correction = sr_correction;
        self.playback_rate = playback_rate;
    }

    fn ready(&self) -> bool {
        self.is_loaded && self.sample_buffer.is_some()
    }

    fn get_frame(&mut self, position: f32) -> Option<Vec<f32>> {
        let buffer = self.sample_buffer.as_ref()?;

        let mut frame = Vec::with_capacity(self.channel_number);

        for channel_index in 0..self.channel_number {
            // Use the existing get_playback_position method which handles all the pitch shifting logic
            let (sample_index, fraction) = self.get_playback_position(position, channel_index);

            // Get current and next sample for interpolation
            let current_sample = buffer.get(sample_index);
            let next_sample = buffer.get(sample_index + self.channel_number);

            let sample_value = match (current_sample, next_sample) {
                // Both samples available - interpolate
                (Some(&current), Some(&next)) => Self::interpolate(current, next, fraction),
                // Only current sample available
                (Some(&current), None) => current,
                // No samples available - end of buffer
                _ => return None,
            };

            frame.push(sample_value);
        }

        Some(frame)
    }

    fn kind(&self) -> PitchShiftKind {
        PitchShiftKind::Classic
    }

    fn get_position(&self, position: f32) -> f32 {
        self.sr_correction * position * self.playback_rate
    }
}

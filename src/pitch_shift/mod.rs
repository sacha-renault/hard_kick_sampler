pub mod classic;
pub mod psola;

use nih_plug::prelude::Enum;

#[derive(Debug, PartialEq, Enum)]
pub enum PitchShiftKind {
    Classic,
    Psola,
}

pub trait PitchShifter {
    fn clear_sample(&mut self);
    fn load_sample(&mut self, sample_buffer: &[f32], channel_number: usize, sample_rate: f32);
    fn trigger(&mut self, sr_correction: f32, playback_rate: f32);
    fn ready(&self) -> bool;
    fn get_frame(&mut self, position: f32) -> Option<Vec<f32>>;
    fn kind(&self) -> PitchShiftKind;
}

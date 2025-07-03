use std::path::PathBuf;

use derive_more::{Constructor, From, Into};
use hound::WavSpec;

#[derive(Debug, Constructor, Into, From)]
pub struct AudioData {
    pub spec: WavSpec,
    pub data: Vec<f32>,
}

#[derive(Debug)]
pub enum TaskIn {
    LoadAudioFile(usize, PathBuf),
}

pub enum TaskOut {
    LoadedFile(usize, PathBuf, AudioData),
}

use std::path::PathBuf;

use derive_more::{Constructor, From, Into};
use hound::WavSpec;

#[derive(Debug, Constructor, Into, From, Clone)]
pub struct AudioData {
    pub spec: WavSpec,
    pub data: Vec<f32>,
}

#[derive(Debug)]
pub enum TaskResults {
    LoadedFile(usize, PathBuf, AudioData),
    ClearSample(usize),
}

#[derive(Debug)]
pub enum TaskRequests {
    TransfertTask(TaskResults),
    LoadFile(usize, PathBuf),
}

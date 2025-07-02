use std::path::PathBuf;

use derive_more::Constructor;

#[derive(Debug, Constructor)]
pub struct FileWithIndex {
    pub path: PathBuf,
    pub index: usize,
}

#[derive(Debug, Constructor)]
pub struct LoadedFileWithIndex {
    pub path: PathBuf,
    pub index: usize,
    pub data: Vec<f32>,
    pub spec: f32,
}

#[derive(Debug)]
pub enum TaskIn {
    LoadAudioFile(FileWithIndex),
}

pub enum TaskOut {
    LoadedFile(LoadedFileWithIndex),
}

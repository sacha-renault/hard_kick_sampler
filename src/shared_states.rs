use nih_plug::prelude::*;
use std::sync::{atomic::AtomicU64, Arc, RwLock};

use crate::{params::HardKickSamplerParams, tasks::AudioData, utils::SharedAudioData};

#[derive(Debug)]
pub struct SharedStates {
    // pub playback_positions: [AtomicI32; MAX_SAMPLES],
    /// Add some triple buffer so the gui can read the
    /// wave that is loaded in the processor
    pub shared_buffer: Vec<Arc<RwLock<Option<AudioData>>>>,

    /// The params of the processor
    pub params: Arc<HardKickSamplerParams>,

    /// The position in each buffer
    pub positions: Vec<Arc<AtomicU64>>,

    /// The tempo of the host
    pub host_bpm: Arc<AtomicF32>,
}

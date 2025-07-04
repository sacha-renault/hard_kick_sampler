// use std::sync::atomic::AtomicI32;
use std::sync::{Arc, RwLock};

use crate::params::{HardKickSamplerParams, MAX_SAMPLES};

#[derive(Debug)]
pub struct SharedStates {
    // pub playback_positions: [AtomicI32; MAX_SAMPLES],
    /// Add some triple buffer so the gui can read the
    /// wave that is loaded in the processor
    pub wave_readers: Vec<Arc<RwLock<Vec<f32>>>>,

    /// The params of the processor
    pub params: Arc<HardKickSamplerParams>,
}

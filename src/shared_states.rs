// use std::sync::atomic::AtomicI32;
use std::sync::{atomic::AtomicU64, Arc, RwLock};

use crate::params::HardKickSamplerParams;

#[derive(Debug)]
pub struct SharedStates {
    // pub playback_positions: [AtomicI32; MAX_SAMPLES],
    /// Add some triple buffer so the gui can read the
    /// wave that is loaded in the processor
    pub shared_buffer: Vec<Arc<RwLock<Vec<f32>>>>,

    /// The params of the processor
    pub params: Arc<HardKickSamplerParams>,

    /// The position in each buffer
    pub positions: Vec<Arc<AtomicU64>>,
}

use nih_plug::nih_error;
use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use tdpsola::{AlternatingHann, Speed, TdpsolaAnalysis, TdpsolaSynthesis};

use crate::{
    pitch_shift::{FrameOutput, PitchShiftKind, PitchShifter},
    utils,
};

const POWER_THRESHOLD: f32 = 5.0;
const CLARITY_THRESHOLD: f32 = 0.1;

pub struct PsolaShifter {
    _hanns: Vec<AlternatingHann>,
    analysis: Vec<TdpsolaAnalysis>,
    synthesis: Option<Vec<TdpsolaSynthesis>>,
    iter_samples: Option<Vec<Vec<f32>>>,
    source_length: f32,
    is_loaded: bool,
    sr_correction: f32,
    playback_rate: f32,
}

impl PsolaShifter {
    pub fn new() -> Self {
        Self {
            _hanns: Vec::new(),
            analysis: Vec::new(),
            synthesis: None,
            iter_samples: None,
            source_length: 0.0,
            is_loaded: false,
            sr_correction: 1.0,
            playback_rate: 1.0,
        }
    }

    fn build_internal(
        &mut self,
        sample_buffer: &[f32],
        channel_number: usize,
        sample_rate: f32,
    ) -> bool {
        let scratch_size = sample_buffer.len() * 2;
        let single_channel = sample_buffer
            .iter()
            .step_by(channel_number)
            .copied()
            .collect::<Vec<f32>>();

        let mut detector = McLeodDetector::new(single_channel.len(), scratch_size);

        if let Some(pitch) = detector.get_pitch(
            &single_channel,
            sample_rate as usize,
            POWER_THRESHOLD,
            CLARITY_THRESHOLD,
        ) {
            nih_plug::nih_log!("Detected frequency {}", pitch.frequency);
            let source_wavelength = sample_rate / pitch.frequency;
            let padding_length = source_wavelength as usize + 1;

            let mut hanns: Vec<AlternatingHann> = (0..channel_number)
                .map(|_| AlternatingHann::new(source_wavelength))
                .collect();

            let mut analysis = hanns.iter().map(TdpsolaAnalysis::new).collect::<Vec<_>>();

            for (channel, (analys, hann)) in analysis.iter_mut().zip(hanns.iter_mut()).enumerate() {
                for _ in 0..padding_length {
                    analys.push_sample(0.0, hann);
                }
                for sample in sample_buffer.iter().skip(channel).step_by(channel_number) {
                    analys.push_sample(*sample, hann);
                }
            }

            self._hanns = hanns;
            self.analysis = analysis;
            self.synthesis = None;
            self.iter_samples = None;
            self.source_length = source_wavelength;
            self.is_loaded = true;

            true
        } else {
            // Pitch detection failed
            self.clear_sample();
            nih_error!("Error: couldn't detect pitch");
            false
        }
    }
}

impl PitchShifter for PsolaShifter {
    fn clear_sample(&mut self) {
        self._hanns.clear();
        self.analysis.clear();
        self.synthesis = None;
        self.iter_samples = None;
        self.source_length = 0.0;
        self.is_loaded = false;
    }

    fn load_sample(&mut self, sample_buffer: &[f32], channel_number: usize, sample_rate: f32) {
        if !self.build_internal(sample_buffer, channel_number, sample_rate) {
            nih_error!("Error while setting up pitch shifter {:?}", self.kind());
        }
    }

    fn trigger(&mut self, sr_correction: f32, semitone_offset: f32) {
        if !self.is_loaded {
            return;
        }

        self.playback_rate = utils::semitone_offset_to_playback_rate(semitone_offset);
        self.sr_correction = sr_correction;

        // Create NEW synthesis objects each time - analysis stays intact!
        let mut synthesis: Vec<TdpsolaSynthesis> = (0..self._hanns.len())
            .map(|_| {
                TdpsolaSynthesis::new(
                    Speed::from_f32(sr_correction),
                    self.source_length / self.playback_rate,
                )
            })
            .collect();

        // Now you can use the same analysis with new synthesis
        self.iter_samples = Some(
            synthesis
                .iter_mut()
                .zip(self.analysis.iter())
                .map(|(s, a)| s.iter(a).skip(self.source_length as usize + 1).collect())
                .collect(),
        );
        self.synthesis = Some(synthesis);
    }

    fn ready(&self) -> bool {
        self.is_loaded && self.iter_samples.is_some()
    }

    fn get_frame(&mut self, position: f32) -> Option<FrameOutput> {
        self.iter_samples
            .as_ref()?
            .iter()
            .map(|channels| {
                channels
                    .get((position * self.sr_correction) as usize)
                    .copied()
            })
            .collect::<Option<Vec<_>>>()
            .map(|v| v.into())
    }

    fn kind(&self) -> PitchShiftKind {
        PitchShiftKind::Psola
    }

    fn get_position(&self, position: f32) -> f32 {
        self.sr_correction * position
    }
}

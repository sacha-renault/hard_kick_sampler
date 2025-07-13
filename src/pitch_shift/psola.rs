use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use tdpsola::{AlternatingHann, Speed, TdpsolaAnalysis, TdpsolaSynthesis};

const POWER_THRESHOLD: f32 = 5.0;
const CLARITY_THRESHOLD: f32 = 0.1;

pub struct PsolaShifter {
    _hanns: Vec<AlternatingHann>,
    analysis: Vec<TdpsolaAnalysis>,
    synthesis: Option<Vec<TdpsolaSynthesis>>,
    iter_samples: Option<Vec<Vec<f32>>>,
    source_length: f32,
}

impl PsolaShifter {
    pub fn build(sample_buffer: &[f32], channel_number: usize, sample_rate: usize) -> Option<Self> {
        let scratch_size = sample_buffer.len() * 2;
        let single_channel = sample_buffer
            .iter()
            .step_by(channel_number)
            .copied()
            .collect::<Vec<f32>>();
        let mut detector = McLeodDetector::new(single_channel.len(), scratch_size);
        let pitch = detector.get_pitch(
            &single_channel,
            sample_rate,
            POWER_THRESHOLD,
            CLARITY_THRESHOLD,
        )?;
        let source_wavelength = sample_rate as f32 / pitch.frequency;

        let mut hanns: Vec<AlternatingHann> = (0..channel_number)
            .map(|_| AlternatingHann::new(source_wavelength))
            .collect();
        let mut analysis = hanns
            .iter()
            .map(|hann| TdpsolaAnalysis::new(hann))
            .collect::<Vec<_>>();

        for (channel, (analys, hann)) in analysis.iter_mut().zip(hanns.iter_mut()).enumerate() {
            for sample in sample_buffer.iter().skip(channel).step_by(channel_number) {
                analys.push_sample(*sample, hann);
            }
        }

        Some(Self {
            _hanns: hanns,
            analysis: analysis,
            synthesis: None,
            iter_samples: None,
            source_length: source_wavelength,
        })
    }

    pub fn trigger(&mut self, sr_correction: f32, playback_rate: f32) {
        // Create NEW synthesis objects each time - analysis stays intact!
        let mut synthesis: Vec<TdpsolaSynthesis> = (0..self._hanns.len())
            .map(|_| {
                TdpsolaSynthesis::new(
                    Speed::from_f32(sr_correction),
                    self.source_length / playback_rate,
                )
            })
            .collect();

        // Now you can use the same analysis with new synthesis
        self.iter_samples = Some(
            synthesis
                .iter_mut()
                .zip(self.analysis.iter())
                .map(|(s, a)| s.iter(a).collect())
                .collect(),
        );
        self.synthesis = Some(synthesis);
    }

    pub fn get_frame(&mut self, position: usize) -> Option<Vec<f32>> {
        self.iter_samples
            .as_ref()?
            .iter()
            .map(|channels| channels.get(position).copied())
            .collect()
    }
}

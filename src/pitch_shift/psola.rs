use pitch_detection::detector::mcleod::McLeodDetector;
use pitch_detection::detector::PitchDetector;
use tdpsola::{AlternatingHann, Speed, TdpsolaAnalysis, TdpsolaSynthesis};

const POWER_THRESHOLD: f32 = 5.0;
const CLARITY_THRESHOLD: f32 = 0.7;

pub struct PsolaShifter {
    _hanns: Vec<AlternatingHann>,
    analysis: Vec<TdpsolaAnalysis>,
    synthesis: Option<Vec<TdpsolaSynthesis>>,
    iter_samples: Option<Vec<Vec<f32>>>,
}

impl PsolaShifter {
    pub fn build(sample_buffer: &[f32], channel_number: usize, sample_rate: usize) -> Option<Self> {
        let scratch_size = sample_buffer.len() * 4;
        let mut detector = McLeodDetector::new(sample_buffer.len(), scratch_size);
        let pitch = detector.get_pitch(
            sample_buffer,
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
        })
    }

    pub fn trigger(&mut self, target_wavelength: f32) {
        // Create NEW synthesis objects each time - analysis stays intact!
        let mut synthesis: Vec<TdpsolaSynthesis> = (0..self._hanns.len())
            .map(|_| {
                let mut s = TdpsolaSynthesis::new(Speed::from_f32(1.0), target_wavelength);
                s.set_wavelength(target_wavelength);
                s
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

    pub fn next(&mut self, position: usize) -> Option<Vec<&f32>> {
        self.iter_samples
            .as_ref()?
            .iter()
            .map(|channels| channels.get(position))
            .collect()
    }
}

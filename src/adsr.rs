/// Represents the current stage of the ADSR envelope
#[derive(Debug)]
enum AdsrStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// A multi-channel ADSR (Attack, Decay, Sustain, Release) envelope generator.
///
/// This implementation allows multiple audio channels to share the same envelope state,
/// with only the first channel advancing the envelope timing on each sample.
///
/// # ADSR Envelope Stages
///
/// 1. **Attack**: When a note starts, the envelope rises from 0 to 1 over the attack time
/// 2. **Decay**: The envelope falls from 1 to the sustain level over the decay time  
/// 3. **Sustain**: The envelope holds at the sustain level until note release
/// 4. **Release**: When the note ends, the envelope falls from sustain level to 0
///
/// ## Generate envelope values for stereo audio (2 channels)
/// let left = adsr.next_value(0.1, 0.2, 0.7, 0.5, true);   // First channel advances
/// let right = adsr.next_value(0.1, 0.2, 0.7, 0.5, false); // Second channel gets same value
pub struct MultiChannelAdsr {
    /// Current sample rate
    sample_rate: f32,

    /// progress in the current stage
    stage_progress: f32,

    /// Current value
    current_value: f32,

    /// current state
    stage: AdsrStage,
}

impl MultiChannelAdsr {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            stage: AdsrStage::Idle,
            current_value: 0.0,
            stage_progress: 0.0,
            sample_rate,
        }
    }

    /// Triggers the start of a note, beginning the attack phase.
    ///
    /// This immediately transitions the envelope to the attack stage and resets
    /// the stage progress to 0. Can be called at any time to restart the envelope.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut adsr = MultiChannelAdsr::new(44100.0);
    /// adsr.note_on();
    /// assert!(!adsr.note_finished());
    /// ```
    pub fn note_on(&mut self) {
        self.stage = AdsrStage::Attack;
        self.stage_progress = 0.0;
    }

    /// Triggers the end of a note, beginning the release phase.
    ///
    /// If the envelope is already idle, this has no effect. Otherwise, it immediately
    /// transitions to the release stage and resets the stage progress to 0.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut adsr = MultiChannelAdsr::new(44100.0);
    /// adsr.note_on();
    /// // ... generate some samples ...
    /// adsr.note_off(); // Begin release phase
    /// ```
    pub fn note_off(&mut self) {
        if !matches!(self.stage, AdsrStage::Idle) {
            self.stage = AdsrStage::Release;
            self.stage_progress = 0.0;
        }
    }

    /// Updates the sample rate.
    ///
    /// This affects the timing of all subsequent envelope stages. Changing the sample
    /// rate mid-envelope may cause timing discontinuities.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Returns whether the envelope has finished and returned to idle state.
    ///
    /// This is useful for voice management - when `true`, the voice can be
    /// deallocated or reused for a new note.
    pub fn is_idling(&self) -> bool {
        return matches!(self.stage, AdsrStage::Idle);
    }

    pub fn is_playing(&self) -> bool {
        return !matches!(self.stage, AdsrStage::Idle);
    }

    /// Internal method that advances the envelope by one sample and returns the current value.
    fn next(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) -> f32 {
        match &self.stage {
            AdsrStage::Idle => {
                self.current_value = 0.0;
            }
            AdsrStage::Attack => {
                let attack_samples = attack * self.sample_rate;
                if self.stage_progress >= attack_samples {
                    self.current_value = 1.0;
                    self.stage = if decay > 0. {
                        AdsrStage::Decay
                    } else {
                        AdsrStage::Sustain
                    };
                    self.stage_progress = 0.0;
                } else {
                    self.current_value = self.stage_progress / attack_samples;
                    self.stage_progress += 1.0;
                }
            }

            AdsrStage::Decay => {
                let decay_samples = decay * self.sample_rate;
                if self.stage_progress >= decay_samples {
                    self.current_value = sustain;
                    self.stage = AdsrStage::Sustain;
                    self.stage_progress = 0.0;
                } else {
                    let progress = self.stage_progress / decay_samples;
                    self.current_value = 1.0 - progress * (1.0 - sustain);
                    self.stage_progress += 1.0;
                }
            }

            AdsrStage::Sustain => {
                self.current_value = sustain;
            }

            AdsrStage::Release => {
                let release_samples = release * self.sample_rate;
                if self.stage_progress >= release_samples {
                    self.current_value = 0.0;
                    self.stage = AdsrStage::Idle;
                    self.stage_progress = 0.0;
                } else {
                    let progress = self.stage_progress / release_samples;
                    self.current_value = sustain * (1.0 - progress);
                    self.stage_progress += 1.0;
                }
            }
        }

        // return current value
        self.current_value
    }

    /// Generates the next envelope value for multi-channel audio.
    ///
    /// This is the main method for generating envelope values. For multi-channel audio,
    /// call this once per channel per sample, with `is_first_channel` set to `true` only
    /// for the first channel. This ensures all channels share the same envelope timing.
    ///
    /// # Arguments
    ///
    /// * `attack` - Attack time in seconds (time to rise from 0 to 1)
    /// * `decay` - Decay time in seconds (time to fall from 1 to sustain level)
    /// * `sustain` - Sustain level (the level held during the sustain phase)
    /// * `release` - Release time in seconds (time to fall from sustain to 0)
    /// * `is_first_channel` - Whether this is the first channel (advances timing if true)
    pub fn next_value(
        &mut self,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
        is_first_channel: bool,
    ) -> f32 {
        // Do not advance if it's not first channel, just return the same value
        if !is_first_channel {
            self.current_value
        } else {
            self.next(attack, decay, sustain, release)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to run ADSR for a number of samples
    fn run_adsr_samples(
        adsr: &mut MultiChannelAdsr,
        samples: usize,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    ) -> Vec<f32> {
        (0..samples)
            .map(|_| adsr.next_value(attack, decay, sustain, release, true))
            .collect()
    }

    #[test]
    fn test_initial_state() {
        let adsr = MultiChannelAdsr::new(44100.0);
        assert_eq!(adsr.current_value, 0.0);
        assert!(matches!(adsr.stage, AdsrStage::Idle));
        assert!(adsr.is_idling());
    }

    #[test]
    fn test_normal_adsr_cycle() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        let attack = 0.1; // 0.1 seconds
        let decay = 0.05; // 0.05 seconds
        let sustain = 0.7;
        let release = 0.2; // 0.2 seconds

        // Start note
        adsr.note_on();
        assert!(matches!(adsr.stage, AdsrStage::Attack));
        assert!(!adsr.is_idling());

        // Attack phase - should reach 1.0
        let attack_samples = (attack * 44100.0) as usize;
        let values = run_adsr_samples(
            &mut adsr,
            attack_samples + 1,
            attack,
            decay,
            sustain,
            release,
        );

        // Should be in decay after attack
        assert!(matches!(adsr.stage, AdsrStage::Decay));
        assert_eq!(adsr.current_value, 1.0);

        // Decay phase - should reach sustain level
        let decay_samples = (decay * 44100.0) as usize;
        run_adsr_samples(
            &mut adsr,
            decay_samples + 1,
            attack,
            decay,
            sustain,
            release,
        );

        assert!(matches!(adsr.stage, AdsrStage::Sustain));
        assert!((adsr.current_value - sustain).abs() < 0.001);

        // Sustain phase - should stay at sustain level
        run_adsr_samples(&mut adsr, 1000, attack, decay, sustain, release);
        assert!(matches!(adsr.stage, AdsrStage::Sustain));
        assert!((adsr.current_value - sustain).abs() < 0.001);

        // Release phase
        adsr.note_off();
        assert!(matches!(adsr.stage, AdsrStage::Release));

        let release_samples = (release * 44100.0) as usize;
        run_adsr_samples(
            &mut adsr,
            release_samples + 1,
            attack,
            decay,
            sustain,
            release,
        );

        assert!(matches!(adsr.stage, AdsrStage::Idle));
        assert_eq!(adsr.current_value, 0.0);
        assert!(adsr.is_idling());
    }

    #[test]
    fn test_zero_attack() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // With zero attack, should immediately jump to decay
        let value = adsr.next_value(0.0, 0.1, 0.5, 0.1, true);
        assert!(matches!(adsr.stage, AdsrStage::Decay));
        assert_eq!(value, 1.0);
    }

    #[test]
    fn test_zero_decay() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Run through attack
        let attack_samples = (0.1 * 44100.0) as usize;
        run_adsr_samples(&mut adsr, attack_samples + 2, 0.1, 0.0, 0.5, 0.1);

        // Should be in sustain with zero decay
        assert!(matches!(adsr.stage, AdsrStage::Sustain));
        assert!((adsr.current_value - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_zero_release() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Get to sustain phase
        let attack_samples = (0.1 * 44100.0) as usize;
        let decay_samples = (0.05 * 44100.0) as usize;
        run_adsr_samples(
            &mut adsr,
            attack_samples + decay_samples + 2,
            0.1,
            0.05,
            0.5,
            0.0,
        );

        assert!(matches!(adsr.stage, AdsrStage::Sustain));

        // With zero release, should immediately go to idle
        adsr.note_off();
        let value = adsr.next_value(0.1, 0.05, 0.5, 0.0, true);
        assert!(matches!(adsr.stage, AdsrStage::Idle));
        assert_eq!(value, 0.0);
        assert!(adsr.is_idling());
    }

    #[test]
    fn test_all_zero_times() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // All zero times - should go: Attack->Decay->Sustain in one sample
        let _ = adsr.next_value(0.0, 0.0, 0.8, 0.0, true);
        let value = adsr.next_value(0.0, 0.0, 0.8, 0.0, true);
        assert!(matches!(adsr.stage, AdsrStage::Sustain));
        assert!((value - 0.8).abs() < 0.001);

        // Release should immediately finish
        adsr.note_off();
        let value = adsr.next_value(0.0, 0.0, 0.8, 0.0, true);
        assert!(matches!(adsr.stage, AdsrStage::Idle));
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_extreme_sustain_values() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Test sustain > 1.0
        let attack_samples = (0.01 * 44100.0) as usize;
        let decay_samples = (0.01 * 44100.0) as usize;
        run_adsr_samples(
            &mut adsr,
            attack_samples + decay_samples + 2,
            0.01,
            0.01,
            1.5,
            0.1,
        );

        assert!(matches!(adsr.stage, AdsrStage::Sustain));
        assert!((adsr.current_value - 1.5).abs() < 0.001);

        // Test negative sustain
        let mut adsr2 = MultiChannelAdsr::new(44100.0);
        adsr2.note_on();
        run_adsr_samples(
            &mut adsr2,
            attack_samples + decay_samples + 2,
            0.01,
            0.01,
            -0.3,
            0.1,
        );

        assert!(matches!(adsr2.stage, AdsrStage::Sustain));
        assert!((adsr2.current_value - (-0.3)).abs() < 0.001);
    }

    #[test]
    fn test_very_long_stages() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Test very long attack (1000 seconds)
        let value1 = adsr.next_value(1000.0, 0.1, 0.5, 0.1, true);
        let value2 = adsr.next_value(1000.0, 0.1, 0.5, 0.1, true);

        // Should still be in attack and progressing very slowly
        assert!(matches!(adsr.stage, AdsrStage::Attack));
        assert!(value1 < 0.001); // Very small progress
        assert!(value2 > value1); // But still progressing
    }

    #[test]
    fn test_multichannel_behavior() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // First channel should advance the state
        let value1 = adsr.next_value(0.1, 0.1, 0.5, 0.1, true);
        let stage_after_first = adsr.stage_progress;

        // Second channel (same sample) should return same value without advancing
        let value2 = adsr.next_value(0.1, 0.1, 0.5, 0.1, false);
        let stage_after_second = adsr.stage_progress;

        assert_eq!(value1, value2);
        assert_eq!(stage_after_first, stage_after_second);
    }

    #[test]
    fn test_note_off_during_attack() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Start attack
        adsr.next_value(0.2, 0.1, 0.5, 0.1, true);
        assert!(matches!(adsr.stage, AdsrStage::Attack));

        // Note off during attack - should go to release
        adsr.note_off();
        assert!(matches!(adsr.stage, AdsrStage::Release));
        assert_eq!(adsr.stage_progress, 0.0);
    }

    #[test]
    fn test_note_off_during_decay() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Get to decay phase
        let attack_samples = (0.01 * 44100.0) as usize;
        run_adsr_samples(&mut adsr, attack_samples + 1, 0.01, 0.1, 0.5, 0.1);
        assert!(matches!(adsr.stage, AdsrStage::Decay));

        // Note off during decay - should go to release
        adsr.note_off();
        assert!(matches!(adsr.stage, AdsrStage::Release));
        assert_eq!(adsr.stage_progress, 0.0);
    }

    #[test]
    fn test_note_off_when_idle() {
        let mut adsr = MultiChannelAdsr::new(44100.0);

        // Note off when already idle - should stay idle
        adsr.note_off();
        assert!(matches!(adsr.stage, AdsrStage::Idle));
        assert!(adsr.is_idling());
    }

    #[test]
    fn test_sample_rate_change() {
        let mut adsr = MultiChannelAdsr::new(44100.0);

        // Change sample rate
        adsr.set_sample_rate(48000.0);
        assert_eq!(adsr.sample_rate, 48000.0);

        // Test that it still works with new sample rate
        adsr.note_on();
        let value = adsr.next_value(0.1, 0.1, 0.5, 0.1, true);
        assert!(matches!(adsr.stage, AdsrStage::Attack));
        assert!(value >= 0.0);
    }

    #[test]
    fn test_attack_progression() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        let attack_time = 0.1; // 0.1 seconds
        let expected_samples = (attack_time * 44100.0) as usize;

        let mut values = Vec::new();
        for _ in 0..expected_samples {
            let value = adsr.next_value(attack_time, 0.1, 0.5, 0.1, true);
            values.push(value);
            if matches!(adsr.stage, AdsrStage::Decay) {
                break;
            }
        }

        // Should be monotonically increasing during attack
        for i in 1..values.len() {
            assert!(
                values[i] >= values[i - 1],
                "Attack should be monotonically increasing"
            );
        }

        // Last value should be close to 1.0 or we should be in decay
        assert!(values.last().unwrap() > &0.9 || matches!(adsr.stage, AdsrStage::Decay));
    }

    #[test]
    fn test_decay_progression() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Get through attack quickly
        let attack_samples = (0.01 * 44100.0) as usize;
        run_adsr_samples(&mut adsr, attack_samples + 1, 0.01, 0.1, 0.3, 0.1);

        assert!(matches!(adsr.stage, AdsrStage::Decay));

        let mut values = Vec::new();
        let decay_samples = (0.1 * 44100.0) as usize;

        for _ in 0..=decay_samples {
            let value = adsr.next_value(0., 0.1, 0.3, 0.1, true);
            values.push(value);
            if matches!(adsr.stage, AdsrStage::Sustain) {
                break;
            }
        }

        // Should be monotonically decreasing during decay
        for i in 1..values.len() {
            assert!(
                values[i] <= values[i - 1],
                "Decay should be monotonically decreasing"
            );
        }

        // Should end up at or near sustain level
        assert!(matches!(adsr.stage, AdsrStage::Sustain));
        assert!((adsr.current_value - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_release_progression() {
        let mut adsr = MultiChannelAdsr::new(44100.0);
        adsr.note_on();

        // Get to sustain phase
        let attack_samples = (0.01 * 44100.0) as usize;
        let decay_samples = (0.01 * 44100.0) as usize;
        run_adsr_samples(
            &mut adsr,
            attack_samples + decay_samples + 10,
            0.01,
            0.01,
            0.7,
            0.1,
        );

        assert!(matches!(adsr.stage, AdsrStage::Sustain));

        // Start release
        adsr.note_off();

        let mut values = Vec::new();
        let release_samples = (0.1 * 44100.0) as usize;

        for _ in 0..release_samples + 1 {
            let value = adsr.next_value(0.01, 0.01, 0.7, 0.1, true);
            values.push(value);
            if matches!(adsr.stage, AdsrStage::Idle) {
                break;
            }
        }

        // Should be monotonically decreasing during release
        for i in 1..values.len() - 1 {
            // Skip last value as it might be 0.0
            if values[i - 1] > 0.0 && values[i] > 0.0 {
                assert!(
                    values[i] <= values[i - 1],
                    "Release should be monotonically decreasing"
                );
            }
        }

        // Should end up at 0.0 and idle
        assert!(matches!(adsr.stage, AdsrStage::Idle));
        assert_eq!(adsr.current_value, 0.0);
    }
}

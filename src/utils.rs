use std::path::Path;

use nih_plug::prelude::{Smoothable, Smoother};

#[inline]
pub fn load_smooth_param<T: Smoothable>(smoother: &Smoother<T>, is_first_channel: bool) -> T {
    if is_first_channel {
        smoother.next()
    } else {
        smoother.previous_value()
    }
}

pub fn load_audio_file(file_path: &Path) -> Result<(u32, Vec<f32>), Box<dyn std::error::Error>> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("wav") => load_wav(file_path),
        _ => Err("Unsupported file format".into()),
    }
}

fn load_wav(file_path: &Path) -> Result<(u32, Vec<f32>), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(file_path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => match spec.bits_per_sample {
            16 => reader
                .samples::<i16>()
                .map(|s| s.map(|sample| sample as f32 / i16::MAX as f32))
                .collect::<Result<Vec<_>, _>>()?,
            24 => reader
                .samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / ((1 << 23) - 1) as f32))
                .collect::<Result<Vec<_>, _>>()?,
            32 => reader
                .samples::<i32>()
                .map(|s| s.map(|sample| sample as f32 / i32::MAX as f32))
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err(format!("Unsupported bit depth: {}", spec.bits_per_sample).into()),
        },
    };

    Ok((sample_rate, samples))
}

#[inline]
pub fn interpolate(v1: f32, v2: f32, fraction: f32) -> f32 {
    v1 * (1. - fraction) + v2 * fraction
}

pub fn semitones_to_note(mut semi: i32) -> String {
    // Handle negative values and values >= 12 by wrapping to 0-11 range
    if semi < 0 {
        semi = ((semi % 12) + 12) % 12;
    } else if semi >= 12 {
        semi = semi % 12;
    }

    let value = match semi {
        0 => "C",
        1 => "C#",
        2 => "D",
        3 => "D#",
        4 => "E",
        5 => "F",
        6 => "F#",
        7 => "G",
        8 => "G#",
        9 => "A",
        10 => "A#",
        11 => "B",
        _ => unreachable!("Semitone value should be 0-11 after modulo operation"),
    };

    String::from(value)
}

use std::path::{Path, PathBuf};

use nih_plug::prelude::{Smoothable, Smoother};

use crate::{params::BlendGroup, tasks::AudioData};

#[inline]
pub fn load_smooth_param<T: Smoothable>(smoother: &Smoother<T>, is_first_channel: bool) -> T {
    if is_first_channel {
        smoother.next()
    } else {
        smoother.previous_value()
    }
}

pub fn load_audio_file(file_path: &Path) -> Result<AudioData, Box<dyn std::error::Error>> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("wav") => load_wav(file_path),
        _ => Err("Unsupported file format".into()),
    }
}

fn load_wav(file_path: &Path) -> Result<AudioData, Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(file_path)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match &spec.sample_format {
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

    Ok(AudioData::new(spec, samples))
}

#[inline]
pub fn interpolate(v1: f32, v2: f32, fraction: f32) -> f32 {
    v1 * (1. - fraction) + v2 * fraction
}

pub fn semitones_to_note(mut semi: i32) -> String {
    // Handle negative values and values >= 12 by wrapping to 0-11 range
    if semi < 0 {
        semi = semi.rem_euclid(12);
    } else if semi >= 12 {
        semi %= 12;
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

fn get_sorted_files_in_directory(file_path: &str) -> Option<Vec<PathBuf>> {
    let path = Path::new(file_path);
    let parent = path.parent()?;

    // Read directory entries and collect files
    let mut entries: Vec<PathBuf> = std::fs::read_dir(parent)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();

    // Sort entries for consistent ordering
    entries.sort();

    Some(entries)
}

pub fn get_next_file_in_directory_wrap(file_path: &str) -> Option<PathBuf> {
    let path = Path::new(file_path);
    let current_filename = path.file_name()?;

    let entries = get_sorted_files_in_directory(file_path)?;

    if entries.is_empty() {
        return None;
    }

    let current_index = entries
        .iter()
        .position(|p| p.file_name() == Some(current_filename))?;

    // Wrap around to first if at end
    let next_index = (current_index + 1) % entries.len();
    Some(entries[next_index].clone())
}

pub fn get_previous_file_in_directory_wrap(file_path: &str) -> Option<PathBuf> {
    let path = Path::new(file_path);
    let current_filename = path.file_name()?;

    let entries = get_sorted_files_in_directory(file_path)?;

    if entries.is_empty() {
        return None;
    }

    let current_index = entries
        .iter()
        .position(|p| p.file_name() == Some(current_filename))?;

    // Wrap around to last if at beginning
    let prev_index = if current_index == 0 {
        entries.len() - 1
    } else {
        current_index - 1
    };

    Some(entries[prev_index].clone())
}

pub fn clipping_sub(lhs: usize, rhs: usize) -> Option<usize> {
    if lhs >= rhs {
        Some(lhs - rhs)
    } else {
        None
    }
}

pub fn get_root_note_from_filename(file_name: String) -> Option<i32> {
    // chunk with some common separator
    for chunk in file_name.split(['_', ' ', '-', '.']).rev() {
        match chunk.to_uppercase().as_str() {
            "C" => return Some(0),
            "C#" | "CS" | "DB" => return Some(1),
            "D" => return Some(2),
            "D#" | "DS" | "EB" => return Some(3),
            "E" => return Some(4),
            "F" => return Some(5),
            "F#" | "FS" | "GB" => return Some(6),
            "G" => return Some(7),
            "G#" | "GS" | "AB" => return Some(8),
            "A" => return Some(9),
            "A#" | "AS" | "BB" => return Some(10),
            "B" => return Some(11),
            _ => {}
        };
    }
    None
}

#[inline]
pub fn get_stretch_playback_position(
    process_count: f32,
    sr_correction: f32,
    playback_rate: f32,
    num_channels: usize,
    channel_index: usize,
) -> (usize, f32) {
    let raw_playback_position = process_count * sr_correction;
    let pitched_position = playback_rate * raw_playback_position;

    let frame_index = pitched_position as usize;
    let fraction = pitched_position.fract();
    let sample_index = frame_index * num_channels + channel_index;

    (sample_index, fraction)
}

#[inline]
pub fn get_blend_value(
    group: BlendGroup,
    current_time: f32,
    blend_time: f32,
    blend_transition: f32,
) -> f32 {
    let value = match group {
        BlendGroup::None => 1.,
        BlendGroup::Start => {
            let blend_start = blend_time - blend_transition / 2.;
            let blend_end = blend_start + blend_transition;

            if current_time < blend_start {
                1.
            } else if current_time > blend_end {
                0.
            } else {
                (blend_end - current_time) / blend_transition
            }
        }
        BlendGroup::End => {
            let blend_start = blend_time - blend_transition / 2.0;
            let blend_end = blend_start + blend_transition;

            if current_time < blend_start {
                0.0 // Silent before blend starts
            } else if current_time > blend_end {
                1.0 // Full volume after blend ends
            } else {
                // Linear fade from 0.0 to 1.0 during transition
                (current_time - blend_start) / blend_transition
            }
        }
    };

    // never too safe in audio processing
    if !value.is_finite() {
        1.
    } else {
        value.clamp(0., 1.)
    }
}

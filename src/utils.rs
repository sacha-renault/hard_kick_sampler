use nih_plug::prelude::{Smoothable, Smoother};

pub fn load_smooth_param<T: Smoothable>(smoother: &Smoother<T>, is_first_channel: bool) -> T {
    if is_first_channel {
        smoother.next()
    } else {
        smoother.previous_value()
    }
}

pub fn load_audio_file(file_path: &str) -> Result<(u32, Vec<f32>), Box<dyn std::error::Error>> {
    let path = std::path::Path::new(file_path);

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("wav") => load_wav(file_path),
        _ => Err("Unsupported file format".into()),
    }
}

fn load_wav(file_path: &str) -> Result<(u32, Vec<f32>), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(file_path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => {
            // Already float samples
            reader.samples::<f32>().map(|s| s.unwrap()).collect()
        }
        hound::SampleFormat::Int => {
            // Integer samples - need to convert based on bit depth
            match spec.bits_per_sample {
                16 => reader
                    .samples::<i16>()
                    .map(|s| s.unwrap() as f32 / i16::MAX as f32)
                    .collect(),
                24 => {
                    reader
                        .samples::<i32>()
                        .map(|s| s.unwrap() as f32 / (1 << 23) as f32) // 24-bit is stored in i32
                        .collect()
                }
                32 => reader
                    .samples::<i32>()
                    .map(|s| s.unwrap() as f32 / i32::MAX as f32)
                    .collect(),
                _ => return Err(format!("Unsupported bit depth: {}", spec.bits_per_sample).into()),
            }
        }
    };

    Ok((sample_rate, samples))
}

pub fn interpolate(buffer: &Vec<f32>, position: f32) -> f32 {
    let index = position.floor() as usize;
    if index >= buffer.len() {
        0.
    } else if index + 1 == buffer.len() {
        buffer[index]
    } else {
        let n = buffer[index];
        let n_plus_1 = buffer[index + 1];
        let fraction = position - (index as f32);
        n * (1. - fraction) + n_plus_1 * fraction
    }
}

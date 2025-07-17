use crate::plugin::DEFAULT_BPM;

pub fn get_num_displayed_frames(bars: f32, sr: f32, mut bpm: f32) -> usize {
    if bpm <= 0. {
        bpm = DEFAULT_BPM as f32;
    }
    (bars * 60.0 * sr / bpm) as usize
}

pub fn get_waveform(
    data: &[f32],
    num_frames: usize,
    num_channels: usize,
    channel_index: usize,
    offset_seconds: f32,
    sample_rate: f32,
) -> Vec<[f32; 2]> {
    let offset_frames = (offset_seconds * sample_rate) as i32;
    let total_frames_in_data = data.len() / num_channels;

    let mut result = Vec::new();

    for i in 0..num_frames {
        let frame_index = i as i32 + offset_frames;

        let sample_value = if frame_index >= 0 && (frame_index as usize) < total_frames_in_data {
            // Extract sample for the specific channel
            let data_index = (frame_index as usize) * num_channels + channel_index;
            data[data_index]
        } else {
            // Outside bounds - return silence
            0.0
        };

        // Normalize x position (0.0 to 1.0)
        let x_norm = i as f32 / (num_frames - 1) as f32;

        result.push([x_norm, sample_value]);
    }

    result
}

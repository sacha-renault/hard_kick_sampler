use std::path::{Path, PathBuf};

use egui::*;

#[derive(Clone, Default)]
pub struct WaveformCache {
    texture_handle: Option<TextureHandle>,
    data_path: PathBuf,
}

impl WaveformCache {
    pub fn get_texture_id(&self) -> Option<TextureId> {
        self.texture_handle.as_ref().map(|handle| handle.id())
    }

    pub fn update_if_needed(
        &mut self,
        ctx: &egui::Context,
        data: &Vec<f32>,
        data_path: &Path,
        viewport: (f32, f32),
    ) {
        if self.texture_handle.is_none() || self.data_path != data_path {
            let (width, height) = viewport;
            let image = render_waveform_as_image(
                data,
                width as usize,
                height as usize,
                Color32::from_gray(25),
                2,
            );
            self.texture_handle = Some(ctx.load_texture("waveform", image, Default::default()));
            self.data_path = data_path.to_path_buf();
        }
    }
}

fn render_waveform_as_image(
    data: &Vec<f32>,
    width: usize,
    height: usize,
    background_color: Color32,
    line_thickness: usize,
) -> ColorImage {
    render_stereo_waveform_with_padding(
        data,
        width,
        height,
        background_color,
        Color32::from_rgb(100, 255, 100), // Left channel - green
        Color32::from_rgb(255, 100, 100), // Right channel - red
        line_thickness,
        20, // padding
    )
}

/// Renders interleaved stereo audio data (L, R, L, R, L, R...) with padding
fn render_stereo_waveform_with_padding(
    data: &Vec<f32>,
    width: usize,
    height: usize,
    background_color: Color32,
    left_color: Color32,
    right_color: Color32,
    line_thickness: usize,
    padding: usize,
) -> ColorImage {
    // Create image buffer
    let mut pixels = vec![background_color; width * height];

    if data.is_empty() {
        return ColorImage {
            size: [width, height],
            pixels,
        };
    }

    // Calculate drawable area (subtract padding)
    let drawable_width = width;
    let drawable_height = height.saturating_sub(padding * 2);

    if drawable_width == 0 || drawable_height == 0 {
        return ColorImage {
            size: [width, height],
            pixels,
        };
    }

    // Split stereo data into left and right channels
    let (left_data, right_data) = split_interleaved_stereo(data);

    // Find global min/max for consistent scaling
    let left_min = left_data.iter().cloned().fold(f32::INFINITY, f32::min);
    let left_max = left_data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let right_min = right_data.iter().cloned().fold(f32::INFINITY, f32::min);
    let right_max = right_data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

    let global_min = left_min.min(right_min);
    let global_max = left_max.max(right_max);
    let range = global_max - global_min;

    if range == 0.0 {
        // Draw flat lines in the middle of each channel
        let left_y = padding + drawable_height / 4;
        let right_y = padding + (drawable_height * 3) / 4;

        for x in 0..drawable_width {
            set_pixel_safe(
                &mut pixels,
                padding + x,
                left_y,
                width,
                height,
                left_color,
                line_thickness,
            );
            set_pixel_safe(
                &mut pixels,
                padding + x,
                right_y,
                width,
                height,
                right_color,
                line_thickness,
            );
        }
        return ColorImage {
            size: [width, height],
            pixels,
        };
    }

    // Draw center line between channels
    let center_y = padding + drawable_height / 2;
    for x in padding..width - padding {
        set_pixel_safe(
            &mut pixels,
            x,
            center_y,
            width,
            height,
            Color32::from_gray(60),
            1,
        );
    }

    // Render left channel (top half)
    render_channel(
        &mut pixels,
        &left_data,
        padding,
        padding,
        drawable_width,
        drawable_height / 2,
        width,
        height,
        global_min,
        range,
        left_color,
        line_thickness,
    );

    // Render right channel (bottom half)
    render_channel(
        &mut pixels,
        &right_data,
        padding,
        padding + drawable_height / 2,
        drawable_width,
        drawable_height / 2,
        width,
        height,
        global_min,
        range,
        right_color,
        line_thickness,
    );

    ColorImage {
        size: [width, height],
        pixels,
    }
}

/// Split interleaved stereo data into separate left and right channels
fn split_interleaved_stereo(data: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let mut left = Vec::with_capacity(data.len() / 2);
    let mut right = Vec::with_capacity(data.len() / 2);

    for chunk in data.chunks_exact(2) {
        left.push(chunk[0]); // Left channel
        right.push(chunk[1]); // Right channel
    }

    // Handle odd number of samples (shouldn't happen with proper stereo data)
    if data.len() % 2 == 1 {
        left.push(data[data.len() - 1]);
        right.push(0.0); // Pad right channel with silence
    }

    (left, right)
}

/// Render a single channel within specified bounds
fn render_channel(
    pixels: &mut Vec<Color32>,
    channel_data: &[f32],
    start_x: usize,
    start_y: usize,
    channel_width: usize,
    channel_height: usize,
    image_width: usize,
    image_height: usize,
    global_min: f32,
    range: f32,
    color: Color32,
    line_thickness: usize,
) {
    if channel_data.is_empty() || channel_width == 0 || channel_height == 0 {
        return;
    }

    // Sample data points for each x coordinate
    for x in 0..channel_width {
        let data_pos = (x as f32 / channel_width as f32) * (channel_data.len() - 1) as f32;
        let data_idx = data_pos as usize;

        // Get interpolated value
        let value = if data_idx + 1 < channel_data.len() {
            let frac = data_pos - data_idx as f32;
            channel_data[data_idx] * (1.0 - frac) + channel_data[data_idx + 1] * frac
        } else {
            channel_data[data_idx.min(channel_data.len() - 1)]
        };

        // Normalize to channel height (flip Y coordinate for screen space)
        let normalized = (value - global_min) / range;
        let y = ((1.0 - normalized) * (channel_height - 1) as f32) as usize;

        // Draw the point with line thickness
        set_pixel_safe(
            pixels,
            start_x + x,
            start_y + y,
            image_width,
            image_height,
            color,
            line_thickness,
        );
    }

    // Connect points with lines for smoother appearance
    for x in 0..channel_width - 1 {
        let data_pos1 = (x as f32 / channel_width as f32) * (channel_data.len() - 1) as f32;
        let data_pos2 = ((x + 1) as f32 / channel_width as f32) * (channel_data.len() - 1) as f32;

        let value1 = get_interpolated_value(channel_data, data_pos1);
        let value2 = get_interpolated_value(channel_data, data_pos2);

        let y1 = ((1.0 - (value1 - global_min) / range) * (channel_height - 1) as f32) as i32;
        let y2 = ((1.0 - (value2 - global_min) / range) * (channel_height - 1) as f32) as i32;

        // Draw line between points
        draw_line(
            pixels,
            (start_x + x) as i32,
            (start_y as i32) + y1,
            (start_x + x + 1) as i32,
            (start_y as i32) + y2,
            image_width,
            image_height,
            color,
        );
    }
}

/// Helper function to get interpolated value from data
fn get_interpolated_value(data: &[f32], pos: f32) -> f32 {
    let idx = pos as usize;
    if idx + 1 < data.len() {
        let frac = pos - idx as f32;
        data[idx] * (1.0 - frac) + data[idx + 1] * frac
    } else {
        data[idx.min(data.len() - 1)]
    }
}

/// Set a pixel with bounds checking and line thickness
fn set_pixel_safe(
    pixels: &mut Vec<Color32>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color32,
    thickness: usize,
) {
    let half_thickness = thickness / 2;

    for dy in 0..thickness {
        for dx in 0..thickness {
            let px = x.saturating_add(dx);
            let py = y.saturating_add(dy).saturating_sub(half_thickness);

            if px < width && py < height {
                let index = py * width + px;
                if index < pixels.len() {
                    pixels[index] = color;
                }
            }
        }
    }
}

/// Draw a line between two points using Bresenham's algorithm
fn draw_line(
    pixels: &mut Vec<Color32>,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    width: usize,
    height: usize,
    color: Color32,
) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
            let index = (y as usize) * width + (x as usize);
            if index < pixels.len() {
                pixels[index] = color;
            }
        }

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

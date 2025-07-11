use derive_more::Constructor;
use egui::Color32;
use egui_plot::*;

use crate::params::BlendGroup;

const N_BEAT_DISPLAYED: f32 = 1.5;

/// Calculate the increment to avoid too may data
///
/// # Arguments
///
/// * `width` - width of the viewport in pxl
/// * `num_data` - number of data in the plot
/// * `num_channels` - number of channel in the interleaved data
// pub fn get_step_by_value(width: f32, num_data: f32, num_channels: f32) -> usize {
//     if width <= 0. {
//         return num_channels as usize;
//     }

//     let per_channel_num = num_data / num_channels;

//     if per_channel_num <= width {
//         return num_channels as usize;
//     }

//     let samples_to_skip = per_channel_num / width;
//     let density = 2.; // might change later
//     (samples_to_skip / density).max(1.) as usize * num_channels as usize
// }

#[derive(Debug, Constructor)]
pub struct WavePlot<'a> {
    buffer: &'a Vec<f32>,
    trim_start: f32,
    delay_start: f32,
    sample_rate: f32,
    num_channels: usize,
    position: u64,
    blend_group: BlendGroup,
    blend_time: f32,
    blend_transition: f32,
    samples_per_beat: f32,
}

impl WavePlot<'_> {
    pub fn data(&self, channel_index: usize) -> impl Iterator<Item = [f64; 2]> + '_ {
        let num_skip = (self.trim_start * self.sample_rate) as usize * self.num_channels;
        let num_silent = (self.delay_start * self.sample_rate) as usize;
        let step = self.num_channels;

        get_plot_line(
            self.buffer,
            num_silent,
            num_skip,
            step,
            self.samples_per_beat * N_BEAT_DISPLAYED,
            channel_index,
        )
    }

    pub fn silent(&self) -> impl Iterator<Item = [f64; 2]> {
        let num_silent = (self.delay_start * self.sample_rate) as usize;
        vec![[0.0, 0.0], [num_silent as f64, 0.0]].into_iter()
    }

    pub fn position(&self) -> impl Iterator<Item = [f64; 2]> {
        let fpos = self.num_channels as f64 * self.position as f64 / self.num_channels as f64;
        vec![[fpos, -1.], [fpos, 1.]].into_iter()
    }

    pub fn blend_plot(&self) -> Option<impl Iterator<Item = [f64; 2]>> {
        match &self.blend_group {
            BlendGroup::None => None,

            BlendGroup::Start => {
                // Calculate blend region boundaries in samples
                let blend_start_sample =
                    (self.blend_time - self.blend_transition / 2.) * self.sample_rate;
                let blend_end_sample =
                    (self.blend_time + self.blend_transition / 2.) * self.sample_rate;

                // Convert sample positions to pixel positions
                let blend_start_pixel = blend_start_sample as f64;
                let blend_end_pixel = blend_end_sample as f64;

                // Create plot points: [pixel_position, amplitude]
                // Start at full amplitude (1.0), maintain until blend starts, then fade to -1.0
                Some(
                    vec![
                        [0.0, 1.0],               // Beginning of audio at full amplitude
                        [blend_start_pixel, 1.0], // Maintain amplitude until blend starts
                        [blend_end_pixel, -1.0],  // Fade to negative amplitude at blend end
                    ]
                    .into_iter(),
                )
            }

            BlendGroup::End => {
                // Calculate blend region boundaries in samples
                let blend_start_sample =
                    (self.blend_time - self.blend_transition / 2.) * self.sample_rate;
                let blend_end_sample =
                    (self.blend_time + self.blend_transition / 2.) * self.sample_rate;
                let audio_end_sample =
                    self.delay_start * self.sample_rate + self.buffer.len() as f32;

                // Convert sample positions to pixel positions
                let blend_start_pixel = blend_start_sample as f64;
                let blend_end_pixel = blend_end_sample as f64;
                let audio_end_pixel = audio_end_sample as f64;

                // Create plot points: [pixel_position, amplitude]
                // Start at negative amplitude, fade to positive, then maintain until end
                Some(
                    vec![
                        [blend_start_pixel, -1.0], // Start blend at negative amplitude
                        [blend_end_pixel, 1.0],    // Fade to full amplitude at blend end
                        [audio_end_pixel, 1.0],    // Maintain amplitude until audio ends
                    ]
                    .into_iter(),
                )
            }
        }
    }

    pub fn display(&self, ui: &mut egui::Ui, channel_index: usize) {
        Plot::new(format!("{}Plot", channel_index))
            // .legend(Legend::default())
            .allow_drag(false)
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .allow_double_click_reset(false)
            .allow_boxed_zoom(false)
            .show_grid(true)
            .show_axes([false, false])
            .show(ui, |ui| {
                // Set the bounds
                let samples_displayed = self.samples_per_beat * N_BEAT_DISPLAYED;
                ui.set_plot_bounds(PlotBounds::from_min_max(
                    [0., -1.],
                    [samples_displayed as f64, 1.],
                ));

                // If the sample is in a blend group
                // We display the blend
                if let Some(plot_data) = self.blend_plot() {
                    ui.line(
                        Line::new(
                            format!("{}_Blend", channel_index),
                            PlotPoints::from_iter(plot_data),
                        )
                        .fill(-1.)
                        .color(Color32::LIGHT_GRAY),
                    );
                }

                ui.line(
                    Line::new(
                        format!("{}_Silent", channel_index),
                        PlotPoints::from_iter(self.silent()),
                    )
                    .color(Color32::LIGHT_RED),
                );
                ui.line(
                    Line::new(
                        format!("{}_Data", channel_index),
                        PlotPoints::from_iter(self.data(channel_index)),
                    )
                    .color(Color32::LIGHT_RED),
                );
                ui.line(
                    Line::new(
                        format!("{}_Play_Position", channel_index),
                        PlotPoints::from_iter(self.position()),
                    )
                    .color(Color32::LIGHT_BLUE)
                    .width(4.),
                );
            });
    }
}

pub fn get_plot_line(
    buffer: &[f32],
    num_silent: usize,
    num_skip: usize,
    step_by: usize,
    samples_per_beat: f32,
    channel_index: usize,
) -> impl Iterator<Item = [f64; 2]> + '_ {
    buffer
        .iter()
        .skip(num_skip + channel_index)
        .step_by(step_by)
        .take(samples_per_beat as usize)
        .enumerate()
        .map(move |(i, &y)| [(num_silent + i) as f64, y as f64])
}

use derive_more::Constructor;
use egui::Color32;
use egui_plot::*;

pub fn render_waveform_stereo(ui: &mut egui::Ui, channel_index: usize, line_data: &PlotData) {
    let width = ui.available_width();

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
            ui.line(
                Line::new(
                    channel_index.to_string(),
                    PlotPoints::from_iter(line_data.silent()),
                )
                .color(Color32::PURPLE),
            );
            ui.line(
                Line::new(
                    channel_index.to_string(),
                    PlotPoints::from_iter(line_data.data(width, channel_index)),
                )
                .color(Color32::PURPLE),
            );
            ui.line(
                Line::new(
                    channel_index.to_string(),
                    PlotPoints::from_iter(line_data.position(width)),
                )
                .color(Color32::BLUE),
            );
        });
}

/// Calculate the increment to avoid too may data
///
/// # Arguments
///
/// * `width` - width of the viewport in pxl
/// * `num_data` - number of data in the plot
/// * `num_channels` - number of channel in the interleaved data
pub fn get_step_by_value(width: f32, num_data: f32, num_channels: f32) -> usize {
    if width <= 0. {
        return num_channels as usize;
    }

    let per_channel_num = num_data / num_channels;

    if per_channel_num <= width {
        return num_channels as usize;
    }

    let samples_to_skip = per_channel_num / width;
    let density = 1.; // might change later
    (samples_to_skip / density).max(1.) as usize * num_channels as usize
}

#[derive(Debug, Constructor)]
pub struct PlotData<'a> {
    buffer: &'a Vec<f32>,
    trim_start: f32,
    delay_start: f32,
    sample_rate: f32,
    num_channels: usize,
    position: u64,
}

impl PlotData<'_> {
    pub fn data(&self, width: f32, channel_index: usize) -> impl Iterator<Item = [f64; 2]> + '_ {
        let step_by = get_step_by_value(width, self.buffer.len() as f32, self.num_channels as f32);
        let num_skip = (self.trim_start * self.sample_rate) as usize * self.num_channels;
        let num_silent = (self.delay_start * self.sample_rate) as usize;

        get_plot_line(self.buffer, num_silent, num_skip, step_by, channel_index)
    }

    pub fn silent(&self) -> impl Iterator<Item = [f64; 2]> {
        let num_silent = (self.delay_start * self.sample_rate) as usize;
        vec![[0.0, 0.0], [num_silent as f64, 0.0]].into_iter()
    }

    pub fn position(&self, width: f32) -> impl Iterator<Item = [f64; 2]> {
        let step_by = get_step_by_value(width, self.buffer.len() as f32, self.num_channels as f32);
        let fpos = self.num_channels as f64 * self.position as f64 / step_by as f64;
        vec![[fpos, -1.], [fpos, 1.]].into_iter()
    }
}

pub fn get_plot_line(
    buffer: &[f32],
    num_silent: usize,
    num_skip: usize,
    step_by: usize,
    channel_index: usize,
) -> impl Iterator<Item = [f64; 2]> + '_ {
    buffer
        .iter()
        .skip(num_skip + channel_index)
        .step_by(step_by)
        .enumerate()
        .map(move |(i, &y)| [(num_silent + i) as f64, y as f64])
}

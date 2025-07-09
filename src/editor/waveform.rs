use egui::Color32;
use egui_plot::*;

pub fn render_waveform_stereo(
    ui: &mut egui::Ui,
    data: &Vec<f32>,
    channel_index: usize,
    num_channels: usize,
    trim_start: f32,
    delay_start: f32,
    sample_rate: f32,
) {
    let skip = trim_start * sample_rate * num_channels as f32 + channel_index as f32;
    let add_silent = (delay_start * sample_rate) as usize;
    let step_by = get_step_by_value(ui.available_width(), data.len() as f32, num_channels as f32);

    let silent_data = vec![[0.0, 0.0], [add_silent as f64, 0.0]];
    let plot_data = data
        .iter()
        .skip(skip as usize)
        .step_by(step_by)
        .enumerate()
        .map(|(i, &y)| [(add_silent + i) as f64, y as f64]);
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
            ui.line(Line::new(
                channel_index.to_string(),
                PlotPoints::from_iter(plot_data),
            ));
            ui.line(
                Line::new(
                    channel_index.to_string(),
                    PlotPoints::from_iter(silent_data),
                )
                .color(Color32::GREEN),
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
fn get_step_by_value(width: f32, num_data: f32, num_channels: f32) -> usize {
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

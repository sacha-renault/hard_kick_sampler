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

    let silent_data = vec![[0.0, 0.0], [add_silent as f64, 0.0]];
    let plot_data = data
        .iter()
        .skip(skip as usize)
        .step_by(num_channels)
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
                PlotPoints::from_iter(silent_data.into_iter().chain(plot_data)),
            ));
        });
}

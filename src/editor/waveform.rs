use egui_plot::*;

pub fn render_waveform_stereo(
    ui: &mut egui::Ui,
    data: &Vec<f32>,
    channel_index: usize,
    num_channels: usize,
) {
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
                PlotPoints::from_iter(
                    data.iter()
                        .skip(channel_index)
                        .step_by(num_channels)
                        .enumerate()
                        .map(|(i, &y)| [i as f64, y as f64]),
                ),
            ));
        });
}

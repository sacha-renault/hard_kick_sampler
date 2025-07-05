use egui_plot::*;

pub fn render_waveform_stereo(ui: &mut egui::Ui, data: &Vec<f32>) {
    Plot::new("Plot")
        .legend(Legend::default())
        .allow_drag(false)
        .allow_scroll(false)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_double_click_reset(false)
        .allow_boxed_zoom(false)
        .show_grid(false)
        .show_axes([false, false])
        .show(ui, |ui| {
            ui.line(Line::new(
                "L",
                PlotPoints::from_iter(
                    data.iter()
                        .step_by(2)
                        .enumerate()
                        .map(|(i, &y)| [i as f64, y as f64]),
                ),
            ));
            ui.line(Line::new(
                "R",
                PlotPoints::from_iter(
                    data.iter()
                        .skip(1)
                        .step_by(2)
                        .enumerate()
                        .map(|(i, &y)| [i as f64, y as f64 - 2.0]),
                ),
            ));
        });
}

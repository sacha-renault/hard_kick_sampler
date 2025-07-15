use nih_plug_vizia::vizia::{prelude::*, vg};

/// Static waveform.
///
/// For displaying frequently updating waveform data, use an [`Oscilloscope`]
/// instead.
pub struct Waveform {
    data: Vec<f32>,
}

impl Waveform {
    pub fn new(cx: &mut Context, data: Vec<f32>) -> Handle<Self> {
        Self { data }.build(cx, |_| {})
    }
}

impl View for Waveform {
    fn element(&self) -> Option<&'static str> {
        Some("waveform")
    }
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let bounds = cx.bounds();

        let x = bounds.x;
        let y = bounds.y;
        let w = bounds.w;
        let h = bounds.h;

        // Waveform
        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                path.move_to(x, y + (h / 2.) * (1. - self.data[0].clamp(-1., 1.)));

                for (i, &v) in self.data.iter().enumerate().skip(1) {
                    let x_pos = x + (w / self.data.len() as f32) * i as f32;
                    let y_pos = y + h / 2.0 - v.clamp(-1., 1.) * (h / 2.0);

                    path.line_to(x_pos, y_pos);
                }
                path
            },
            &vg::Paint::color(cx.font_color().into())
                .with_line_width(cx.scale_factor() * cx.outline_width()),
        );
    }
}

use nih_plug_vizia::vizia::{prelude::*, vg};

/// Static waveform.
///
/// For displaying frequently updating waveform data, use an [`Oscilloscope`]
/// instead.
pub struct Waveform {
    data: Vec<[f32; 2]>,
}

impl Waveform {
    pub fn new(cx: &mut Context, data: Vec<[f32; 2]>) -> Handle<Self> {
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
        let hh = bounds.h / 2.0; // half height

        // Waveform
        canvas.stroke_path(
            &{
                let mut path = vg::Path::new();

                if let Some(first_point) = self.data.first() {
                    let x_pos = x + first_point[0] * w;
                    let y_pos = y + hh - first_point[1].clamp(-1., 1.) * hh;
                    path.move_to(x_pos, y_pos);
                }

                for &[x_norm, v] in self.data.iter().skip(1) {
                    let x_pos = x + x_norm * w;
                    let y_pos = y + hh - v.clamp(-1., 1.) * hh;
                    path.line_to(x_pos, y_pos);
                }
                path
            },
            &vg::Paint::color(cx.font_color().into())
                .with_line_width(cx.scale_factor() * cx.outline_width()),
        );
    }
}

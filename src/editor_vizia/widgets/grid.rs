use nih_plug_vizia::vizia::{
    context::{Context, DrawContext},
    layout::BoundingBox,
    vg,
    view::{Canvas, Handle, View},
    views::Orientation,
};

pub struct StaticGridLines {
    lines: Vec<f32>,
    orientation: Orientation,
}

impl StaticGridLines {
    pub fn new(cx: &mut Context, lines: Vec<f32>, orientation: Orientation) -> Handle<Self> {
        Self { lines, orientation }.build(cx, |_| {})
    }
}

impl View for StaticGridLines {
    fn element(&self) -> Option<&'static str> {
        Some("static-grid-line")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let BoundingBox { x, y, w, h } = cx.bounds();
        let mut path = vg::Path::new();

        match self.orientation {
            Orientation::Horizontal => {
                for &line in self.lines.iter() {
                    let line_height = y + line.clamp(0., 1.) * h;
                    path.move_to(x, line_height);
                    path.line_to(x + w, line_height);
                }
            }
            Orientation::Vertical => {
                for &line in self.lines.iter() {
                    let line_position = x + line.clamp(0., 1.) * w;
                    path.move_to(line_position, y);
                    path.line_to(line_position, y + h);
                }
            }
        }

        canvas.stroke_path(
            &path,
            &vg::Paint::color(cx.font_color().into())
                .with_line_width(cx.scale_factor() * cx.outline_width()),
        );
    }
}

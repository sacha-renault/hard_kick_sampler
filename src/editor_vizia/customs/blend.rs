use nih_plug_vizia::vizia::{prelude::*, vg};

#[derive(Lens)]
pub struct BlendVizualizer<L: Lens<Target = f32>> {
    blend_time: L,
    blend_transition: L,
}

impl<L: Lens<Target = f32>> BlendVizualizer<L> {
    pub fn new(cx: &mut Context, blend_time: L, blend_transition: L) -> Handle<Self> {
        Self {
            blend_time,
            blend_transition,
        }
        .build(cx, |_| {})
        .class("blend-vizualizer")
    }
}

impl<L: Lens<Target = f32>> View for BlendVizualizer<L> {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let time = self.blend_time.get(cx);
        let transition = self.blend_transition.get(cx);

        // First let's do the blend time (bar on the center)
        let BoundingBox { x, y, w, h } = cx.bounds();

        let mut middle_path = vg::Path::new();
        let mid_x = x + time.clamp(0., 1.) * w;
        let half_transition = (transition.clamp(0., 1.) * w) / 2.0;
        middle_path.move_to(mid_x, y);
        middle_path.line_to(mid_x, y + h);

        let mut rectangle_path = vg::Path::new();
        let left_x = (mid_x - half_transition).clamp(x, x + w);
        let right_x = (mid_x + half_transition).clamp(x, x + w);
        rectangle_path.move_to(left_x, y);
        rectangle_path.line_to(right_x, y);
        rectangle_path.line_to(right_x, y + h);
        rectangle_path.line_to(left_x, y + h);
        rectangle_path.close();

        let color = cx.font_color();
        let stroke_width = cx.scale_factor() * cx.outline_width();

        // First the rectangle
        canvas.fill_path(
            &rectangle_path,
            &vg::Paint::color(Color::rgba(color.r(), color.g(), color.b(), 127).into()),
        );

        canvas.stroke_path(
            &middle_path,
            &vg::Paint::color(color.into()).with_line_width(stroke_width),
        );
    }
}

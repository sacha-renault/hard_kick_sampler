use nih_plug_vizia::vizia::{prelude::*, vg};

#[derive(Lens)]
pub struct AdsrVizualizer<L: Lens<Target = f32>> {
    attack: L,
    decay: L,
    sustain: L,
    release: L,
}

impl<L: Lens<Target = f32>> AdsrVizualizer<L> {
    pub fn new(cx: &mut Context, attack: L, decay: L, sustain: L, release: L) -> Handle<Self> {
        Self {
            attack,
            decay,
            sustain,
            release,
        }
        .build(cx, |_| {})
        .class("adsr-vizualizer")
    }
}

impl<L: Lens<Target = f32>> View for AdsrVizualizer<L> {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        let attack = self.attack.get(cx);
        let decay = self.decay.get(cx);
        let sustain = self.sustain.get(cx);
        let release = self.release.get(cx);

        // First let's do the blend time (bar on the center)
        let BoundingBox { x, y, w, h } = cx.bounds();

        let mut path = vg::Path::new();
        path.move_to(x, y + h);
        path.line_to(x + attack * w, y);
        path.line_to(x + (attack + decay) * w, y + (1. - sustain) * h);
        path.line_to(x + (attack + decay + release) * w, y + h);

        let mut fill_path = path.clone();
        fill_path.close();

        // First the rectangle
        let color = cx.font_color();
        canvas.fill_path(
            &fill_path,
            &vg::Paint::color(Color::rgba(color.r(), color.g(), color.b(), 127).into()),
        );

        canvas.stroke_path(
            &path,
            &vg::Paint::color(color.into()).with_line_width(cx.scale_factor() * cx.outline_width()),
        );
    }
}

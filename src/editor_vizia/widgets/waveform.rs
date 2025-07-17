use std::cell::RefCell;

use derive_more::Constructor;
use nih_plug::nih_log;
use nih_plug_vizia::vizia::{prelude::*, vg};

/// Mini struct to easy normalize the vlaues
#[derive(Debug, Constructor)]
struct Normalizer {
    x_bound: f32,
    y_bound: f32,
    width: f32,
    half_height: f32,
}

impl Normalizer {
    #[inline]
    fn normalize_x(&self, x: f32) -> f32 {
        self.x_bound + x * self.width
    }

    #[inline]
    fn normalize_y(&self, y: f32) -> f32 {
        self.y_bound + self.half_height - y.clamp(-1., 1.) * self.half_height
    }

    #[inline]
    fn normalize(&self, x: f32, y: f32) -> (f32, f32) {
        (self.normalize_x(x), self.normalize_y(y))
    }
}

/// Static waveform.
///
/// For displaying frequently updating waveform data, use an [`Oscilloscope`]
/// instead.
pub struct WavePlot {
    data: Vec<[f32; 2]>,
    cached_texture: RefCell<Option<vg::ImageId>>,
}

impl WavePlot {
    pub fn new(cx: &mut Context, data: Vec<[f32; 2]>) -> Handle<Self> {
        Self {
            data,
            cached_texture: RefCell::new(None),
        }
        .build(cx, |_| {})
    }

    fn _downsample(_data: &[[f32; 2]], _threshold: f32) -> Vec<[f32; 2]> {
        todo!()
    }

    fn build_waveform_path(&self, normalizer: &Normalizer) -> vg::Path {
        let mut path = vg::Path::new();
        let mut iterator = self.data.iter();

        if let Some(&[mut x, mut y]) = iterator.next() {
            (x, y) = normalizer.normalize(x, y);
            path.move_to(x, y);
        }

        while let Some(&[mut x, mut y]) = iterator.next() {
            (x, y) = normalizer.normalize(x, y);
            path.line_to(x, y);
        }

        nih_log!("Path created!");

        path
    }

    fn create_texture(&self, cx: &DrawContext, canvas: &mut Canvas) -> Option<vg::ImageId> {
        let BoundingBox { x: _, y: _, w, h } = cx.bounds();
        let texture = canvas
            .create_image_empty(
                w as usize,
                h as usize,
                vg::PixelFormat::Rgba8,
                vg::ImageFlags::empty(),
            )
            .ok()?;

        // Render waveform to texture (ONCE)
        canvas.save();
        canvas.set_render_target(vg::RenderTarget::Image(texture));
        canvas.clear_rect(0, 0, w as u32, h as u32, vg::Color::rgba(0, 0, 0, 0));

        // Build and stroke path to texture (expensive, but only once!)
        let normalizer = Normalizer::new(0.0, 0.0, w, h / 2.0);
        canvas.stroke_path(
            &self.build_waveform_path(&normalizer),
            &vg::Paint::color(cx.font_color().into())
                .with_line_width(cx.scale_factor() * cx.outline_width()),
        );

        // Restore render target
        canvas.set_render_target(vg::RenderTarget::Screen);
        canvas.restore();

        // Cache and return
        *self.cached_texture.borrow_mut() = Some(texture);
        Some(texture)
    }

    fn get_cached_texture(&self, cx: &mut DrawContext, canvas: &mut Canvas) -> Option<vg::ImageId> {
        if self.cached_texture.borrow().is_some() {
            self.cached_texture.borrow().clone()
        } else {
            self.create_texture(cx, canvas)
        }
    }
}

impl View for WavePlot {
    fn element(&self) -> Option<&'static str> {
        Some("waveform")
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        if let Some(texture) = self.get_cached_texture(cx, canvas) {
            let mut rect_path = vg::Path::new();
            let BoundingBox { x, y, w, h } = cx.bounds();
            rect_path.rect(x, y, w, h);
            canvas.fill_path(
                &rect_path, // Simple rectangle
                &vg::Paint::image(texture, x, y, w, h, 0.0, 1.0),
            );
            return;
        }

        let bounds = cx.bounds();

        let x_bound = bounds.x;
        let y_bound = bounds.y;
        let width = bounds.w;
        let half_height = bounds.h / 2.0; // half height

        let normalizer = Normalizer::new(x_bound, y_bound, width, half_height);

        // Waveform
        canvas.stroke_path(
            &self.build_waveform_path(&normalizer),
            &vg::Paint::color(cx.font_color().into())
                .with_line_width(cx.scale_factor() * cx.outline_width()),
        );
    }
}

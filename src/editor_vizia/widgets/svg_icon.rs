use nih_plug::nih_error;
use nih_plug::nih_log;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::vizia::vg;

use usvg::{Options, Tree};

pub struct SvgIcon {
    svg_content: String,
}

impl SvgIcon {
    pub fn new<'a>(cx: &'a mut Context, svg_content: impl Into<String>) -> Handle<'a, Self> {
        Self {
            svg_content: svg_content.into(),
        }
        .build(cx, |cx| {
            Element::new(cx);
        })
    }
}

impl View for SvgIcon {
    fn draw(&self, cx: &mut DrawContext, canvas: &mut Canvas) {
        // Parse the SVG using usvg
        let opt = Options::default();
        if let Ok(tree) = Tree::from_str(&self.svg_content, &opt) {
            let bounds = cx.bounds();
            let size = tree.size();

            // Scale to fit bounds
            let scale_x = bounds.width() / size.width();
            let scale_y = bounds.height() / size.height();
            let scale = scale_x.min(scale_y);

            // Calculate centered position
            let scaled_width = size.width() * scale;
            let scaled_height = size.height() * scale;
            let center_x = bounds.x + (bounds.width() - scaled_width) / 2.0;
            let center_y = bounds.y + (bounds.height() - scaled_height) / 2.0;

            canvas.save();
            canvas.translate(center_x, center_y);
            canvas.scale(scale, scale);

            // Render the tree nodes
            for node in tree.root().children() {
                self.render_node(cx, canvas, node);
            }

            canvas.restore();
        } else {
            nih_error!("Couldn't parse svg : {}", self.svg_content);
        }
    }
}

impl SvgIcon {
    fn render_node(&self, cx: &mut DrawContext, canvas: &mut Canvas, node: &usvg::Node) {
        match node {
            usvg::Node::Group(group) => {
                // Render children
                for child in group.children() {
                    self.render_node(cx, canvas, &child);
                }
            }
            usvg::Node::Path(path) => {
                self.render_path(cx, canvas, path.as_ref());
            }
            _ => {} // Don't handle text or image
        }
    }

    fn render_path(&self, cx: &mut DrawContext, canvas: &mut Canvas, path: &usvg::Path) {
        let mut vg_path = vg::Path::new();

        // Convert usvg path to vg path
        for segment in path.data().segments() {
            match segment {
                usvg::tiny_skia_path::PathSegment::MoveTo(pt) => {
                    vg_path.move_to(pt.x, pt.y);
                }
                usvg::tiny_skia_path::PathSegment::LineTo(pt) => {
                    vg_path.line_to(pt.x, pt.y);
                }
                usvg::tiny_skia_path::PathSegment::QuadTo(pt1, pt2) => {
                    vg_path.quad_to(pt1.x, pt1.y, pt2.x, pt2.y);
                }
                usvg::tiny_skia_path::PathSegment::CubicTo(pt1, pt2, pt3) => {
                    vg_path.bezier_to(pt1.x, pt1.y, pt2.x, pt2.y, pt3.x, pt3.y);
                }
                usvg::tiny_skia_path::PathSegment::Close => {
                    vg_path.close();
                }
            }
        }

        // Handle fill
        if let Some(_) = path.fill() {
            let paint = self.convert_paint(cx);
            canvas.fill_path(&vg_path, &paint);
        }

        // Handle stroke
        if let Some(stroke) = path.stroke() {
            let mut paint = self
                .convert_paint(cx)
                .with_line_width(cx.outline_width() * cx.scale_factor());

            // Apply line cap and join
            match stroke.linecap() {
                usvg::LineCap::Butt => paint = paint.with_line_cap(vg::LineCap::Butt),
                usvg::LineCap::Round => paint = paint.with_line_cap(vg::LineCap::Round),
                usvg::LineCap::Square => paint = paint.with_line_cap(vg::LineCap::Square),
            }

            match stroke.linejoin() {
                usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => {
                    paint = paint.with_line_join(vg::LineJoin::Miter);
                }
                usvg::LineJoin::Round => paint = paint.with_line_join(vg::LineJoin::Round),
                usvg::LineJoin::Bevel => paint = paint.with_line_join(vg::LineJoin::Bevel),
            }

            canvas.stroke_path(&vg_path, &paint);
        }
    }

    fn convert_paint(&self, cx: &mut DrawContext) -> vg::Paint {
        vg::Paint::color(cx.font_color().into())
    }
}

// Usage function
pub fn svg_icon<'a>(
    cx: &'a mut Context,
    svg_content: &str,
    size: Units,
    stroke: f32,
) -> Handle<'a, SvgIcon> {
    SvgIcon::new(cx, svg_content)
        .width(size)
        .height(size)
        .outline_width(Pixels(stroke))
}

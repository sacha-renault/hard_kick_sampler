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
            Label::new(cx, "");
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

        // Convert usvg path to femtovg path
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

        // Apply fill if present
        if let Some(_) = path.fill() {
            let paint = self.convert_paint(cx);
            canvas.fill_path(&vg_path, &paint);
        }

        // Apply stroke if present
        if let Some(_) = path.stroke() {
            let paint = self.convert_paint(cx);
            canvas.stroke_path(&vg_path, &paint);
        }
    }

    fn convert_paint(&self, cx: &mut DrawContext) -> vg::Paint {
        vg::Paint::color(cx.font_color().into())
            .with_line_width(cx.outline_width() * cx.scale_factor())
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

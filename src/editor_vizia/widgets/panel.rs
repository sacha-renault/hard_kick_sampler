use nih_plug_vizia::vizia::prelude::*;

use crate::editor_vizia::style::*;

pub struct WidgetPanel;

impl WidgetPanel {
    /// Creates a new widget panel with a title and content
    #[allow(clippy::new_ret_no_self)]
    pub fn new<'a, F>(cx: &'a mut Context, title: &str, content: F) -> Handle<'a, VStack>
    where
        F: FnOnce(&mut Context),
    {
        VStack::new(cx, |cx| {
            // Title
            Label::new(cx, title).class("panel-title");

            // Content area
            HStack::new(cx, content).class("panel-content");
        })
        .child_space(Units::Pixels(PANEL_PADDING))
        .border_radius(Units::Pixels(BORDER_RADIUS))
        .class("widget-panel")
    }

    pub fn vnew<'a, F>(cx: &'a mut Context, title: &str, content: F) -> Handle<'a, VStack>
    where
        F: FnOnce(&mut Context),
    {
        VStack::new(cx, |cx| {
            // Title
            Label::new(cx, title).class("panel-title");

            // Content area
            VStack::new(cx, content).class("panel-content");
        })
        .class("widget-panel")
    }
}

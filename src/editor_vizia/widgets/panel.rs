use nih_plug_vizia::vizia::prelude::*;

use super::super::style::*;

pub struct WidgetPanel;

impl WidgetPanel {
    /// Creates a new widget panel with a title and content
    pub fn new<'a, F>(cx: &'a mut Context, title: &str, content: F) -> Handle<'a, VStack>
    where
        F: FnOnce(&mut Context),
    {
        VStack::new(cx, |cx| {
            // Title
            Label::new(cx, title)
                .color(TEXT_COLOR_ACCENT)
                .class("panel-title");

            // Content area
            HStack::new(cx, content).class("panel-content");
        })
        .class("widget-panel")
    }
}

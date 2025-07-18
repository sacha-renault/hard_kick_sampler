use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

use super::widget_base::*;

/// Icon position for the toggle button
#[derive(Debug, Clone, Copy, Default)]
pub enum IconPosition {
    #[default]
    Left,
    Right,
    None,
}

#[derive(Clone, Default)]
pub struct ButtonToggleBuilder {
    icon: String,
    text: Option<String>,
    icon_position: IconPosition,
}

impl ButtonToggleBuilder {
    /// Sets the icon for the toggle button
    pub fn with_icon(mut self, icon: impl ToString) -> Self {
        self.icon = icon.to_string();
        self
    }

    /// Sets custom text for the toggle button
    /// If not set, uses the parameter name
    pub fn with_text(mut self, text: impl ToString) -> Self {
        self.text = Some(text.to_string());
        self
    }

    /// Sets the icon position to the right of the text
    pub fn icon_right(mut self) -> Self {
        self.icon_position = IconPosition::Right;
        self
    }

    /// Sets the icon position to the left of the text (default)
    pub fn icon_left(mut self) -> Self {
        self.icon_position = IconPosition::Left;
        self
    }

    pub fn no_icon(mut self) -> Self {
        self.icon_position = IconPosition::None;
        self
    }
}

impl ParamWidgetBuilder for ButtonToggleBuilder {
    type Widget = ButtonToggle;
}

#[derive(Lens)]
pub struct ButtonToggle {
    param_base: ParamWidgetBase,
    builder: ButtonToggleBuilder,
}

impl ParamWidget for ButtonToggle {
    type Builder = ButtonToggleBuilder;

    fn param_base(&self) -> &ParamWidgetBase {
        &self.param_base
    }

    fn new_from_builder<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
        builder: ButtonToggleBuilder,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            param_base: ParamWidgetBase::new(cx, params.clone(), params_to_param),
            builder,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
                let current_value_lens = param_data.make_lens(|p| p.modulated_normalized_value());
                let is_checked_lens = current_value_lens.map(|val| *val > 0.5);

                Button::new(
                    cx,
                    move |cx| {
                        let value = if param_data.param().modulated_normalized_value() > 0.5 {
                            0.0
                        } else {
                            1.0
                        };
                        cx.emit(NormalizedParamUpdate(value))
                    },
                    |cx| {
                        // Get builder configuration
                        let builder = ButtonToggle::builder.get(cx);

                        HStack::new(cx, |cx| {
                            // Toggle element
                            if matches!(builder.icon_position, IconPosition::Left if !builder.icon.is_empty()) {
                                Icon::new(cx, &builder.icon).width(Units::Auto);
                            }

                            // Display the current state as text
                            let param_name = param_data.param().name().to_string();
                            let txt = match builder.text {
                                Some(t) => t.clone(),
                                None => param_name.clone(),
                            };
                            Label::new(cx, &txt);

                            if matches!(builder.icon_position, IconPosition::Right if !builder.icon.is_empty()) {
                                Icon::new(cx, &builder.icon).width(Units::Auto);
                            }
                        })
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .col_between(Units::Pixels(4.0))
                        .width(Units::Auto)
                        .height(Units::Auto)
                    },
                )
                .checkable(true)
                .checked(is_checked_lens)
                .class("toggle-container");
            }),
        )
    }
}

impl View for ButtonToggle {
    fn element(&self) -> Option<&'static str> {
        Some("param-toggle")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        self.handle_param_event(cx, event);
    }
}

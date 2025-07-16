use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Lens)]
pub struct ParamSwitch {
    param_base: ParamWidgetBase,
}

impl ParamSwitch {
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            param_base: ParamWidgetBase::new(cx, params, params_to_param),
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
                let current_value_lens = param_data.make_lens(|p| p.modulated_normalized_value());
                let is_checked_lens = current_value_lens.map(|val| *val > 0.5);

                HStack::new(cx, |cx| {
                    // Toggle element
                    // Main toggle track
                    Switch::new(cx, is_checked_lens).on_toggle(|cx| {
                        cx.emit(ToggleChangeEvent);
                    });

                    // Display the current state as text
                    Label::new(cx, param_data.param().name())
                        .toggle_class("active", is_checked_lens)
                        .on_press(|cx| {
                            cx.emit(ToggleChangeEvent);
                        });
                })
                .child_top(Stretch(1.0))
                .child_bottom(Stretch(1.0))
                .col_between(Units::Pixels(2.0))
                .width(Units::Auto) // ← ADD THIS
                .height(Units::Auto)
                .checkable(true)
                .checked(is_checked_lens)
                .on_press(|cx| {
                    cx.emit(ToggleChangeEvent);
                })
                .class("switch-container");
            }),
        )
    }
}

struct ToggleChangeEvent;

impl View for ParamSwitch {
    fn element(&self) -> Option<&'static str> {
        Some("param-toggle")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|_toggle_event: &ToggleChangeEvent, meta| {
            self.param_base.begin_set_parameter(cx);

            // Get current normalized value and flip it
            let current_value = self.param_base.modulated_normalized_value();
            let new_value = if current_value > 0.5 { 0.0 } else { 1.0 };

            self.param_base.set_normalized_value(cx, new_value);
            self.param_base.end_set_parameter(cx);
            meta.consume();
        });
    }
}

use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Lens)]
pub struct ButtonToggle {
    param_base: ParamWidgetBase,
    text: Option<String>,
}

impl ButtonToggle {
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        icon: String,
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
            text: None,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
                let current_value_lens = param_data.make_lens(|p| p.modulated_normalized_value());
                let is_checked_lens = current_value_lens.map(|val| *val > 0.5);

                Button::new(
                    cx,
                    |cx| cx.emit(ToggleChangeEvent),
                    |cx| {
                        HStack::new(cx, |cx| {
                            // Toggle element
                            Icon::new(cx, &icon).width(Units::Auto);

                            // Display the current state as text
                            let param_name = param_data.param().name().to_string();
                            let txt_lens = ButtonToggle::text.map(move |text| -> String {
                                match text {
                                    Some(t) => t.clone(),
                                    None => param_name.clone(),
                                }
                            });
                            Label::new(cx, txt_lens);
                        })
                        .child_top(Stretch(1.0))
                        .child_bottom(Stretch(1.0))
                        .col_between(Units::Pixels(2.0))
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

struct ToggleChangeEvent;

impl View for ButtonToggle {
    fn element(&self) -> Option<&'static str> {
        Some("param-toggle")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|_: &ToggleChangeEvent, meta| {
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

use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::{ParamWidgetBase, ParamWidgetData};

#[derive(Lens)]
pub struct ParamRadio {
    param_base: ParamWidgetBase,
}

impl ParamRadio {
    pub fn vertical<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
        display_param_name: bool,
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
                VStack::new(cx, |cx| {
                    if display_param_name {
                        Label::new(cx, param_data.param().name());
                    }

                    VStack::new(cx, |cx| {
                        Self::content(cx, param_data);
                    });
                })
                .class("radio-container");
            }),
        )
    }

    pub fn horizontal<L, Params, P, FMap>(
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
                VStack::new(cx, |cx| {
                    Label::new(cx, param_data.param().name());

                    HStack::new(cx, |cx| {
                        Self::content(cx, param_data);
                    })
                    .child_space(Stretch(1.0))
                    .col_between(Pixels(10.0));
                })
                .class("radio-container")
                .child_space(Stretch(1.0))
                .row_between(Pixels(5.0))
                .col_between(Pixels(0.0));
            }),
        )
    }

    fn content<L, Params, P, FMap>(
        cx: &mut Context,
        param_data: ParamWidgetData<L, Params, P, FMap>,
    ) where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        let current_value_lens = param_data.make_lens(|p| p.modulated_normalized_value());

        for step in 0..param_data.param().step_count().unwrap_or(1) + 1 {
            let normalized_value =
                step as f32 / param_data.param().step_count().unwrap_or(1) as f32;
            let display_value = param_data
                .param()
                .normalized_value_to_string(normalized_value, true);

            HStack::new(cx, |cx| {
                RadioButton::new(
                    cx,
                    current_value_lens
                        .map(move |val| (*val - normalized_value).abs() < f32::EPSILON),
                )
                .on_select(move |cx| {
                    cx.emit(RadioChangeEvent(normalized_value));
                });

                Label::new(cx, &display_value).on_press(move |cx| {
                    cx.emit(RadioChangeEvent(normalized_value));
                });
            })
            .child_top(Stretch(1.0))
            .child_bottom(Stretch(1.0))
            .col_between(Pixels(5.0));
        }
    }
}

struct RadioChangeEvent(f32);

impl View for ParamRadio {
    fn element(&self) -> Option<&'static str> {
        Some("param-radio")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|radio_event: &RadioChangeEvent, meta| {
            self.param_base.begin_set_parameter(cx);
            self.param_base.set_normalized_value(cx, radio_event.0);
            self.param_base.end_set_parameter(cx);
            meta.consume();
        });
    }
}

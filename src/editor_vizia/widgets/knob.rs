use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Lens)]
pub struct ParamKnob {
    param_base: ParamWidgetBase,
}

impl ParamKnob {
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
            param_base: ParamWidgetBase::new(cx, params.clone(), params_to_param),
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
                // Don't return the VStack, just create it
                VStack::new(cx, |cx| {
                    let lens = param_data.make_lens(|p| p.modulated_normalized_value());

                    Label::new(cx, param_data.param().name());

                    Knob::new(
                        cx,
                        param_data.param().default_normalized_value(),
                        param_data.make_lens(|p| p.modulated_normalized_value()),
                        false,
                    )
                    .on_changing(|cx, val| {
                        cx.emit(KnobChangeEvent(val));
                    });

                    Label::new(
                        cx,
                        lens.map(move |val| {
                            param_data.param().normalized_value_to_string(*val, true)
                        }),
                    );
                }); // No return here, the semicolon makes it return ()
            }),
        )
    }
}

struct KnobChangeEvent(f32);

impl View for ParamKnob {
    fn element(&self) -> Option<&'static str> {
        Some("param-knob")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|knob_event: &KnobChangeEvent, meta| {
            self.param_base.begin_set_parameter(cx);
            self.param_base.set_normalized_value(cx, knob_event.0);
            self.param_base.end_set_parameter(cx);
            meta.consume();
        });
    }
}

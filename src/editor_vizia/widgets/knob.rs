use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Lens)]
pub struct ParamKnob {
    param_base: ParamWidgetBase,
    label: Option<String>,
    show_label: bool,
    show_value: bool,
    centered: bool,
    custom_class: Option<String>,
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
            label: None,
            show_label: true,
            show_value: true,
            custom_class: None,
            centered: false,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                VStack::new(cx, |cx| {
                    let lens = param_data.make_lens(|p| p.modulated_normalized_value());

                    // Get the current ParamKnob instance to access its properties
                    let show_label = ParamKnob::show_label.get(cx);
                    let show_value = ParamKnob::show_value.get(cx);
                    let centered = ParamKnob::centered.get(cx);
                    let text = ParamKnob::label
                        .get(cx)
                        .unwrap_or(param_data.param().name().to_string());

                    if show_label {
                        Label::new(cx, &text);
                    }

                    Knob::new(
                        cx,
                        param_data.param().default_normalized_value(),
                        param_data.make_lens(|p| p.modulated_normalized_value()),
                        centered,
                    )
                    .on_changing(|cx, val| {
                        cx.emit(KnobChangeEvent(val));
                    });

                    if show_value {
                        Label::new(
                            cx,
                            lens.map(move |val| {
                                param_data.param().normalized_value_to_string(*val, true)
                            }),
                        );
                    }
                })
                .class("knob-container")
                .child_space(Stretch(1.0))
                .row_between(Stretch(1.0));
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

pub trait ParamKnobModifiers {
    fn with_label(self, label: impl ToString) -> Self;
    fn without_label(self) -> Self;
    fn without_value(self) -> Self;
    fn with_class(self, class: impl ToString) -> Self;
    fn with_centered(self, centered: bool) -> Self;
}

impl ParamKnobModifiers for Handle<'_, ParamKnob> {
    fn with_label(self, label: impl ToString) -> Self {
        self.modify(|knob: &mut ParamKnob| {
            knob.label = Some(label.to_string());
        })
    }

    fn without_label(self) -> Self {
        self.modify(|knob: &mut ParamKnob| {
            knob.show_label = false;
        })
    }

    fn without_value(self) -> Self {
        self.modify(|knob: &mut ParamKnob| {
            knob.show_value = false;
        })
    }

    fn with_centered(self, centered: bool) -> Self {
        self.modify(|knob: &mut ParamKnob| {
            knob.centered = centered;
        })
    }

    fn with_class(self, class: impl ToString) -> Self {
        self.modify(|knob: &mut ParamKnob| {
            knob.custom_class = Some(class.to_string());
        })
        .class(&class.to_string())
    }
}

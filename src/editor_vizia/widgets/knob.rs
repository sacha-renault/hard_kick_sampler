use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Clone, Default)]
pub struct ParamKnobBuilder {
    centered: bool,
    label: Option<String>,
    hide_label: bool,
    hide_value: bool,
}

impl ParamKnobBuilder {
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    pub fn with_label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn hide_label(mut self) -> Self {
        self.hide_label = true;
        self
    }

    pub fn hide_value(mut self) -> Self {
        self.hide_value = true;
        self
    }

    pub fn build<L, Params, P, FMap>(
        self,
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
    ) -> Handle<ParamKnob>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        ParamKnob {
            param_base: ParamWidgetBase::new(cx, params.clone(), params_to_param),
            modifiers: self,
        }
        .inner_build(cx, params, params_to_param)
    }
}

#[derive(Lens)]
pub struct ParamKnob {
    param_base: ParamWidgetBase,
    modifiers: ParamKnobBuilder,
}

impl ParamKnob {
    pub fn builder() -> ParamKnobBuilder {
        ParamKnobBuilder::default()
    }

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
            modifiers: ParamKnobBuilder::default(),
        }
        .inner_build(cx, params, params_to_param)
    }

    fn inner_build<L, Params, P, FMap>(
        self,
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
        self.build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, move |cx, param_data| {
                VStack::new(cx, |cx| {
                    let lens = param_data.make_lens(|p| p.modulated_normalized_value());

                    // Get the current ParamKnob instance to access its properties
                    let modifiers = ParamKnob::modifiers.get(cx);
                    let text = modifiers
                        .label
                        .unwrap_or(param_data.param().name().to_string());

                    if !modifiers.hide_label {
                        Label::new(cx, &text);
                    }

                    Knob::new(
                        cx,
                        param_data.param().default_normalized_value(),
                        param_data.make_lens(|p| p.modulated_normalized_value()),
                        modifiers.centered,
                    )
                    .on_changing(|cx, val| {
                        cx.emit(KnobChangeEvent(val));
                    });

                    if !modifiers.hide_value {
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

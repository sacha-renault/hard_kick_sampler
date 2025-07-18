use nih_plug::params::Param;
use nih_plug_vizia::{
    vizia::{
        binding::Lens,
        context::{Context, EventContext},
        events::Event,
        view::Handle,
    },
    widgets::param_base::ParamWidgetBase,
};

pub trait ParamWidgetBuilder: Default {
    type Widget: ParamWidget<Builder = Self>;

    fn build<L, Params, P, FMap>(
        self,
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
    ) -> Handle<Self::Widget>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self::Widget::new_from_builder(cx, params, params_to_param, self)
    }
}

pub trait ParamWidget: Sized {
    type Builder: ParamWidgetBuilder<Widget = Self>;

    fn builder() -> Self::Builder {
        Self::Builder::default()
    }

    fn new<L, Params, P, FMap>(cx: &mut Context, params: L, params_to_param: FMap) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self::new_from_builder(cx, params, params_to_param, Self::Builder::default())
    }

    fn new_from_builder<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
        builder: Self::Builder,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static;

    fn param_base(&self) -> &ParamWidgetBase;

    fn handle_param_event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event: &NormalizedParamUpdate, meta| {
            self.param_base().begin_set_parameter(cx);
            self.param_base().set_normalized_value(cx, event.0);
            self.param_base().end_set_parameter(cx);
            meta.consume();
        });
    }
}

pub struct NormalizedParamUpdate(pub f32);

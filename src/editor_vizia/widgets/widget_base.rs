//! Parameter widget framework for audio plugins using nih-plug and Vizia.
//!
//! This module provides a consistent builder pattern and event handling system
//! for creating parameter widgets in audio plugin UIs.
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

/// Builder trait for parameter widgets.
///
/// This trait provides a consistent way to build parameter widgets with
/// customizable options using the builder pattern.
///
/// # Associated Types
///
/// * `Widget` - The parameter widget type that this builder creates
///
/// # Example Implementation
///
/// ```rust
/// #[derive(Clone, Default)]
/// pub struct ParamKnobBuilder {
///     centered: bool,
/// }
///
/// impl ParamWidgetBuilder for ParamKnobBuilder {
///     type Widget = ParamKnob;
/// }
/// ```
pub trait ParamWidgetBuilder: Default {
    /// The parameter widget type that this builder creates.
    type Widget: ParamWidget<Builder = Self>;

    /// Builds the parameter widget with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `cx` - The Vizia context
    /// * `params` - Lens to the parameter struct
    /// * `params_to_param` - Function to extract the specific parameter
    ///
    /// # Type Parameters
    ///
    /// * `L` - Lens type for accessing parameters
    /// * `Params` - The parameter struct type
    /// * `P` - The specific parameter type
    /// * `FMap` - Function type for parameter extraction
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

/// Core trait for parameter widgets.
///
/// This trait defines the interface for widgets that control audio parameters.
/// It provides both simple construction and builder-based construction methods.
///
/// # Associated Types
///
/// * `Builder` - The builder type for this widget
///
/// # Example Implementation
///
/// ```rust
/// impl ParamWidget for ParamKnob {
///     type Builder = ParamKnobBuilder;
///
///     fn param_base(&self) -> &ParamWidgetBase {
///         &self.param_base
///     }
///
///     fn new_from_builder<L, Params, P, FMap>(
///         cx: &mut Context,
///         params: L,
///         params_to_param: FMap,
///         builder: Self::Builder,
///     ) -> Handle<Self> {
///         // Implementation here
///     }
/// }
/// ```
pub trait ParamWidget: Sized {
    /// The builder type for this parameter widget.
    type Builder: ParamWidgetBuilder<Widget = Self>;

    /// Creates a new builder for this parameter widget.
    ///
    /// # Example
    ///
    /// ```rust
    /// ParamKnob::builder()
    ///     .centered()
    ///     .build(cx, params, |p| &p.gain);
    /// ```
    fn builder() -> Self::Builder {
        Self::Builder::default()
    }

    /// Creates a parameter widget with default settings.
    ///
    /// This is a convenience method equivalent to calling `builder().build()`.
    ///
    /// # Arguments
    ///
    /// * `cx` - The Vizia context
    /// * `params` - Lens to the parameter struct
    /// * `params_to_param` - Function to extract the specific parameter
    ///
    /// # Example
    ///
    /// ```rust
    /// ParamKnob::new(cx, params, |p| &p.gain);
    /// ```
    fn new<L, Params, P, FMap>(cx: &mut Context, params: L, params_to_param: FMap) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self::new_from_builder(cx, params, params_to_param, Self::Builder::default())
    }

    /// Handles parameter update events.
    ///
    /// This method provides default event handling for parameter updates.
    /// It automatically manages parameter changes and host communication.
    ///
    /// Call this from your widget's `View::event` implementation:
    ///
    /// ```rust
    /// impl View for ParamKnob {
    ///     fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
    ///         self.handle_param_event(cx, event);
    ///         // Handle other events here...
    ///     }
    /// }
    /// ```
    fn handle_param_event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|event: &NormalizedParamUpdate, meta| {
            self.param_base().begin_set_parameter(cx);
            self.param_base().set_normalized_value(cx, event.0);
            self.param_base().end_set_parameter(cx);
            meta.consume();
        });
    }

    /// Creates a parameter widget from a builder configuration.
    ///
    /// This method is called internally by the builder's `build()` method.
    /// Implement this to handle the actual widget construction.
    ///
    /// # Arguments
    ///
    /// * `cx` - The Vizia context
    /// * `params` - Lens to the parameter struct
    /// * `params_to_param` - Function to extract the specific parameter
    /// * `builder` - The configured builder instance
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

    // Returns a reference to the underlying parameter base.
    ///
    /// This is used for parameter automation and host communication.
    fn param_base(&self) -> &ParamWidgetBase;
}

/// Event sent when a parameter widget's value changes.
///
/// This event contains the normalized parameter value (0.0 to 1.0)
/// and is handled automatically by the `ParamWidget::handle_param_event` method.
///
/// # Fields
///
/// * `0` - The normalized parameter value (0.0 to 1.0)
///
/// # Example
///
/// ```rust
/// // Emit this event when your widget's value changes
/// cx.emit(NormalizedParamUpdate(0.75));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NormalizedParamUpdate(pub f32);

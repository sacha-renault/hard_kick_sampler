use nih_plug::prelude::*;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::param_base::ParamWidgetBase;

#[derive(Lens)]
pub struct ParamDragNumber {
    param_base: ParamWidgetBase,
    drag_start_y: f32,
    drag_start_value: f32,
    is_dragging: bool,
}

impl ParamDragNumber {
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
            drag_start_y: 0.0,
            drag_start_value: 0.0,
            is_dragging: false,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params, params_to_param, |cx, param_data| {
                let current_value_lens = param_data.make_lens(|p| p.modulated_normalized_value());

                // Display the current value as formatted text
                Label::new(
                    cx,
                    current_value_lens
                        .map(move |val| param_data.param().normalized_value_to_string(*val, true)),
                )
                .class("drag-number-input")
                .on_double_click(|cx, _| cx.emit(ResetEvent))
                .focusable(true);
            }),
        )
    }
}

struct ResetEvent;

impl View for ParamDragNumber {
    fn element(&self) -> Option<&'static str> {
        Some("param-drag-number")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|_reset_event: &ResetEvent, meta| {
            self.param_base.begin_set_parameter(cx);

            // Get current normalized value and flip it
            let default = self.param_base.default_normalized_value();
            self.param_base.set_normalized_value(cx, default);
            self.param_base.end_set_parameter(cx);
            meta.consume();
        });
        event.map(|window_event, meta| match window_event {
            WindowEvent::MouseDown(mouse_button) => {
                if *mouse_button == MouseButton::Left {
                    self.is_dragging = true;
                    self.drag_start_y = cx.mouse().cursory;
                    self.drag_start_value = self.param_base.modulated_normalized_value();
                    cx.capture();
                    cx.set_active(true);
                    meta.consume();
                }
            }
            WindowEvent::MouseMove(_, y) => {
                if self.is_dragging {
                    let delta_y = self.drag_start_y - *y;
                    let sensitivity = 0.0035;

                    // Calculate new value based on drag distance
                    let new_value = (self.drag_start_value + delta_y * sensitivity).clamp(0.0, 1.0);

                    self.param_base.begin_set_parameter(cx);
                    self.param_base.set_normalized_value(cx, new_value);
                    self.param_base.end_set_parameter(cx);

                    meta.consume();
                }
            }
            WindowEvent::MouseUp(mouse_button) => {
                if *mouse_button == MouseButton::Left && self.is_dragging {
                    self.is_dragging = false;
                    cx.release();
                    cx.set_active(false);
                    meta.consume();
                }
            }
            WindowEvent::MouseScroll(_, y) => {
                if *y != 0.0 {
                    self.param_base.begin_set_parameter(cx);

                    let current_value = self.param_base.modulated_normalized_value();

                    // Get the step size (1 step in normalized space)
                    let step_size = if let Some(step_count) = self.param_base.step_count() {
                        1.0 / step_count as f32
                    } else {
                        0.01 // Default step for continuous parameters
                    };

                    // Scroll up = positive y = increase value
                    let new_value = if *y > 0.0 {
                        (current_value + step_size).min(1.0)
                    } else {
                        (current_value - step_size).max(0.0)
                    };

                    self.param_base.set_normalized_value(cx, new_value);
                    self.param_base.end_set_parameter(cx);
                    meta.consume();
                }
            }
            _ => {}
        });
    }
}

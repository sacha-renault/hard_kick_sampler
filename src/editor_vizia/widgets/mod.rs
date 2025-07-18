pub mod drag_to_change;
pub mod icon_button_toggle;
pub mod knob;
pub mod panel;
pub mod radio;
pub mod switch;
pub mod waveform;

pub use {
    drag_to_change::ParamDragNumber, icon_button_toggle::ButtonToggle, knob::ParamKnob,
    panel::WidgetPanel, radio::ParamRadio, switch::ParamSwitch, waveform::StaticWavePlot,
};

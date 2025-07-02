use std::sync::Arc;

use nih_plug::prelude::*;
use nih_plug_iced::*;

use crate::params::HardKickSamplerParams;

pub fn create(
    editor_state: Arc<IcedState>,
    params: Arc<HardKickSamplerParams>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<HardKickSamplerEditor>(editor_state, params)
}

pub struct HardKickSamplerEditor {
    params: Arc<HardKickSamplerParams>,
    context: Arc<dyn GuiContext>,
}

impl IcedEditor for HardKickSamplerEditor {
    type Executor = executor::Default;
    type Message = ();
    type InitializationFlags = Arc<HardKickSamplerParams>;

    fn new(
        initialization_fags: Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, Command<Self::Message>) {
        let editor = Self {
            params: initialization_fags,
            context,
        };
        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        _window: &mut WindowQueue,
        _message: Self::Message,
    ) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new().into()
    }
}

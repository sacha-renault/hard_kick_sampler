use std::path::PathBuf;
use std::sync::Arc;

use nih_plug::{editor::Editor, prelude::AsyncExecutor};
use nih_plug_egui::{create_egui_editor, EguiState};

use crate::params::HardKickSamplerParams;
use crate::plugin::HardKickSampler;
use crate::tasks::TaskIn;

pub fn create_editor(
    params: Arc<HardKickSamplerParams>,
    async_executor: AsyncExecutor<HardKickSampler>,
) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        EguiState::from_size(800, 600),
        params.clone(),
        |_ctx, _params| {},
        move |ctx, _setter, _state| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Enable drag and drop
                ctx.input(|i| {
                    if !i.raw.dropped_files.is_empty() {
                        for file in &i.raw.dropped_files {
                            if let Some(path) = &file.path {
                                async_executor.execute_background(TaskIn::LoadFile(0, path.clone()));
                            }
                        }
                    }
                });

                if ui.button("Click").clicked() {
                    async_executor.execute_background(TaskIn::LoadFile(5, PathBuf::from(r"C:\Program Files\Image-Line\FL Studio 21\Data\Patches\Packs\Custom pack\OPS\OPS - Euphoric Hardstyle Kick Expansion (Vol. 1)\Kick Build Folder\Crunches\OPS_ECHKE1_CRUNCH_3_F.wav")));
                }
            });
        },
    )
}

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText, TextureId},
    EguiContexts, EguiTextureHandle,
};

use crate::{message_display, AvailableKeybinds, PathBuf};

use super::editor_panel_frame;

const BOTTOM_PANEL_HEIGHT: f32 = 30.0;

#[derive(Resource, Default)]
pub struct BottomPanel {
    kb_icon: Option<TextureId>,
}

impl BottomPanel {
    fn init_kb_icon(contexts: &mut EguiContexts, asset_server: &AssetServer) -> TextureId {
        contexts.add_image(EguiTextureHandle::Strong(
            asset_server.load(PathBuf::from("textures/menus/keyboard_tip_icon.png")),
        ))
    }

    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        bottom_bar: &message_display::MessageDisplay,
        available_keybinds: &AvailableKeybinds,
        asset_server: &AssetServer,
    ) -> Result {
        let tex_id = match self.kb_icon {
            Some(id) => id,
            None => {
                let id = Self::init_kb_icon(contexts, asset_server);
                self.kb_icon = Some(id);
                id
            }
        };

        let panel_height = BOTTOM_PANEL_HEIGHT;
        let message_string = &bottom_bar.displayed;

        let ctx = contexts.ctx_mut()?;
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(editor_panel_frame())
            .resizable(false)
            .default_height(panel_height)
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.add(egui::Label::new(
                        RichText::new(message_string)
                            .strong()
                            .size(18.0)
                            .color(egui::Color32::from_rgb(220, 230, 245)),
                    ));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        for item in &available_keybinds.keybinds {
                            item.show(ui, tex_id);
                        }
                    });
                });
            });

        Ok(())
    }
}

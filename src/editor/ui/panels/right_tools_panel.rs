use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText, TextureId},
    EguiContexts, EguiTextureHandle,
};

use crate::{
    mouse_state::{MouseState, MouseToolKind},
    EditorState, PathBuf,
};

use super::editor_panel_frame;

#[derive(Resource, Default)]
pub struct RightToolsPanel {
    pointer_icon: Option<TextureId>,
    pencil_icon: Option<TextureId>,
    eraser_icon: Option<TextureId>,
    eyedropper_icon: Option<TextureId>,
    highlight_icon: Option<TextureId>,
}

impl RightToolsPanel {
    fn init_icons(&mut self, contexts: &mut EguiContexts, asset_server: &AssetServer) {
        if self.pointer_icon.is_none() {
            self.pointer_icon = Some(contexts.add_image(EguiTextureHandle::Strong(
                asset_server.load(PathBuf::from("textures/editor/icons/tool_pointer.png")),
            )));
        }

        if self.pencil_icon.is_none() {
            self.pencil_icon = Some(contexts.add_image(EguiTextureHandle::Strong(
                asset_server.load(PathBuf::from("textures/editor/icons/tool_pencil.png")),
            )));
        }

        if self.eraser_icon.is_none() {
            self.eraser_icon = Some(contexts.add_image(EguiTextureHandle::Strong(
                asset_server.load(PathBuf::from("textures/editor/icons/tool_eraser.png")),
            )));
        }

        if self.eyedropper_icon.is_none() {
            self.eyedropper_icon = Some(contexts.add_image(EguiTextureHandle::Strong(
                asset_server.load(PathBuf::from("textures/editor/icons/tool_eyedrop.png")),
            )));
        }

        if self.highlight_icon.is_none() {
            self.highlight_icon = Some(contexts.add_image(EguiTextureHandle::Strong(
                asset_server.load(PathBuf::from("textures/editor/icons/hl_tool.png")),
            )));
        }
    }

    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        asset_server: &AssetServer,
        editor_state: &EditorState,
        mut mouse_state: ResMut<MouseState>,
    ) -> Result {
        if !matches!(editor_state, EditorState::Normal | EditorState::Editing(_)) {
            return Ok(());
        }

        self.init_icons(contexts, asset_server);

        let Some(pointer_icon) = self.pointer_icon else {
            return Ok(());
        };
        let Some(pencil_icon) = self.pencil_icon else {
            return Ok(());
        };
        let Some(eraser_icon) = self.eraser_icon else {
            return Ok(());
        };
        let Some(eyedropper_icon) = self.eyedropper_icon else {
            return Ok(());
        };
        let Some(highlight_icon) = self.highlight_icon else {
            return Ok(());
        };

        let ctx = contexts.ctx_mut()?;

        let mut draw_tool = |ui: &mut egui::Ui, tool_kind: MouseToolKind, icon: TextureId| {
            let size = egui::vec2(40.0, 40.0);
            let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

            ui.painter().rect_filled(
                rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            );

            if mouse_state.mode == tool_kind {
                ui.painter().image(
                    highlight_icon,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }

            ui.painter().image(
                icon,
                rect.shrink2(egui::vec2(4.0, 4.0)),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            if response.clicked() {
                mouse_state.mode = tool_kind;
            }
        };

        egui::SidePanel::right("tool_sidebar")
            .frame(editor_panel_frame())
            .resizable(false)
            .default_width(64.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(
                        RichText::new("Tools")
                            .strong()
                            .size(16.0)
                            .color(egui::Color32::from_rgba_unmultiplied(220, 230, 245, 255)),
                    );
                    ui.separator();
                    draw_tool(ui, MouseToolKind::Pointer, pointer_icon);
                    ui.add_space(6.0);
                    draw_tool(ui, MouseToolKind::Pencil, pencil_icon);
                    ui.add_space(6.0);
                    draw_tool(ui, MouseToolKind::Eraser, eraser_icon);
                    ui.add_space(6.0);
                    draw_tool(ui, MouseToolKind::Eyedropper, eyedropper_icon);
                });
            });

        Ok(())
    }
}

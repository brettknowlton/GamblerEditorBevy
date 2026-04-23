use bevy::prelude::*;
use bevy_egui::egui::{self, RichText};

use crate::{EditorState, editor_object::EditorObjectKind};

#[derive(Resource, Default)]
pub struct ModeTabsPanel;

impl ModeTabsPanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        editor_state: &EditorState,
        next_state: &mut NextState<EditorState>,
        panel_right_x: f32,
    ) {
        if !matches!(editor_state, EditorState::Normal | EditorState::Editing(_)) {
            return;
        }

        let tab_x = panel_right_x.max(0.0);
        let tab_y = 0.0;

        egui::Area::new("mode_tabs_area".into())
            .fixed_pos(egui::pos2(tab_x, tab_y))
            .interactable(true)
            .show(ctx, |ui| {
                egui::Frame {
                    fill: egui::Color32::from_rgba_unmultiplied(18, 22, 30, 165),
                    stroke: egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_unmultiplied(220, 230, 245, 120),
                    ),
                    inner_margin: egui::Margin::same(4),
                    ..Default::default()
                }
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        let tile_active =
                            matches!(editor_state, EditorState::Editing(EditorObjectKind::Tile));
                        let collider_active =
                            matches!(editor_state, EditorState::Editing(EditorObjectKind::Collider));
                        let actor_active =
                            matches!(editor_state, EditorState::Editing(EditorObjectKind::Actor));

                        let tab = |ui: &mut egui::Ui, label: &str, active: bool| {
                            let mut button = egui::Button::new(RichText::new(label).size(14.0));
                            button = button.min_size(egui::vec2(80.0, 24.0));
                            if active {
                                button = button
                                    .fill(egui::Color32::from_rgba_unmultiplied(245, 230, 120, 40))
                                    .stroke(egui::Stroke::new(
                                        1.0,
                                        egui::Color32::from_rgb(245, 230, 120),
                                    ));
                            }
                            ui.add(button)
                        };

                        if tab(ui, "1: Tile", tile_active).clicked() {
                            next_state.set(EditorState::Editing(EditorObjectKind::Tile));
                        }
                        if tab(ui, "2: Collider", collider_active).clicked() {
                            next_state.set(EditorState::Editing(EditorObjectKind::Collider));
                        }
                        if tab(ui, "3: Actor", actor_active).clicked() {
                            next_state.set(EditorState::Editing(EditorObjectKind::Actor));
                        }
                    });
                });
            });
    }
}

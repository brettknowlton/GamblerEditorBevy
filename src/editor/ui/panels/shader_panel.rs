use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContexts,
};

use crate::{
    rendering::{PixelArtSettings, PixelEffectParams},
    EditorState,
};

use super::editor_panel_frame;

#[derive(Resource, Default)]
pub struct ShaderPanel;

#[derive(Resource, Default)]
pub struct SceneRedrawHint {
    pub required: bool,
    pub request_redraw: bool,
}

impl ShaderPanel {
    fn draw_effect_group(ui: &mut egui::Ui, label: &str, params: &mut PixelEffectParams) {
        ui.collapsing(label, |ui| {
            ui.add(
                egui::Slider::new(&mut params.pixel_size, 1.0..=24.0)
                    .text("Pixel Size")
                    .clamping(egui::SliderClamping::Always),
            );
            ui.add(
                egui::Slider::new(&mut params.color_levels, 2.0..=32.0)
                    .text("Color Levels")
                    .clamping(egui::SliderClamping::Always),
            );
            ui.add(
                egui::Slider::new(&mut params.dither_strength, 0.0..=1.0)
                    .text("Dither")
                    .clamping(egui::SliderClamping::Always),
            );
            ui.add(
                egui::Slider::new(&mut params.scanline_strength, 0.0..=1.0)
                    .text("Scanline")
                    .clamping(egui::SliderClamping::Always),
            );
            ui.add(
                egui::Slider::new(&mut params.palette_enabled, 0.0..=1.0)
                    .text("Palette Mix")
                    .clamping(egui::SliderClamping::Always),
            );
        });
    }

    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        editor_state: &EditorState,
        settings: &mut PixelArtSettings,
        redraw_hint: &mut SceneRedrawHint,
    ) -> Result {
        if matches!(editor_state, EditorState::Inactive) {
            return Ok(());
        }

        let ctx = contexts.ctx_mut()?;

        egui::Window::new("Pixel Shader")
            .default_open(true)
            .resizable(true)
            .frame(editor_panel_frame())
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-78.0, 16.0))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Live controls for tile + player shader passes")
                        .italics()
                        .size(11.0)
                        .color(egui::Color32::from_gray(200)),
                );

                if redraw_hint.required {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("! Scene re-draw recommended (Ctrl + D)")
                            .strong()
                            .color(egui::Color32::from_rgb(255, 210, 105)),
                    );
                }

                if ui.button("Re-draw Scene (Ctrl + D)").clicked() {
                    redraw_hint.request_redraw = true;
                }
                ui.separator();

                let before_tile = settings.tile;
                let before_player = settings.player;

                Self::draw_effect_group(ui, "Tiles", &mut settings.tile);
                ui.separator();
                Self::draw_effect_group(ui, "Player", &mut settings.player);

                if before_tile.pixel_size != settings.tile.pixel_size
                    || before_tile.color_levels != settings.tile.color_levels
                    || before_tile.dither_strength != settings.tile.dither_strength
                    || before_tile.scanline_strength != settings.tile.scanline_strength
                    || before_tile.palette_enabled != settings.tile.palette_enabled
                    || before_player.pixel_size != settings.player.pixel_size
                    || before_player.color_levels != settings.player.color_levels
                    || before_player.dither_strength != settings.player.dither_strength
                    || before_player.scanline_strength != settings.player.scanline_strength
                    || before_player.palette_enabled != settings.player.palette_enabled
                {
                    redraw_hint.required = true;
                }
            });

        Ok(())
    }
}

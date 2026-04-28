use bevy::input::mouse::MouseWheel;
use bevy::input::{keyboard::KeyCode, ButtonInput};
use bevy::prelude::*;
use bevy_egui::egui::{self, TextureId};
use bevy_egui::EguiContexts;
use bevy_egui::EguiTextureHandle;

use crate::direction::Direction;
use crate::ui::panels::editor_panel_frame;
use crate::{
    editor_modes::EditorObjectKind, EditorState, TextureHandles, ToolingMenuItem, ToolingMenuState,
    MAX_SPRITESHEET_ITEMS, SPRITESHEET_WIDTH, TILE_SIZE,
};

const PANEL_MARGIN_PX: f32 = 24.0;

const DEFAULT_PANEL_WIDTH: f32 = 220.0;

#[derive(Resource)]
pub struct LeftPanel {
    items: Vec<ToolingMenuItem>,
    next_selected_id: Option<u64>,
    current_index: usize,
    tile_texture_id: Option<TextureId>,
    num_columns: usize,
    panel_width: f32,
    tile_spacing: f32,
    tile_button_px: f32,
}
impl Default for LeftPanel {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_selected_id: None,
            current_index: 0,
            tile_texture_id: None,
            num_columns: 1,
            panel_width: 0.0,
            tile_spacing: 4.0,
            tile_button_px: TILE_SIZE as f32,
        }
    }
}

impl LeftPanel {
    fn tooling_columns(is_tile_mode: bool) -> usize {
        if is_tile_mode {
            (SPRITESHEET_WIDTH as usize / 2).max(1)
        } else {
            1
        }
    }

    fn move_selection_index(
        input: &ButtonInput<KeyCode>,
        mwheel: &mut MessageReader<MouseWheel>,
        item_count: usize,
        current_index: usize,
        columns: usize,
        egui_wants_keyboard: bool,
    ) -> usize {
        if item_count == 0 {
            return 0;
        }
        let mut next = current_index;

        // Consume wheel events for panel tile navigation
        let wheel_direction = mwheel
            .read()
            .filter_map(|event| {
                if event.y > 0.0 {
                    Some(if input.pressed(KeyCode::ShiftLeft) {
                        Direction::Up
                    } else {
                        Direction::Left
                    })
                } else if event.y < 0.0 {
                    Some(if input.pressed(KeyCode::ShiftLeft) {
                        Direction::Down
                    } else {
                        Direction::Right
                    })
                } else {
                    None
                }
            })
            .next();

        if let Some(dir) = wheel_direction {
            match dir {
                Direction::Up => {
                    next = if next >= columns {
                        next - columns
                    } else {
                        (item_count + next - (columns % item_count)) % item_count
                    }
                }
                Direction::Down => next = (next + columns) % item_count,
                Direction::Left => next = if next == 0 { item_count - 1 } else { next - 1 },
                Direction::Right => next = (next + 1) % item_count,
            }
        } else if !egui_wants_keyboard {
            // Only process keyboard navigation when egui doesn't own keyboard input
            if input.just_pressed(KeyCode::ArrowRight) {
                next = (next + 1) % item_count;
            } else if input.just_pressed(KeyCode::ArrowLeft) {
                next = if next == 0 { item_count - 1 } else { next - 1 };
            } else if input.just_pressed(KeyCode::ArrowDown) {
                next = (next + columns) % item_count;
            } else if input.just_pressed(KeyCode::ArrowUp) {
                next = if next >= columns {
                    next - columns
                } else {
                    (item_count + next - (columns % item_count)) % item_count
                };
            }
        }

        next
    }

    pub fn get_current_index(&mut self, tooling_menu_state: &ToolingMenuState) -> usize {
        self.current_index = self
            .items
            .iter()
            .position(|item| Some(item.id) == tooling_menu_state.selected_item_id)
            .unwrap_or(0);
        self.current_index
    }

    pub fn get_panel_width(&self, is_tile_mode: bool) -> f32 {
        if is_tile_mode {
            // Width = N buttons + spacing + padding for scroll bar/margins.
            (self.num_columns as f32 * self.tile_button_px)
                + ((self.num_columns.saturating_sub(1)) as f32 * self.tile_spacing)
                + PANEL_MARGIN_PX
        } else {
            DEFAULT_PANEL_WIDTH
        }
    }

    pub fn show(
        &mut self,
        contexts: &mut EguiContexts,
        editor_state: &Res<State<EditorState>>,
        tooling_menu_state: &mut ToolingMenuState,
        input: &ButtonInput<KeyCode>,
        mwheel: &mut MessageReader<MouseWheel>,
        textures: &Res<TextureHandles>,
        egui_wants_keyboard: bool,
    ) -> Result<f32> {
        if let EditorState::Editing(eo_kind) = editor_state.get() {
            self.configure_left_panel(
                contexts,
                eo_kind,
                tooling_menu_state,
                textures,
                input,
                mwheel,
                egui_wants_keyboard,
            );
        }
        let width = self.draw_left_panel(tooling_menu_state, editor_state, contexts)?;
        return Ok(width);
    }

    fn configure_left_panel(
        &mut self,
        contexts: &mut EguiContexts,
        eo_kind: &EditorObjectKind,
        tooling_menu_state: &ToolingMenuState,
        textures: &Res<TextureHandles>,
        input: &ButtonInput<KeyCode>,
        mwheel: &mut MessageReader<MouseWheel>,
        egui_wants_keyboard: bool,
    ) {
        self.items = tooling_menu_state.items.clone();

        let is_tile_mode = matches!(eo_kind, EditorObjectKind::Tile(_));

        self.tile_texture_id = if is_tile_mode {
            textures.0.get(&eo_kind).map(|handle: &Handle<Image>| {
                contexts.add_image(EguiTextureHandle::Strong(handle.clone()))
            })
        } else {
            None
        };

        self.current_index = self
            .items
            .iter()
            .position(|item| Some(item.id) == tooling_menu_state.selected_item_id)
            .unwrap_or(0);

        self.num_columns = Self::tooling_columns(is_tile_mode);

        self.tile_button_px = TILE_SIZE as f32;
        self.tile_spacing = 4.0;

        self.panel_width = self.get_panel_width(is_tile_mode);

        if self.items.is_empty() {
            self.current_index = 0;
            self.next_selected_id = None;
            return;
        }

        //update current index based on input, this will determine which item is highlighted in the menu and which tile is selected for placement if in tile mode
        self.current_index = Self::move_selection_index(
            input,
            mwheel,
            self.items.len(),
            self.current_index,
            self.num_columns,
            egui_wants_keyboard,
        );

        self.next_selected_id = Some(self.items[self.current_index].id);
    }

    fn draw_left_panel(
        &mut self,
        tooling_menu_state: &mut ToolingMenuState,
        editor_state: &Res<State<EditorState>>,
        contexts: &mut EguiContexts,
    ) -> Result<f32> {
        //only draw the panel if it's visible and we're in an editing state, otherwise just return the default width for the panel without drawing it
        if !tooling_menu_state.visible || !matches!(editor_state.get(), EditorState::Editing(_)) {
            return Ok(0.0);
        }
        //make sure there are items to show, if not just return the default width for the panel without drawing it
        if self.items.is_empty() {
            return Ok(0.0);
        }

        let ctx = contexts.ctx_mut()?;

        let is_tile_mode = matches!(
            editor_state.get(),
            EditorState::Editing(EditorObjectKind::Tile(_))
        );

        let res = egui::SidePanel::left("tooling_menu_panel")
            .frame(editor_panel_frame())
            .resizable(false)
            .default_width(self.panel_width)
            .show(ctx, |ui| {
                ui.heading(
                    egui::RichText::new(tooling_menu_state.title.clone())
                        .strong()
                        .size(18.0)
                        .color(egui::Color32::from_rgba_unmultiplied(220, 230, 245, 255)),
                );
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if is_tile_mode {
                        egui::Grid::new("tooling_grid")
                            .num_columns(self.num_columns)
                            .spacing([self.tile_spacing, self.tile_spacing])
                            .show(ui, |ui| {
                                for (i, item) in self.items.iter().enumerate() {
                                    let item_is_selected = Some(item.id) == self.next_selected_id;

                                    let tile_button_size =
                                        egui::vec2(self.tile_button_px, self.tile_button_px);

                                    let (rect, response) = ui.allocate_exact_size(
                                        tile_button_size,
                                        egui::Sense::click(),
                                    );

                                    let bg = if item_is_selected {
                                        egui::Color32::from_rgba_unmultiplied(245, 230, 120, 100)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20)
                                    };
                                    ui.painter().rect_filled(rect, 4.0, bg);

                                    if let (Some(texture_id), Some(tile_rect)) =
                                        (self.tile_texture_id, item.rect)
                                    {
                                        let atlas_w = (SPRITESHEET_WIDTH * TILE_SIZE as u64) as f32;
                                        let rows = ((MAX_SPRITESHEET_ITEMS - SPRITESHEET_WIDTH)
                                            / SPRITESHEET_WIDTH)
                                            as f32;
                                        let atlas_h =
                                            (rows * TILE_SIZE as f32).max(TILE_SIZE as f32);
                                        let uv = egui::Rect::from_min_max(
                                            egui::pos2(
                                                tile_rect.min.x / atlas_w,
                                                tile_rect.min.y / atlas_h,
                                            ),
                                            egui::pos2(
                                                tile_rect.max.x / atlas_w,
                                                tile_rect.max.y / atlas_h,
                                            ),
                                        );

                                        let image_rect = rect.shrink2(egui::vec2(2.0, 2.0));
                                        ui.painter().image(
                                            texture_id,
                                            image_rect,
                                            uv,
                                            egui::Color32::WHITE,
                                        );
                                    }

                                    let stroke_color = if item_is_selected {
                                        egui::Color32::from_rgb(245, 230, 120)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(210, 220, 235, 120)
                                    };
                                    ui.painter().rect_stroke(
                                        rect,
                                        4.0,
                                        egui::Stroke::new(1.0, stroke_color),
                                        egui::StrokeKind::Outside,
                                    );

                                    let label = item.label.as_str();
                                    let text_pos =
                                        egui::pos2(rect.right() - 3.0, rect.bottom() - 2.0);
                                    ui.painter().text(
                                        text_pos,
                                        egui::Align2::RIGHT_BOTTOM,
                                        label,
                                        egui::FontId::proportional(11.0),
                                        egui::Color32::from_rgb(250, 250, 250),
                                    );

                                    if response.clicked() {
                                        self.next_selected_id = Some(item.id);
                                    }

                                    if (i + 1) % self.num_columns == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    } else {
                        for item in &self.items {
                            let selected = Some(item.id) == self.next_selected_id;
                            if ui.selectable_label(selected, item.label.as_str()).clicked() {
                                self.next_selected_id = Some(item.id);
                            }
                        }
                    }
                });
            });

        if tooling_menu_state.selected_item_id != self.next_selected_id {
            tooling_menu_state.selected_item_id = self.next_selected_id;
        }
        Ok(res.response.rect.width())
    }
}

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bevy_egui::egui::{self, TextureId};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::editor::editor_modes::tile_mode::TileObject;
use crate::editor::ui::message_display::MessageDisplay;
use crate::{MAX_SPRITESHEET_ITEMS, SPRITESHEET_WIDTH, TILE_SIZE};

const TILE_CATALOG_PATH: &str = "assets/catalog/tile_catalog.ron";
const TILESET_TEXTURE_PATH: &str = "textures/tiles/tilesheet.png";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaterialType {
    Ground,
    Brick,
    Pipe,
    Decor,
    Hazard,
}

impl Default for MaterialType {
    fn default() -> Self {
        Self::Ground
    }
}

impl MaterialType {
    fn all() -> [Self; 5] {
        [
            Self::Ground,
            Self::Brick,
            Self::Pipe,
            Self::Decor,
            Self::Hazard,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Ground => "ground",
            Self::Brick => "brick",
            Self::Pipe => "pipe",
            Self::Decor => "decor",
            Self::Hazard => "hazard",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraversalAffordance {
    WalkableTop,
    Wall,
    OneWay,
}

impl Default for TraversalAffordance {
    fn default() -> Self {
        Self::WalkableTop
    }
}

impl TraversalAffordance {
    fn all() -> [Self; 3] {
        [Self::WalkableTop, Self::Wall, Self::OneWay]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::WalkableTop => "walkable_top",
            Self::Wall => "wall",
            Self::OneWay => "one_way",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TileCatalogEntry {
    pub name: String,
    pub material_type: MaterialType,
    pub solidity_default: bool,
    pub traversal_affordance: TraversalAffordance,
    pub style_tags: Vec<String>,
    pub usage_hints: String,
    pub anti_patterns: String,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct TileCatalog(pub HashMap<u64, TileCatalogEntry>);

#[derive(Debug, Clone)]
struct TileCatalogDraft {
    tile_id: u64,
    name: String,
    material_type: MaterialType,
    solidity_default: bool,
    traversal_affordance: TraversalAffordance,
    style_tags_csv: String,
    usage_hints: String,
    anti_patterns: String,
}

impl TileCatalogDraft {
    fn for_tile(tile_id: u64) -> Self {
        Self {
            tile_id,
            name: format!("tile_{tile_id}"),
            material_type: MaterialType::Ground,
            solidity_default: true,
            traversal_affordance: TraversalAffordance::WalkableTop,
            style_tags_csv: String::new(),
            usage_hints: String::new(),
            anti_patterns: String::new(),
        }
    }

    fn into_entry(self) -> TileCatalogEntry {
        let style_tags = self
            .style_tags_csv
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        TileCatalogEntry {
            name: self.name.trim().to_string(),
            material_type: self.material_type,
            solidity_default: self.solidity_default,
            traversal_affordance: self.traversal_affordance,
            style_tags,
            usage_hints: self.usage_hints.trim().to_string(),
            anti_patterns: self.anti_patterns.trim().to_string(),
        }
    }
}

#[derive(Resource)]
struct TileCatalogUpdateState {
    pending_tile_ids: Vec<u64>,
    current_draft: Option<TileCatalogDraft>,
    tile_texture: Handle<Image>,
    tile_texture_id: Option<TextureId>,
    active: bool,
    last_error: Option<String>,
    skipped_tile_ids: Vec<u64>,
}

impl TileCatalogUpdateState {
    fn new(pending_tile_ids: Vec<u64>, tile_texture: Handle<Image>) -> Self {
        let current_draft = pending_tile_ids.first().copied().map(TileCatalogDraft::for_tile);
        Self {
            active: !pending_tile_ids.is_empty(),
            pending_tile_ids,
            current_draft,
            tile_texture,
            tile_texture_id: None,
            last_error: None,
            skipped_tile_ids: Vec::new(),
        }
    }

    fn advance(&mut self) {
        if !self.pending_tile_ids.is_empty() {
            self.pending_tile_ids.remove(0);
        }
        self.current_draft = self
            .pending_tile_ids
            .first()
            .copied()
            .map(TileCatalogDraft::for_tile);
        self.active = self.current_draft.is_some();
        self.last_error = None;
    }
}

pub struct CatalogueUpdatePlugin;

impl Plugin for CatalogueUpdatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileCatalog>()
            .add_systems(Startup, initialize_tile_catalog)
            .add_systems(EguiPrimaryContextPass, render_catalogue_update_ui);
    }
}

fn initialize_tile_catalog(
    mut commands: Commands,
    mut catalog: ResMut<TileCatalog>,
    asset_server: Res<AssetServer>,
    mut bottom_bar: ResMut<MessageDisplay>,
) {
    match load_catalog_from_disk() {
        Ok(loaded) => catalog.0 = loaded,
        Err(err) => {
            bottom_bar.send_message(format!("Tile catalog load failed: {err}"));
            catalog.0.clear();
        }
    }

    let mut missing_tile_ids = Vec::new();
    for tile_id in 0..MAX_SPRITESHEET_ITEMS {
        if !catalog.0.contains_key(&tile_id) {
            missing_tile_ids.push(tile_id);
        }
    }

    let tile_texture = asset_server.load(TILESET_TEXTURE_PATH);
    let missing_count = missing_tile_ids.len();

    commands.insert_resource(TileCatalogUpdateState::new(missing_tile_ids, tile_texture));

    if missing_count > 0 {
        bottom_bar.send_message(format!(
            "Tile catalog has {missing_count} missing entries. Opened catalog updater."
        ));
    }
}

fn load_catalog_from_disk() -> Result<HashMap<u64, TileCatalogEntry>, String> {
    let path = Path::new(TILE_CATALOG_PATH);
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(path)
        .map_err(|err| format!("could not read {}: {err}", path.display()))?;

    if content.trim().is_empty() {
        return Ok(HashMap::new());
    }

    ron::de::from_str::<HashMap<u64, TileCatalogEntry>>(&content)
        .map_err(|err| format!("invalid RON in {}: {err}", path.display()))
}

fn persist_catalog_to_disk(catalog: &HashMap<u64, TileCatalogEntry>) -> Result<(), String> {
    let path = PathBuf::from(TILE_CATALOG_PATH);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("could not create {}: {err}", parent.display()))?;
    }

    let serialized = ron::ser::to_string_pretty(catalog, PrettyConfig::default())
        .map_err(|err| format!("could not serialize tile catalog: {err}"))?;

    std::fs::write(&path, serialized)
        .map_err(|err| format!("could not write {}: {err}", path.display()))
}

enum CatalogueUiAction {
    None,
    Save,
    Skip,
    SkipAll,
}

fn render_catalogue_update_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<TileCatalogUpdateState>,
    mut catalog: ResMut<TileCatalog>,
    mut bottom_bar: ResMut<MessageDisplay>,
) -> Result {
    if !state.active {
        return Ok(());
    }

    if state.current_draft.is_none() {
        state.active = false;
        return Ok(());
    }

    if state.tile_texture_id.is_none() {
        state.tile_texture_id = Some(contexts.add_image(EguiTextureHandle::Strong(
            state.tile_texture.clone(),
        )));
    }

    let ctx = contexts.ctx_mut()?;

    let Some(texture_id) = state.tile_texture_id else {
        return Ok(());
    };

    let mut action = CatalogueUiAction::None;

    let pending_total = state.pending_tile_ids.len();
    let skipped_count = state.skipped_tile_ids.len();
    let last_error = state.last_error.clone();

    let Some(draft) = state.current_draft.as_mut() else {
        return Ok(());
    };

    egui::Window::new("Catalogue Update")
        .collapsible(false)
        .resizable(true)
        .default_size(egui::vec2(520.0, 460.0))
        .show(ctx, |ui| {
            ui.label(format!(
                "Missing tile catalog entry for tile_id {} ({} remaining)",
                draft.tile_id, pending_total
            ));
            if skipped_count > 0 {
                ui.label(format!("Skipped so far: {skipped_count}"));
            }
            if let Some(err) = &last_error {
                ui.colored_label(egui::Color32::from_rgb(230, 90, 90), err);
            }

            ui.separator();

            let preview_rect = TileObject::get_uv_rect(draft.tile_id);
            let atlas_w = (SPRITESHEET_WIDTH * TILE_SIZE as u64) as f32;
            let atlas_rows = (MAX_SPRITESHEET_ITEMS / SPRITESHEET_WIDTH) as f32;
            let atlas_h = atlas_rows.max(1.0) * TILE_SIZE as f32;
            let uv = egui::Rect::from_min_max(
                egui::pos2(preview_rect.min.x / atlas_w, preview_rect.min.y / atlas_h),
                egui::pos2(preview_rect.max.x / atlas_w, preview_rect.max.y / atlas_h),
            );

            let preview_size = egui::vec2((TILE_SIZE * 4) as f32, (TILE_SIZE * 4) as f32);

            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new((texture_id, preview_size))
                        .uv(uv)
                        .texture_options(egui::TextureOptions::NEAREST),
                );

                ui.vertical(|ui| {
                    ui.label("name");
                    ui.text_edit_singleline(&mut draft.name);

                    ui.label("material type");
                    egui::ComboBox::from_id_salt("catalog_material_type")
                        .selected_text(draft.material_type.label())
                        .show_ui(ui, |ui| {
                            for option in MaterialType::all() {
                                ui.selectable_value(
                                    &mut draft.material_type,
                                    option.clone(),
                                    option.label(),
                                );
                            }
                        });

                    ui.checkbox(&mut draft.solidity_default, "solidity default");

                    ui.label("traversal affordance");
                    egui::ComboBox::from_id_salt("catalog_traversal_affordance")
                        .selected_text(draft.traversal_affordance.label())
                        .show_ui(ui, |ui| {
                            for option in TraversalAffordance::all() {
                                ui.selectable_value(
                                    &mut draft.traversal_affordance,
                                    option.clone(),
                                    option.label(),
                                );
                            }
                        });
                });
            });

            ui.separator();
            ui.label("style tags (comma-separated)");
            ui.text_edit_singleline(&mut draft.style_tags_csv);

            ui.label("usage hints");
            ui.add(
                egui::TextEdit::multiline(&mut draft.usage_hints)
                    .desired_rows(4)
                    .hint_text("What this tile is good for"),
            );

            ui.label("anti-patterns");
            ui.add(
                egui::TextEdit::multiline(&mut draft.anti_patterns)
                    .desired_rows(4)
                    .hint_text("How this tile should not be used"),
            );

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save Entry").clicked() {
                    action = CatalogueUiAction::Save;
                }
                if ui.button("Skip").clicked() {
                    action = CatalogueUiAction::Skip;
                }
                if ui.button("Skip All").clicked() {
                    action = CatalogueUiAction::SkipAll;
                }
            });
        });

    match action {
        CatalogueUiAction::None => {}
        CatalogueUiAction::Save => {
            let Some(saved_draft) = state.current_draft.clone() else {
                return Ok(());
            };

            let tile_id = saved_draft.tile_id;
            let entry = saved_draft.into_entry();

            if entry.name.is_empty() {
                state.last_error = Some("name can not be empty".to_string());
                return Ok(());
            }

            catalog.0.insert(tile_id, entry);
            match persist_catalog_to_disk(&catalog.0) {
                Ok(_) => {
                    bottom_bar.send_message(format!("Catalog updated for tile_id {tile_id}"));
                    state.advance();
                    if !state.active {
                        bottom_bar.send_message("Tile catalog update complete.");
                    }
                }
                Err(err) => {
                    state.last_error = Some(err);
                }
            }
        }
        CatalogueUiAction::Skip => {
            if let Some(current_tile_id) = state.current_draft.as_ref().map(|d| d.tile_id) {
                state.skipped_tile_ids.push(current_tile_id);
                bottom_bar.send_message(format!("Skipped tile_id {}", current_tile_id));
            }
            state.advance();
            if !state.active {
                bottom_bar.send_message("Tile catalog updater finished with skips.");
            }
        }
        CatalogueUiAction::SkipAll => {
            let remaining = state.pending_tile_ids.len();
            state.pending_tile_ids.clear();
            state.current_draft = None;
            state.active = false;
            bottom_bar.send_message(format!(
                "Tile catalog updater dismissed (skipped {remaining} remaining entries)."
            ));
        }
    }

    Ok(())
}

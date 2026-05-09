use bevy::window::PrimaryWindow;

use super::*;
use crate::editor_modes::editor_mode::EditorModePlugin;
use crate::editor_modes::significant_component::SignificantComponent;
use crate::ui::{message_display::*, ToolingMenuItem};
use crate::{
    mouse_state::MouseState, Crosshair, CustomInput, EditorState, TextureHandles,
    MAX_SPRITESHEET_ITEMS,
};
use crate::rendering::MainWorldCamera;

use crate::{configure_tooling_menu, SelectedTileID, ToolingMenuState};

use std::path::PathBuf;

pub mod tile_id;
pub use tile_id::TileID;

pub mod tile;
pub use tile::TileObject;

pub struct TileModePlugin;

impl Plugin for TileModePlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin::<TileObject>(app);
    }
}

impl EditorModePlugin for TileModePlugin {
    fn modify_app(app: &mut App) -> &mut App {
        app.init_resource::<SelectedTileID>()
    }

    fn mode() -> EditorState {
        EditorState::Editing(EditorObjectKind::Tile(TileID::Any))
    }

    fn init(
        mut spritesheets: ResMut<TextureHandles>,
        mut bottom_bar: ResMut<MessageDisplay>,
        asset_server: Res<AssetServer>,
    ) {
        let tex_path = PathBuf::from("textures/tiles/tilesheet.png");

        bottom_bar.send_message(format!("Tilesheet Loaded: \"{}\"", tex_path.display()));

        spritesheets.0.insert(
            EditorObjectKind::Tile(TileID::Any),
            asset_server.load(tex_path),
        );
    }

    fn enter_mode(
        mut tooling_menu: ResMut<ToolingMenuState>,
        selected_item_id: Res<SelectedTileID>,
    ) {
        configure_tooling_menu(
            &mut tooling_menu,
            "Tile Parts",
            Some(selected_item_id.0),
            (0..MAX_SPRITESHEET_ITEMS)
                .map(|tile_id| ToolingMenuItem {
                    id: tile_id,
                    label: tile_id.to_string(),
                    texture_key: Some(EditorObjectKind::Tile(TileID::Some(tile_id))),
                    rect: Some(TileObject::get_uv_rect(tile_id)),
                })
                .collect(),
        );
    }

    fn exit_mode(mut bottom_bar: ResMut<MessageDisplay>) {
        bottom_bar.send_mode_exit_message("Tile");
    }

    fn mode_keybinds<T: Component + SignificantComponent>(
        mut commands: Commands,
        input: Res<ButtonInput<KeyCode>>,
        crosshair: Single<(&Transform, &Crosshair)>,
        tiles: Query<(Entity, &EditorObject), With<T>>,
        selected_tile_id: ResMut<SelectedTileID>,
        mut bottom_bar: ResMut<MessageDisplay>,
        _next_editor_state: ResMut<NextState<EditorState>>,
    ) {
        if input.just_pressed(KeyCode::KeyP) {
            let coord = Coordinate::new_world_space(
                crosshair.0.translation.x as i64,
                crosshair.0.translation.y as i64,
            )
            .snap_to_grid();
            let to_place =
                EditorObject::new(EditorObjectKind::Tile(TileID::Some(selected_tile_id.0)), coord);

            TileObject::place(&mut commands, to_place, &tiles);
            bottom_bar.send_place_eo_message("tile", coord);
        }

        if input.just_pressed(KeyCode::KeyL) {
            let coord = Coordinate::new_world_space(
                crosshair.0.translation.x as i64,
                crosshair.0.translation.y as i64,
            )
            .snap_to_grid();

            TileObject::remove(
                &mut commands,
                coord,
                EditorObjectKind::Tile(TileID::Any),
                &tiles,
            );
            bottom_bar.send_remove_eo_message("tiles", coord);
        }
    }

    fn mode_click<T: SignificantComponent + Component + Default>(
        commands: Commands,
        window: Single<&Window, With<PrimaryWindow>>,
        camera: Single<(&Camera, &GlobalTransform), With<MainWorldCamera>>,
        items: Query<(Entity, &EditorObject), With<T>>,
        mut selected_tile_id: ResMut<SelectedTileID>,
        mouse_state: Res<MouseState>,
        bottom_bar: ResMut<MessageDisplay>,
    ) {
        if let Some(mouse_pos) = window.cursor_position() {
            let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, mouse_pos) else {
                return;
            };

            let snapped_coord: Coordinate =
                Coordinate::new_world_space(world_pos.x as i64, world_pos.y as i64).snap_to_grid();

            let eyedropped = mouse_state.drag_action(
                commands,
                snapped_coord,
                EditorObjectKind::Tile(TileID::Some(selected_tile_id.0)),
                bottom_bar,
                items,
            );

            if let Some(EditorObjectKind::Tile(TileID::Some(id))) = eyedropped {
                selected_tile_id.0 = id;
            }
        }
    }

    fn get_mode_kb() -> Vec<(CustomInput, String)> {
        vec![
            (CustomInput::Single(KeyCode::KeyP), "Place Tile".into()),
            (CustomInput::Single(KeyCode::KeyL), "Remove Tile".into()),
            (CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into()),
        ]
    }
}

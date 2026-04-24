use bevy::window::PrimaryWindow;

use super::*;
use crate::editor_modes::editor_mode::EditorModePlugin;
use crate::editor_modes::significant_component::SignificantComponent;
use crate::ui::{message_display::*, update_placeholder, ToolingMenuItem};
use crate::{
    AvailableKeybinds, Crosshair, CustomInput, Dragging, Editor, EditorState, MouseToolState,
    TextureHandles, TileUpdateNeeded, MAX_SPRITESHEET_ITEMS,
};

use crate::{configure_tooling_menu, SelectedTileID, ToolingMenuState};

use std::path::PathBuf;

pub mod tile_id;
pub use tile_id::TileID;

pub mod tile;
pub use tile::TileObject;


pub struct TileModePlugin;

impl Plugin for TileModePlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin(app);
    }
}

impl TileModePlugin {
    fn run_if_tile_update_needed(needs_update: Res<TileUpdateNeeded>) -> bool {
        needs_update.0
    }

    fn reset_update_needed(mut needs_update: ResMut<TileUpdateNeeded>) {
        needs_update.0 = false;
    }

    fn populate_tooling_menu(
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
}

impl EditorModePlugin for TileModePlugin {
    fn build_plugin(app: &mut App) {
        app.register_type::<TileObject>()
            .register_type::<Coordinate>()
            .register_type::<TCoordinate>()
            .init_resource::<SelectedTileID>()
            .init_resource::<TileUpdateNeeded>()
            // .init_resource::<SpritesheetCrop>()
            // .insert_resource(PlaceholderObject(EditorObject::default()))
            //startup systems (may need to be moved from here to maintain order)
            .add_systems(Startup, Self::init)
            //OnEnter systems
            .add_systems(
                OnEnter(EditorState::Editing(EditorObjectKind::Tile(TileID::Any))),
                (
                    Self::populate_tooling_menu,
                    update_placeholder::<TileObject>,
                    Self::add_mode_kb,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(EditorState::Editing(EditorObjectKind::Tile(TileID::Any))),
                (Self::remove_mode_kb).chain(),
            )
            //Update systems, that run only while TileEditor is active
            .add_systems(
                Update,
                (
                    Self::mode_keybinds::<TileObject>,
                    (Self::mode_click::<TileObject>).run_if(Editor::editor_is_dragging),
                    (update_placeholder::<TileObject>, Self::reset_update_needed)
                        .run_if(Self::run_if_tile_update_needed),
                )
                    .chain()
                    .run_if(in_state(EditorState::Editing(
                        EditorObjectKind::Tile(TileID::Any),
                    ))),
            )
            //OnExit systems
            .add_systems(
                OnExit(EditorState::Editing(EditorObjectKind::Tile(TileID::Any))),
                (Self::exit_mode).chain(),
            );
    }

    fn init(
        mut spritesheets: ResMut<TextureHandles>,
        mut bottom_bar: ResMut<MessageDisplay>,
        asset_server: Res<AssetServer>,
    ) {
        //load the tilesheet for this mode
        let tex_path = PathBuf::from("textures/tiles/tilesheet.png");

        send_message!(
            Some('i'),
            bottom_bar.queue,
            format!("Tilesheet Loaded: \"{}\"", &tex_path.clone().display())
        );

        //load happens here
        spritesheets.0.insert(
            EditorObjectKind::Tile(TileID::Any),
            asset_server.load(tex_path),
        );
    }

    fn exit_mode(mut bottom_bar: ResMut<MessageDisplay>) {
        bottom_bar.send_mode_exit_message("Tile");
    }

    fn mode_keybinds<T: Component + SignificantComponent + Default>(
        mut commands: Commands,

        mut bottom_bar: ResMut<MessageDisplay>,
        input: Res<ButtonInput<KeyCode>>,

        crosshair: Single<(&Transform, &Crosshair)>,
        tiles: Query<(Entity, &EditorObject), With<T>>,
        selected_tile_id: ResMut<SelectedTileID>,
    ) {
        //"P" handles placement of a tile and adding it to the scene
        //places the first tile in the selection rect
        if input.just_pressed(KeyCode::KeyP) {
            let coord = Coordinate::from(crosshair.0.translation).snap_to_grid();
            let to_place = EditorObject::new(
                EditorObjectKind::Tile(TileID::Some(selected_tile_id.0)),
                coord,
                EditorObjectKind::Other,
            );

            TileObject::place(&mut commands, to_place, &tiles);
            bottom_bar.send_place_eo_message("tile", coord);
        }

        // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
        if input.just_pressed(KeyCode::KeyL) {
            let coord = Coordinate::from(crosshair.0.translation).snap_to_grid();

            TileObject::remove(
                &mut commands,
                coord,
                EditorObjectKind::Tile(TileID::Any),
                &tiles,
            );
            bottom_bar.send_remove_eo_message("tiles", coord);
        }

        // Selection changes are now handled by the egui tooling panel.
    }

    fn mode_click<T: SignificantComponent + Component + Default>(
        mut commands: Commands,
        window: Single<&Window, With<PrimaryWindow>>,
        camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
        items: Query<(Entity, &EditorObject), With<T>>,
        selected_tile_id: Res<SelectedTileID>,
        dragging: Res<Dragging>,
        bottom_bar: ResMut<MessageDisplay>,
        mouse_mode: ResMut<MouseToolState>,
    ) {
        if let Some(mouse_pos) = window.cursor_position() {
            let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, mouse_pos) else {
                return;
            };

            let snapped_coord: Coordinate =
                Coordinate::new_world_space(world_pos.x as i64, world_pos.y as i64).snap_to_grid();

            dragging.drag_action(
                &mut commands,
                mouse_mode.current,
                snapped_coord,
                EditorObjectKind::Tile(TileID::Some(selected_tile_id.0)),
                bottom_bar,
                &items,
            );
        }
    }

    fn add_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyL), "Remove Tile".into());
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyP), "Place Tile".into());
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into());
    }
    fn remove_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
        available_keybinds.clear();
    }
}

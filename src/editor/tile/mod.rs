pub mod tile_ui;

use super::*;
use crate::ui::{bottom_bar::*, update_placeholder, ToolingMenuItem};

use crate::{EditorObject, TILE_SIZE};
use bevy::prelude::*;
use std::path::PathBuf;
use tools::SignificantComponent;

fn populate_tile_tooling_menu(
    mut tooling_menu: ResMut<ToolingMenuState>,
    selected_tile_id: Res<SelectedTileID>,
) {
    configure_tooling_menu(
        &mut tooling_menu,
        "Tile Parts",
        Some(selected_tile_id.id),
        (0..MAX_SPRITESHEET_ITEMS)
            .map(|tile_id| ToolingMenuItem {
                id: tile_id,
                label: tile_id.to_string(),
                texture_key: Some(EditorObjectKind::Tile),
                rect: Some(Tile::get_tex_rect(tile_id)),
            })
            .collect(),
    );
}

fn load_spritesheet(
    asset_server: Res<AssetServer>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    mut texture_handles: ResMut<TextureHandles>,
) {
    //load the tilesheet for this mode
    let tex_path = PathBuf::from("textures/tiles/tilesheet.png");

    send_message!(
        Some('i'),
        message_queue,
        format!("Tilesheet Loaded: \"{}\"", &tex_path.clone().display())
    );

    //load happens here
    texture_handles
        .0
        .insert(EditorObjectKind::Tile, asset_server.load(tex_path));
}

fn tilemode_click(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    tiles: Query<(Entity, &EditorObject), With<Tile>>,
    selected_tile_id: Res<SelectedTileID>,
    dragging: Res<Dragging>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
) {
    if let Some(mouse_pos) = window.cursor_position() {
        let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, mouse_pos) else {
            return;
        };

        let snapped_coord: Coordinate =
            Coordinate::game(world_pos.x as i64, world_pos.y as i64).snap_to_grid();

        match dragging.dragging_button() {
            Some(MouseButton::Left) => {
                let to_place = build_editor_object(
                    EditorObjectKind::Tile,
                    selected_tile_id.id,
                    snapped_coord,
                    EditorObjectKind::Other,
                );
                Tile::place(&mut commands, to_place, &tiles);
                send_place_eo_message(&mut message_queue, "tile", snapped_coord);
            }
            Some(MouseButton::Right) => {
                Tile::remove(&mut commands, snapped_coord, EditorObjectKind::Tile, &tiles);
                send_remove_eo_message(&mut message_queue, "tiles", snapped_coord);
            }
            _ => {}
        }
    }
}

fn tilemode_keybinds(
    mut commands: Commands,

    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    input: Res<ButtonInput<KeyCode>>,

    crosshair: Single<(&Transform, &Crosshair)>,
    tiles: Query<(Entity, &EditorObject), With<Tile>>,
    selected_tile_id: ResMut<SelectedTileID>,
) {
    //"P" handles placement of a tile and adding it to the scene
    //places the first tile in the selection rect
    if input.just_pressed(KeyCode::KeyP) {
        let coord = Coordinate::from(crosshair.0.translation).snap_to_grid();
        let to_place = build_editor_object(
            EditorObjectKind::Tile,
            selected_tile_id.id,
            coord,
            EditorObjectKind::Other,
        );

        Tile::place(&mut commands, to_place, &tiles);
        send_place_eo_message(&mut message_queue, "tile", coord);
    }

    // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let coord = Coordinate::from(crosshair.0.translation).snap_to_grid();

        Tile::remove(&mut commands, coord, EditorObjectKind::Tile, &tiles);
        send_remove_eo_message(&mut message_queue, "tiles", coord);
    }

    // Selection changes are now handled by the egui tooling panel.
}

fn exit_tilemode(mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {
    send_mode_exit_message(&mut message_queue, "Tile");
}

/// A component that marks an entity as part of the tile editing UI.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(CameraLockedUI)]
struct TileModeUI;

/// A component to track some basic info about a tile (actually its just a tag right now but that might change)
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(EditorObject)]

pub struct Tile {}

impl Tile {
    fn new() -> Self {
        Self {}
    }

    pub fn get_tex_rect(tile_id: u64) -> Rect {
        Rect {
            min: Vec2::new(
                (tile_id % SPRITESHEET_WIDTH) as f32 * TILE_SIZE as f32,
                (tile_id / SPRITESHEET_WIDTH) as f32 * TILE_SIZE as f32,
            ),
            max: Vec2::new(
                (tile_id % SPRITESHEET_WIDTH + 1) as f32 * TILE_SIZE as f32,
                (tile_id / SPRITESHEET_WIDTH + 1) as f32 * TILE_SIZE as f32,
            ),
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new()
    }
}

impl SignificantComponent for Tile {
    fn place_rectangle(_rect: Rect, _commands: Commands) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }

    fn from_rect(_rect: Rect, _coord: Coordinate) -> Self {
        Self {}
    }
}

fn run_if_tile_update_needed(needs_update: Res<TileUpdateNeeded>) -> bool {
    needs_update.0
}

fn reset_update_needed(mut needs_update: ResMut<TileUpdateNeeded>) {
    needs_update.0 = false;
}

fn add_tile_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyL), "Remove Tile".into());
    available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyP), "Place Tile".into());
    available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into());
}
fn remove_tile_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.clear();
}


pub fn tilemode_plugin(app: &mut App) {
    app.register_type::<Tile>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        .register_type::<TileModeUI>()
        .init_resource::<SelectedTileID>()
        .init_resource::<TileUpdateNeeded>()
        // .init_resource::<SpritesheetCrop>()
        // .insert_resource(PlaceholderObject(EditorObject::default()))
        //startup systems (may need to be moved from here to maintain order)
        .add_systems(Startup, load_spritesheet)
        //OnEnter systems
        .add_systems(
            OnEnter(EditorState::Editing(EditorObjectKind::Tile)),
            (
                populate_tile_tooling_menu,
                update_placeholder::<Tile>,
                add_tile_mode_kb,
            )
                .chain(),
        )
        .add_systems(
            OnExit(EditorState::Editing(EditorObjectKind::Tile)),
            (remove_tile_mode_kb).chain(),
        )
        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (
                tilemode_keybinds,
                (tilemode_click).run_if(is_dragging),
                (update_placeholder::<Tile>, reset_update_needed).run_if(run_if_tile_update_needed),
            )
                .chain()
                .run_if(in_state(EditorState::Editing(EditorObjectKind::Tile))),
        )
        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditorObjectKind::Tile)),
            (despawn_all::<TileModeUI>, exit_tilemode).chain(),
        );
    //we could also take care of some post-exit cleanup here, like despawning all the UI elements by using the schedule OnEnter(EditorState::Inactive) and then despawning all the UI elements
}
//NOTHING BELOW THE PLUGINS!

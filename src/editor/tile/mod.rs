pub mod ui;

use super::*;
use crate::ui::{update_placeholder, ToolingMenuItem};
use crate::{EditorObject, TILE_SIZE};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::path::PathBuf;
use tools::SignificantComponent;

fn get_tex_rect(tile_id: u64) -> Rect {
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

fn populate_tile_tooling_menu(
    mut tooling_menu: ResMut<ToolingMenuState>,
    selected_tile_id: Res<SelectedTileID>,
) {
    tooling_menu.title = "Tile Parts".to_string();
    tooling_menu.visible = true;
    tooling_menu.selected_item_id = Some(selected_tile_id.id);
    tooling_menu.items = (0..MAX_SPRITESHEET_ITEMS)
        .map(|tile_id| ToolingMenuItem {
            id: tile_id,
            label: tile_id.to_string(),
            texture_key: Some(EditorObjectKind::Tile),
            rect: Some(get_tex_rect(tile_id)),
        })
        .collect();
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

fn tilemode_keybinds(
    mut commands: Commands,

    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    input: Res<ButtonInput<KeyCode>>,

    crosshair: Single<(&Transform, &Crosshair)>,
    tiles: Query<(Entity, &EditorObject), With<Tile>>,
    gridsnap: Res<State<GridSnap>>,
    selected_tile_id: ResMut<SelectedTileID>,
) {
    //"P" handles placement of a tile and adding it to the scene
    //places the first tile in the selection rect
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        //get the coordinate of the crosshair AND snap it to the grid if gridsnap is enabled
        let mut coord = Coordinate::from(crosshair.0.translation);
        if gridsnap.get() == &GridSnap::Enabled {
            coord = snap_coordinate_to_grid(coord);
        }

        let first_tile = selected_tile_id.id;

        // create a new EditorObject to place
        let to_place = EditorObject {
            
            kind: EditorObjectKind::Tile,
            coordinate: coord,
            internal_kind: first_tile as u64,
            zone_id: TCoordinate::new(
                EditorObjectKind::Other,
                Coordinate {
                    0: coord.0 / ZONE_SIZE as i64,
                    1: coord.1 / ZONE_SIZE as i64,
                },
            ),
        };

        //place the tile using our SignificantComponent trait
        Tile::place(
            &mut commands,
            to_place,
            &tiles,
        );
        send_message!(
            Some('i'),
            message_queue,
            format!("Placed tile at: ({}, {})", coord.0, coord.1)
        );
    }

    // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let mut coord = Coordinate::from(crosshair.0.translation);
        coord = snap_coordinate_to_grid(coord);

        Tile::remove(&mut commands, coord, EditorObjectKind::Tile, &tiles);
        send_message!(
            Some('i'),
            message_queue,
            format!("Removing tiles at: ({}, {})", coord.0, coord.1)
        );
    }

    // Selection changes are now handled by the egui tooling panel.
}

fn exit_tilemode(mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {
    //remove the CurrentEditorObject resource
    // commands.insert_resource(PlaceholderObject(EditorObject::default()));
    send_message!(
        Some('i'),
        message_queue,
        "Exiting Tile Editing Mode".to_string()
    );
}

/// A component that marks an entity as part of the tile editing UI.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(UIItem)]
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
    if needs_update.0 {
        true
    } else {
        false
    }
}

fn reset_update_needed(mut needs_update: ResMut<TileUpdateNeeded>) {
    needs_update.0 = false;
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
            OnEnter(EditorState::Editing(EditingComponent::Tile)),
            (
                populate_tile_tooling_menu,
                update_placeholder::<Tile>,
                ui::spawn_tile_placeholder,
            )
                .chain(),
        )
        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (
                tilemode_keybinds,
                (update_placeholder::<Tile>, reset_update_needed).run_if(run_if_tile_update_needed),
            )
                .chain()
                .run_if(in_state(EditorState::Editing(EditingComponent::Tile))),
        )
        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditingComponent::Tile)),
            (despawn_all::<TileModeUI>, exit_tilemode).chain(),
        );
    //we could also take care of some post-exit cleanup here, like despawning all the UI elements by using the schedule OnEnter(EditorState::Inactive) and then despawning all the UI elements
}
//NOTHING BELOW THE PLUGINS!

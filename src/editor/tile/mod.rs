pub mod ui;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::path::PathBuf;
use crate::{ utilities::*, resources::*, EditorObject, TILE_SIZE };
use crate::consts::*;
use super::*;

// #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
// pub enum TileEditorState {
//     #[default]
//     Inactive,
//     Active,
// }

pub fn tilemode_plugin(app: &mut App) {
    app
        .register_type::<Tile>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        .register_type::<TileModeUI>()
        .insert_resource(PlaceholderTile(Tile::new()))
        //startup systems (may need to be moved from here to maintain order)
        .add_systems(Startup, load_spritesheet)

        //OnEnter systems
        .add_systems(OnEnter(EditorState::Editing(EditingMode::Tile)), (init_tilemode, ui::show_placeholder, ui::create_tilemode_ui).chain())

        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (tilemode_keybinds, ui::update_placeholder)
                .chain()
                .run_if(in_state(EditorState::Editing(EditingMode::Tile)))
        )

        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditingMode::Tile)), 
            (
                despawn_all::<TileModeUI>,
                exit_tilemode
            ).chain()
        );

    //we could also take care of some post-exit cleanup here, like despawning all the UI elements by using the schedule OnEnter(EditorState::Inactive) and then despawning all the UI elements
}

// fn ensure_unique_tiles(mut commands: Commands, tiles: Query<(Entity, &Tile)>) {
//     let mut seen = std::collections::HashSet::new();

//     for (e, t) in tiles.iter() {
//         if seen.contains(&t.coordinate) {
//             //remove the older of the two tiles
//             commands.entity(e).despawn();
//         } else {
//             seen.insert(&t.coordinate);
//         }
//     }
// }

fn init_tilemode(
) {
    println!("Entering Tile Editing Mode");

}

fn tilemode_keybinds(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,

    crosshair: Single<(&Transform, &Crosshair)>,
    tiles: Query<(Entity, &EditorObject), With<Tile>>,
    gridsnap: Res<State<GridSnap>>,
    selected_tile_id: ResMut<SelectedTileID>,
) {
    //"P" handles placement of a tile and adding it to the scene
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        //get the coordinate of the crosshair AND snap it to the grid if gridsnap is enabled
        let mut coord = Coordinate::from(crosshair.0.translation);
        if gridsnap.get() == &GridSnap::Enabled {
            coord = snap_coordinate_to_grid(coord);
        }

        let first_tile = selected_tile_id.id;

        let to_place = EditorObject {
            coordinate: TCoordinate::new('t', coord),
            internal_type: first_tile as u64,
            zone_id: TCoordinate::new(
                'f',
                Coordinate {
                    0: coord.0 / ZONE_SIZE as i64,
                    1: coord.1 / ZONE_SIZE as i64,
                },
            ),
        };

        //place the tile using our SignificantComponent trait
        Tile::place(&mut commands, to_place, 't', coord, &tiles);
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

        Tile::remove(&mut commands, coord, &tiles);
        send_message!(
            Some('i'),
            message_queue,
            format!("Removing tiles at: ({}, {})", coord.0, coord.1)
        );
    }


    if input.just_pressed(KeyCode::ArrowRight) {
        //cycles through the spritesheet to the right
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type + 1) % (MAX_SPRITESHEET_ITEMS as u64);
    }
    if input.just_pressed(KeyCode::ArrowLeft) {
        //cycles through the spritesheet to the left
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type + (MAX_SPRITESHEET_ITEMS as u64) - 1) %
            (MAX_SPRITESHEET_ITEMS as u64);
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        //cycles through the spritesheet up
        current_editor_object.0.tile_type = (MAX_SPRITESHEET_ITEMS - SPRITESHEET_WIDTH) as u64 + (current_editor_object.0.tile_type % SPRITESHEET_WIDTH as u64);
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        //cycles through the spritesheet down
        current_editor_object.0.tile_type =
            (current_editor_object.0.tile_type + (SPRITESHEET_WIDTH as u64)) %
            (MAX_SPRITESHEET_ITEMS as u64);
    }
}

fn load_spritesheet(mut commands: Commands, asset_server: Res<AssetServer>) {
    //load the tilesheet for this mode
    let tex_path = PathBuf::from("textures/tiles/tilesheet.png");

    //load happens here
    let texture = asset_server.load(tex_path);

    //insert the texture handle into the resources for easy access later
    commands.insert_resource(TilesheetHandle(texture.clone()));
}

fn exit_tilemode(mut commands: Commands, mut tile_state: ResMut<NextState<EditorState>>) {
    tile_state.set(EditorState::Editing(EditingMode::None));
    println!("Exiting Tile Editing Mode");

    //remove the CurrentEditorObject resource
    commands.insert_resource(PlaceholderTile(Tile::new()));
}

/// A component that marks an entity as part of the tile editing UI.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(UIItem)]
struct TileModeUI;

/// A component to track some basic info about a tile
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Tile {
    pub tile_type: u64,
    pub coordinate: Coordinate,
}
impl Tile {
    fn new() -> Self {
        Self {
            tile_type: 0,
            coordinate: Coordinate(0, 0),
        }
    }
}
impl Default for Tile {
    fn default() -> Self {
        Self::new()
    }
}

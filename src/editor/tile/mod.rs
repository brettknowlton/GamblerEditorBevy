pub mod ui;

use bevy::prelude::*;
use bevy::sprite::Anchor;
use tools::SignificantComponent;
use std::path::PathBuf;
use crate::{ utilities::*, EditorObject, TILE_SIZE };
use crate::consts::*;
use super::*;

use ui::*;




fn load_spritesheet(asset_server: Res<AssetServer>, mut message_queue: ResMut<EditorBottomBarQueuedMessages>, mut textures: ResMut<TextureHandles>) {
    //load the tilesheet for this mode
    let tex_path = PathBuf::from("textures/tiles/tilesheet.png");

    send_message!(Some('i'), message_queue, format!("Tilesheet Loaded: \"{}\"", &tex_path.clone().display()));
    //load happens here
    textures.0.insert('t',asset_server.load(tex_path));
}

fn tilemode_keybinds(
    mut commands: Commands,

    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    input: Res<ButtonInput<KeyCode>>,

    crosshairs: Query<(&Transform, &Crosshair)>,
    tiles: Query<(Entity, &EditorObject), With<Tile>>,
    gridsnap: Res<State<GridSnap>>,
    mut selected_tile_id: ResMut<SelectedTileID>,
    mut needs_update: ResMut<TileUpdateNeeded>

)
{
    //"P" handles placement of a tile and adding it to the scene
    //places the first tile in the selection rect
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        let (t, _) = crosshairs.single();

        //get the coordinate of the crosshair AND snap it to the grid if gridsnap is enabled
        let mut coord = Coordinate::from(t.translation);
        if gridsnap.get() == &GridSnap::Enabled {
            coord = snap_coordinate_to_grid(coord);
        }

        let first_tile = selected_tile_id.id;

        let to_place = EditorObject {
            coordinate: TCoordinate::new('t', coord),
            internal_type: first_tile as u64,
        };

        //place the tile using our SignificantComponent trait
        Tile::place(&mut commands, to_place, 't', coord, &tiles);
        send_message!(Some('i'), message_queue, format!("Placed tile at: ({}, {})", coord.0, coord.1));
    }

    // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let (t, _) = crosshairs.single();
        let mut coord = Coordinate::from(t.translation);
        coord = snap_coordinate_to_grid(coord);

        Tile::remove(&mut commands, coord, &tiles);
        send_message!(Some('i'), message_queue, format!("Removing tiles at: ({}, {})", coord.0, coord.1));
    }

    if input.just_pressed(KeyCode::ArrowRight) {
        //pressing right increments selected_rect_id by 1, looping back to 0 if it goes over the max_spritesheet_items
        selected_tile_id.id = (selected_tile_id.id + 1) % MAX_SPRITESHEET_ITEMS;
        needs_update.0 = true;
    }
    if input.just_pressed(KeyCode::ArrowLeft) {
        //cycles through the spritesheet to the left
        selected_tile_id.id = (selected_tile_id.id + MAX_SPRITESHEET_ITEMS - 1) % MAX_SPRITESHEET_ITEMS;
        needs_update.0 = true;
    }
    if input.just_pressed(KeyCode::ArrowUp) {
        //cycles through the spritesheet up using the spritesheet width
        selected_tile_id.id = (selected_tile_id.id + MAX_SPRITESHEET_ITEMS - SPRITESHEET_WIDTH as u64) % MAX_SPRITESHEET_ITEMS;
        needs_update.0 = true;
    }
    if input.just_pressed(KeyCode::ArrowDown) {
        //cycles through the spritesheet down
        selected_tile_id.id = (selected_tile_id.id + SPRITESHEET_WIDTH as u64) % MAX_SPRITESHEET_ITEMS;
        needs_update.0 = true;
    }


}

fn exit_tilemode(mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {
    //remove the CurrentEditorObject resource
    // commands.insert_resource(PlaceholderObject(EditorObject::default()));
    send_message!(Some('i'), message_queue, "Exiting Tile Editing Mode".to_string());
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
            (update_placeholder::<Tile>, ui::spawn_tile_placeholder, ui::create_tilemode_ui).chain()
        )

        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (tilemode_keybinds, (update_placeholder::<Tile>, reset_update_needed).run_if(run_if_tile_update_needed))
                .chain()
                .run_if(in_state(EditorState::Editing(EditingComponent::Tile)))
        )

        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditingComponent::Tile)),
            (despawn_all::<TileModeUI>, exit_tilemode).chain()
        );
    //we could also take care of some post-exit cleanup here, like despawning all the UI elements by using the schedule OnEnter(EditorState::Inactive) and then despawning all the UI elements
}
//NOTHING BELOW THE PLUGINS!
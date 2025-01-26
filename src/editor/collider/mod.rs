mod ui;
pub use ui::*;

use bevy::prelude::*;
use tools::SignificantComponent;
use std::path::PathBuf;
use crate::{ utilities::*, EditorObject, TILE_SIZE };
use crate::consts::*;
use super::*;

fn init(mut spritesheets: ResMut<TextureHandles>, asset_server: Res<AssetServer>){
    let texpath= PathBuf::from("textures/tiles/collider_debug.png");
    spritesheets.0.insert('c', asset_server.load(texpath));

}

fn collidermode_keybinds(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,

    crosshairs: Query<(&Transform, &Crosshair)>,
    colliders: Query<(Entity, &EditorObject), With<Collider>>,
    gridsnap: Res<State<GridSnap>>,

    mut message_queue: ResMut<EditorBottomBarQueuedMessages>
) {
    //"P" handles placement of a collider and adding it to the scene
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        let (t, _) = crosshairs.single();

        //get the coordinate of the crosshair AND snap it to the grid if gridsnap is enabled
        let mut coord = Coordinate::from(t.translation);
        if gridsnap.get() == &GridSnap::Enabled {
            coord = snap_coordinate_to_grid(coord);
        }

        //define the editor object to place
        let to_place = EditorObject {
            coordinate: TCoordinate::new('C', coord),
            internal_type: 0,
        };

        //place the tile using our SignificantComponent trait
        Collider::place(&mut commands, to_place, coord, &colliders);
        send_message!(Some('i'), message_queue, format!("Placed collider at: ({}, {})", coord.0, coord.1));
    }


    // "L" handles removal of a collider from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let (t, _) = crosshairs.single();
        let mut coord = Coordinate::from(t.translation);
        coord = snap_coordinate_to_grid(coord);

        Tile::remove(&mut commands, coord, &colliders);
        send_message!(Some('i'), message_queue, format!("Removing colliders at: ({}, {})", coord.0, coord.1));
    }

}

fn exit_collidermode(mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {

    send_message!(Some('i'), message_queue, "Exiting Collider Editing Mode".to_string());

    // //remove the CurrentEditorObject resource
    // commands.insert_resource(PlaceholderObject(EditorObject::default()));
}

/// A component that marks an entity as part 
/// 
/// of the tile editing UI.
#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(UIItem)]
struct ColliderModeUI;

/// A component to track some basic info about a tile
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Collider {
    pub internal_type: u64,
    pub coordinate: TCoordinate,
    pub rect: Rect,
}
impl Collider {
    fn new() -> Self {
        Self {
            internal_type: 0,
            coordinate: TCoordinate{type_char: 'C', coord: Coordinate{0: 0, 1: 0}},
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}
impl Default for Collider {
    fn default() -> Self {
        Self::new()
    }
}
impl SignificantComponent for Collider {

    fn place_rectangle(_rect: Rect, _commands: Commands) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }
}


pub fn collidermode_plugin(app: &mut App) {
    app
        .register_type::<Collider>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        .register_type::<ColliderModeUI>()

        //startup systems (may need to be moved from here to maintain order)
        .add_systems(Startup, init)

        //OnEnter systems
        .add_systems(OnEnter(EditorState::Editing(EditingComponent::Collider)), (update_placeholder::<Collider>, create_collidermode_ui).chain())

        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (collidermode_keybinds, super::ui::update_placeholder::<Collider>)
                .chain()
                .run_if(in_state(EditorState::Editing(EditingComponent::Collider)))
        )

        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditingComponent::Collider)), 
            (
                despawn_all::<ColliderModeUI>,
                exit_collidermode
            ).chain()
        );
}
//NOTHING BELOW THE PLUGINS
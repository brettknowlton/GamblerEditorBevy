pub mod ui;

use bevy::prelude::*;
use tools::SignificantComponent;
use std::path::PathBuf;
use crate::{ utilities::*, EditorObject, TILE_SIZE };
use crate::consts::*;
use super::*;

// #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
// pub enum TileEditorState {
//     #[default]
//     Inactive,
//     Active,
// }

pub fn collidermode_plugin(app: &mut App) {
    app
        .register_type::<Collider>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        .register_type::<ColliderModeUI>()
        .insert_resource(PlaceholderObject(EditorObject::default()))

        //startup systems (may need to be moved from here to maintain order)

        //OnEnter systems
        .add_systems(OnEnter(EditorState::Editing(EditingMode::Tile)), (init_collidermode, ui::show_collider_placeholder, ui::create_collidermode_ui).chain())

        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (collidermode_keybinds, ui::update_placeholder)
                .chain()
                .run_if(in_state(EditorState::Editing(EditingMode::Collider)))
        )

        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditingMode::Collider)), 
            (
                despawn_all::<ColliderModeUI>,
                exit_collidermode
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

fn init_collidermode(mut message_queue: ResMut<EditorBottomBarQueuedMessages>
) {
    send_message!(Some('i'), message_queue, "Entering Collider Editing Mode".to_string());
}

fn collidermode_keybinds(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,

    crosshairs: Query<(&Transform, &Crosshair)>,
    colliders: Query<(Entity, &Collider)>,
    current_editor_object: Res<PlaceholderObject>
) {
    //"P" handles placement of a collider and adding it to the scene
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        let (t, _) = crosshairs.single();
        let focused_item = &current_editor_object.0;
        let mut coord = Coordinate::from(t.translation);
        let pushover = 1000 * ((TILE_SIZE * TILE_SCALE) as i64); //this effectively offsets all my tiles 1000 to the right and down, so that negative coordinates arent a problem and rounds to the nearest integer
        coord = Coordinate(
            ((coord.0 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover,
            ((coord.1 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover
        );

        // println!("coords: {}{}", coord.0, coord.1);

        //check if a tile already exists at this location and remove it if it does
        if let Some(item) = colliders.iter().find(|(_, c)| c.coordinate == coord) {
            //remove the old tile
            commands.entity(item.0).despawn();
        }

        Collider::place(&mut commands, focused_item.clone(), coord);
    }


    // "L" handles removal of a collider from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let (t, _) = crosshairs.single();
        let mut coord = Coordinate::from(t.translation);
        //"floor" the coordinate to the nearest tile grid space in a way that (kind of) respects the negative coordinate space, just dont place anything more than 1000 tiles away from the origin until I can figure that out
        let pushover = 1000 * ((TILE_SIZE * TILE_SCALE) as i64);
        coord = Coordinate(
            ((coord.0 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover,
            ((coord.1 + pushover) / ((TILE_SIZE * TILE_SCALE) as i64)) *
                ((TILE_SIZE * TILE_SCALE) as i64) -
                pushover
        );

        // println!("coords: {}{}", coord.0, coord.1);

        Collider::remove(&mut commands, coord, colliders);
    }

}

fn exit_collidermode(mut commands: Commands, mut tile_state: ResMut<NextState<EditorState>>, mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {
    tile_state.set(EditorState::Editing(EditingMode::None));
    send_message!(Some('i'), message_queue, "Exiting Collider Editing Mode".to_string());
    //remove the CurrentEditorObject resource
    commands.insert_resource(PlaceholderObject(EditorObject::default()));
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
    pub coordinate: Coordinate,
    pub rect: Rect,
}
impl Collider {
    fn new() -> Self {
        Self {
            internal_type: 0,
            coordinate: Coordinate(0, 0),
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
    fn get_coordinate(&self) -> Coordinate {
        self.coordinate
    }
    
    fn place(commands: &mut Commands, item: EditorObject, coord: Coordinate){
        commands.spawn((
            Collider {
                internal_type: item.internal_type,
                coordinate: coord,
                rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            },
            Transform {
                translation: Vec3::new(coord.0 as f32, coord.1 as f32, -5.0),
                scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 1.0),
                ..default()
            },
            EditorObject {
                coordinate: TCoordinate::new('T', coord),
                internal_type: item.internal_type,
            },
        ));
    }

    fn use_rectangle_tool(_rect: Rect) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }
}
// impl SignificantComponent for Tile {
//     fn use_rectangle_tool(rect: Rect) {
//         //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
//         todo!();
//     }
// }

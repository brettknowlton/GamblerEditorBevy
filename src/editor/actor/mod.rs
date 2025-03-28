#[macro_use]
mod ui;
use bevy::gizmos::grid;
pub use ui::*;

pub mod player;
pub use player::*;

use bevy::prelude::*;
use tools::SignificantComponent;
use std::path::PathBuf;
use crate::{ utilities::*, EditorObject, TILE_SIZE };
use crate::consts::*;
use super::*;

use bevy_rapier2d::prelude::*;

#[derive(Component, Reflect, Debug, Clone, PartialEq,)]
#[require(EditorObject)]
pub struct Actor {
    pub internal_type: u64,
    pub coordinate: TCoordinate,
    pub rect: Rect,
}

impl Actor {
    pub fn new() -> Self {
        Self {
           internal_type: 0,
            coordinate: TCoordinate::new('a', Coordinate{0: 0, 1: 0}),
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}

impl Default for Actor {
    fn default() -> Self {
        Self::new()
    }
}
impl SignificantComponent for Actor {
    fn place_rectangle(_rect: Rect, _commands: Commands) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }

    fn from_rect(_rect: Rect, _coord: Coordinate) -> Self {
        Self::new()
    }
}

pub fn actor_mode_keybinds(
    // editor_state: ResMut<State<EditorState>>,
    // mut next_editor_state: ResMut<NextState<EditorState>>,
    // mut next_game_state: ResMut<NextState<GameState>>,
    // input: Res<ButtonInput<KeyCode>>,
    // crosshairs: Query<(&Transform, &Crosshair)>,
    // mut actors:Query<(Entity, &EditorObject), With<Actor>>,
    // gridsnap: Res<State<GridSnap>>,
    // mut commands: &mut Commands,
    

    // mut message_queue: ResMut<EditorBottomBarQueuedMessages>
    mut commands: Commands,

    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    input: Res<ButtonInput<KeyCode>>,

    crosshairs: Query<(&Transform, &Crosshair)>,
    actors: Query<(Entity, &EditorObject), With<Actor>>,
    gridsnap: Res<State<GridSnap>>,
    // mut selected_tile_id: ResMut<SelectedTileID>,

    // mut placeholder_update_ev: EventWriter<UpdatePlaceholderEvent>

) {

    //"P" handles placement of an actor and adding it to the scene
    //places the first actor in the selection rect
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        let (t, _) = crosshairs.single();

        //get the coordinate of the crosshair AND snap it to the grid if gridsnap is enabled
        let mut coord = Coordinate::from(t.translation);
        if gridsnap.get() == &GridSnap::Enabled {
            coord = snap_coordinate_to_grid(coord);
        }

        let def_actor_id = Actor::new().internal_type;

        let to_place = EditorObject {
            coordinate: TCoordinate::new('a', coord),
            internal_type: def_actor_id as u64,
            zone_id: TCoordinate::new('a', Coordinate{0: coord.0 / ZONE_SIZE as i64, 1: coord.1 / ZONE_SIZE as i64}),
        };

        //place the tile using our SignificantComponent trait
        Actor::place(&mut commands, to_place, 'a', coord, &actors);
        send_message!(Some('i'), message_queue, format!("Placed actor at: ({}, {})", coord.0, coord.1));
    }

    // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let (t, _) = crosshairs.single();
        let mut coord = Coordinate::from(t.translation);
        coord = snap_coordinate_to_grid(coord);

        Actor::remove(&mut commands, coord, &actors);
        send_message!(Some('i'), message_queue, format!("Removing actor at: ({}, {})", coord.0, coord.1));
    }
}


pub fn actormode_plugin(app: &mut App) {
    app
        .register_type::<Player>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        .register_type::<ui::ActorModeUI>()

        //startup systems (may need to be moved from here to maintain order)
        // .add_systems(Startup, init)

        //OnEnter systems
        .add_systems(OnEnter(EditorState::Editing(EditingComponent::Collider)), (crate::ui::update_placeholder::<Actor>, ui::create_collidermode_ui).chain())

        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (super::ui::update_placeholder::<Actor>, actor_mode_keybinds).chain()
                .run_if(in_state(EditorState::Editing(EditingComponent::Actor)))
        )


        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditingComponent::Collider)), 
            (
                despawn_all::<ui::ActorModeUI>,
            ).chain()
        );
}
//NOTHING BELOW THE PLUGINS
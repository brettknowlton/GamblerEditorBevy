pub mod player;
use player::Player;

use super::*;
use crate::editor_modes::significant_component::SignificantComponent;
use crate::message_display::MessageDisplay;
use crate::ui::{self, ToolingMenuItem};
use crate::{configure_tooling_menu, Crosshair, EditorState, TextureHandles, ToolingMenuState};
use std::path::PathBuf;

use bevy_rapier2d::prelude::*;

fn populate_actor_tooling_menu(mut tooling_menu: ResMut<ToolingMenuState>) {
    configure_tooling_menu(
        &mut tooling_menu,
        "Actor Parts",
        Some(0),
        vec![ToolingMenuItem {
            id: 0,
            label: "Default Actor".to_string(),
            texture_key: Some(EditorObjectKind::Actor),
            rect: None,
        }],
    );
}

fn init(mut spritesheets: ResMut<TextureHandles>, asset_server: Res<AssetServer>) {
    let texpath = PathBuf::from("textures/player/PlayerHD.png");
    spritesheets
        .0
        .insert(EditorObjectKind::Actor, asset_server.load(texpath));
}

#[derive(Component, Reflect, Debug, Clone, PartialEq)]
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
            coordinate: TCoordinate::new(EditorObjectKind::Actor, Coordinate::new_world_space(0, 0)),
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

    fn at_coordinate(_coord: Coordinate) -> Self {
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

    mut bottom_bar: ResMut<MessageDisplay>,
    input: Res<ButtonInput<KeyCode>>,

    crosshairs: Query<(&Transform, &Crosshair)>,
    actors: Query<(Entity, &EditorObject), With<Actor>>,
    // mut selected_tile_id: ResMut<SelectedTileID>,

    // mut placeholder_update_ev: EventWriter<UpdatePlaceholderEvent>
) {
    //"P" handles placement of an actor and adding it to the scene
    //places the first actor in the selection rect
    if input.just_pressed(KeyCode::KeyP) {
        let Ok((crosshair_location, _)) = crosshairs.single() else {
            return;
        };

        let coord = Coordinate::from(crosshair_location.translation).snap_to_grid();
        let to_place = EditorObject::new(EditorObjectKind::Actor, coord, EditorObjectKind::Actor);

        Actor::place(&mut commands, to_place, &actors);
        bottom_bar.send_place_eo_message("actor", coord);
    }

    // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let Ok((t, _)) = crosshairs.single() else {
            return;
        };
        let coord = Coordinate::from(t.translation).snap_to_grid();

        Actor::remove(&mut commands, coord, EditorObjectKind::Actor, &actors);
        bottom_bar.send_remove_eo_message("actor", coord);
    }
}

fn exit_actormode(mut bottom_bar: ResMut<MessageDisplay>) {
    bottom_bar.send_mode_exit_message("Actor");
}

pub fn actormode_plugin(app: &mut App) {
    app.register_type::<Player>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        //startup systems (may need to be moved from here to maintain order)
        .add_systems(Startup, init)
        //OnEnter systems
        .add_systems(
            OnEnter(EditorState::Editing(EditorObjectKind::Actor)),
            (
                populate_actor_tooling_menu,
                crate::ui::update_placeholder::<Actor>,
            )
                .chain(),
        )
        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (ui::update_placeholder::<Actor>, actor_mode_keybinds)
                .chain()
                .run_if(in_state(EditorState::Editing(EditorObjectKind::Actor))),
        )
        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditorObjectKind::Actor)),
            (exit_actormode).chain(),
        );
}
//NOTHING BELOW THE PLUGINS

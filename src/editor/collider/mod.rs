mod ui;
pub use ui::*;

use super::*;
use crate::ui::ToolingMenuItem;
use crate::{EditorObject, TILE_SIZE};
use bevy::prelude::*;
use std::path::PathBuf;
use tools::SignificantComponent;

fn populate_collider_tooling_menu(mut tooling_menu: ResMut<ToolingMenuState>) {
    tooling_menu.title = "Collider Parts".to_string();
    tooling_menu.visible = true;
    tooling_menu.selected_item_id = Some(0);
    tooling_menu.items = vec![ToolingMenuItem {
        id: 0,
        label: "Solid".to_string(),
        texture_key: Some(EditorObjectKind::Collider),
        rect: Some(Rect::new(0.0, 0.0, TILE_SIZE as f32, TILE_SIZE as f32)),
    }];
}

fn init(mut spritesheets: ResMut<TextureHandles>, asset_server: Res<AssetServer>) {
    let texpath = PathBuf::from("textures/tiles/collider_debug.png");
    spritesheets
        .0
        .insert(EditorObjectKind::Collider, asset_server.load(texpath));
}

fn collidermode_keybinds(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,

    crosshairs: Query<(&Transform, &Crosshair)>,
    colliders: Query<(Entity, &EditorObject), With<ColliderObject>>,
    gridsnap: Res<State<GridSnap>>,

    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
) {
    //"P" handles placement of a collider and adding it to the scene
    if input.just_pressed(KeyCode::KeyP) {
        //clean up the bevy query overhead
        let Ok((t, _)) = crosshairs.single() else {
            return;
        };

        //get the coordinate of the crosshair AND snap it to the grid if gridsnap is enabled
        let mut coord = Coordinate::from(t.translation);
        if gridsnap.get() == &GridSnap::Enabled {
            coord = snap_coordinate_to_grid(coord);
        }

        //define the editor object to place
        let to_place = EditorObject {
            coordinate: coord,
            kind: EditorObjectKind::Collider,
            internal_kind: 0,
            zone_id: TCoordinate::new(
                EditorObjectKind::Other,
                Coordinate {
                    0: coord.0 / ZONE_SIZE as i64,
                    1: coord.1 / ZONE_SIZE as i64,
                },
            ),
        };

        //place the tile using our SignificantComponent trait
        ColliderObject::place(
            &mut commands,
            to_place,
            &colliders,
        );
        send_message!(
            Some('i'),
            message_queue,
            format!("Placed collider at: ({}, {})", coord.0, coord.1)
        );
    }

    // "L" handles removal of a collider from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let Ok((t, _)) = crosshairs.single() else {
            return;
        };
        let mut coord = Coordinate::from(t.translation);
        coord = snap_coordinate_to_grid(coord);

        ColliderObject::remove(&mut commands, coord, EditorObjectKind::Collider,&colliders);
        send_message!(
            Some('i'),
            message_queue,
            format!("Removing colliders at: ({}, {})", coord.0, coord.1)
        );
    }
}

fn exit_collidermode(mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {
    send_message!(
        Some('i'),
        message_queue,
        "Exiting Collider Editing Mode".to_string()
    );

    // //remove the CurrentEditorObject resource
    // commands.insert_resource(PlaceholderObject(EditorObject::default()));
}

/// A component to track some basic info about a tile
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ColliderObject {
    pub internal_type: u64,
    pub coordinate: TCoordinate,
    pub rect: Rect,
}

impl ColliderObject {
    fn new() -> Self {
        Self {
            internal_type: 0,
            coordinate: TCoordinate {
                kind: EditorObjectKind::Collider,
                coord: Coordinate { 0: 0, 1: 0 },
            },
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}
impl Default for ColliderObject {
    fn default() -> Self {
        Self::new()
    }
}

impl SignificantComponent for ColliderObject {
    fn place_rectangle(_rect: Rect, _commands: Commands) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }
    fn from_rect(rect: Rect, coord: Coordinate) -> Self {
        Self {
            internal_type: 0,
            coordinate: TCoordinate {
                kind: EditorObjectKind::Collider,
                coord,
            },
            rect,
        }
    }
}

pub fn collidermode_plugin(app: &mut App) {
    app.register_type::<ColliderObject>()
        .register_type::<Coordinate>()
        .register_type::<TCoordinate>()
        .register_type::<ColliderModeUI>()
        //startup systems (may need to be moved from here to maintain order)
        .add_systems(Startup, init)
        //OnEnter systems
        .add_systems(
            OnEnter(EditorState::Editing(EditingComponent::Collider)),
            (
                populate_collider_tooling_menu,
                crate::ui::update_placeholder::<ColliderObject>,
            )
                .chain(),
        )
        //Update systems, that run only while TileEditor is active
        .add_systems(
            Update,
            (
                collidermode_keybinds,
                super::ui::update_placeholder::<ColliderObject>,
            )
                .chain()
                .run_if(in_state(EditorState::Editing(EditingComponent::Collider))),
        )
        //OnExit systems
        .add_systems(
            OnExit(EditorState::Editing(EditingComponent::Collider)),
            (despawn_all::<ui::ColliderModeUI>, exit_collidermode).chain(),
        );
}
//NOTHING BELOW THE PLUGINS

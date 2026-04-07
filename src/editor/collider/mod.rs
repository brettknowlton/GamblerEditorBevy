mod ui;
pub use ui::*;

use super::*;
use crate::ui::ToolingMenuItem;
use crate::{EditorObject, TILE_SIZE};
use bevy::prelude::*;
use std::path::PathBuf;
use tools::SignificantComponent;

fn populate_collider_tooling_menu(mut tooling_menu: ResMut<ToolingMenuState>) {
    configure_tooling_menu(
        &mut tooling_menu,
        "Collider Parts",
        Some(0),
        vec![ToolingMenuItem {
            id: 0,
            label: "Solid".to_string(),
            texture_key: Some(EditorObjectKind::Collider),
            rect: Some(Rect::new(0.0, 0.0, TILE_SIZE as f32, TILE_SIZE as f32)),
        }],
    );
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
        let Ok((t, _)) = crosshairs.single() else {
            return;
        };

        let coord = snapped_coordinate_from_translation(t.translation, &gridsnap);
        let to_place = build_editor_object(
            EditorObjectKind::Collider,
            0,
            coord,
            EditorObjectKind::Other,
        );

        ColliderObject::place(&mut commands, to_place, &colliders);
        send_placement_message(&mut message_queue, "collider", coord);
    }

    // "L" handles removal of a collider from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
    if input.just_pressed(KeyCode::KeyL) {
        let Ok((t, _)) = crosshairs.single() else {
            return;
        };
        let coord = snap_coordinate_to_grid(Coordinate::from(t.translation));

        ColliderObject::remove(&mut commands, coord, EditorObjectKind::Collider, &colliders);
        send_removal_message(&mut message_queue, "colliders", coord);
    }
}

fn exit_collidermode(mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {
    send_mode_exit_message(&mut message_queue, "Collider");
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

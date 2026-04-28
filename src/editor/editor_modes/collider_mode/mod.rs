use bevy::window::PrimaryWindow;

use super::*;
use crate::bounding_box::BoundingBox;
use crate::editor_modes::editor_mode::EditorModePlugin;
use crate::editor_modes::significant_component::SignificantComponent;
use crate::message_display::MessageDisplay;
use crate::{
    configure_tooling_menu, mouse_state::MouseState, Crosshair, CustomInput, EditorState,
    SelectedTileID, TextureHandles, ToolingMenuItem, ToolingMenuState, TILE_SIZE,
};
use std::path::PathBuf;

/// A component to track some basic info about a tile
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ColliderObject {
    pub kind: EditorObjectKind,
    pub coord: Coordinate,
    pub bounding_box: BoundingBox,
}

impl Default for ColliderObject {
    fn default() -> Self {
        Self {
            kind: EditorObjectKind::Collider,
            ..default()
        }
    }
}

impl SignificantComponent for ColliderObject {
    fn place_rectangle(_rect: Rect, _commands: Commands) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }
    fn at_coordinate(coord: Coordinate) -> Self {
        Self {
            kind: EditorObjectKind::Collider,
            coord,
            bounding_box: BoundingBox::from_coordinate(coord),
        }
    }

    fn relevant_editor_object(&self) -> EditorObjectKind {
        EditorObjectKind::Collider
    }
    fn to_type_string(&self) -> String {
        "collider_object".to_string()
    }
}

pub struct ColliderModePlugin;

impl Plugin for ColliderModePlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin::<ColliderObject>(app);
    }
}

impl EditorModePlugin for ColliderModePlugin {
    fn mode() -> EditorState {
        EditorState::Editing(EditorObjectKind::Collider)
    }

    fn init(
        mut spritesheets: ResMut<TextureHandles>,
        mut bottom_bar: ResMut<MessageDisplay>,
        asset_server: Res<AssetServer>,
    ) {
        let tex_path = PathBuf::from("textures/tiles/collider_debug.png");

        bottom_bar.send_message(format!("Tilesheet Loaded: \"{}\"", tex_path.display()));

        spritesheets
            .0
            .insert(EditorObjectKind::Collider, asset_server.load(tex_path));
    }

    fn enter_mode(
        mut tooling_menu: ResMut<ToolingMenuState>,
        _selected_item_id: Res<SelectedTileID>,
    ) {
        configure_tooling_menu(
            &mut tooling_menu,
            "Collider",
            Some(0),
            vec![ToolingMenuItem {
                id: 0,
                label: "Solid".to_string(),
                texture_key: Some(EditorObjectKind::Collider),
                rect: Some(Rect::new(0.0, 0.0, TILE_SIZE as f32, TILE_SIZE as f32)),
            }],
        );
        println!("populated collider tooling menu");
    }

    fn exit_mode(mut bottom_bar: ResMut<MessageDisplay>) {
        bottom_bar.send_mode_exit_message("Collider");
    }

    fn mode_keybinds<T: SignificantComponent>(
        mut commands: Commands,
        input: Res<ButtonInput<KeyCode>>,

        crosshair: Single<(&Transform, &Crosshair)>,
        items: Query<(Entity, &EditorObject), With<T>>,
        _selected_tile_id: ResMut<SelectedTileID>,

        mut bottom_bar: ResMut<MessageDisplay>,
        _next_editor_state: ResMut<NextState<EditorState>>,
    ) {
        //"P" handles placement of a collider and adding it to the scene
        if input.just_pressed(KeyCode::KeyP) {
            let coord = Coordinate::from_vec3(crosshair.0.translation).snap_to_grid();
            let to_place = EditorObject::new(EditorObjectKind::Collider, coord);

            ColliderObject::place(&mut commands, to_place, &items);
            bottom_bar.send_place_eo_message("collider", coord);
        }

        // "L" handles removal of a collider from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
        if input.just_pressed(KeyCode::KeyL) {
            let coord = Coordinate::from_vec3(crosshair.0.translation).snap_to_grid();

            ColliderObject::remove(&mut commands, coord, EditorObjectKind::Collider, &items);
            bottom_bar.send_remove_eo_message("colliders", coord);
        }
    }

    fn mode_click<T: SignificantComponent + Component + Default>(
        commands: Commands,
        window: Single<&Window, With<PrimaryWindow>>,
        camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
        items: Query<(Entity, &EditorObject), With<T>>,
        _selected_tile_id: ResMut<SelectedTileID>,
        mouse_state: Res<MouseState>,
        bottom_bar: ResMut<MessageDisplay>,
    ) {
        if let Some(mouse_pos) = window.cursor_position() {
            let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, mouse_pos) else {
                return;
            };

            let snapped_coord: Coordinate =
                Coordinate::new_world_space(world_pos.x as i64, world_pos.y as i64).snap_to_grid();

            mouse_state.drag_action(
                commands,
                snapped_coord,
                EditorObjectKind::Collider,
                bottom_bar,
                items,
            );
        }
    }

    fn get_mode_kb() -> Vec<(CustomInput, String)> {
        vec![
            (CustomInput::Single(KeyCode::KeyP), "Place Collider".into()),
            (CustomInput::Single(KeyCode::KeyL), "Remove Collider".into()),
            (CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into()),
        ]
    }
}

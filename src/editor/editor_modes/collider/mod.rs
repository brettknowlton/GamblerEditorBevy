use bevy::window::PrimaryWindow;

use super::*;
use crate::bounding_box::BoundingBox;
use crate::editor_modes::editor_mode::EditorModePlugin;
use crate::editor_modes::significant_component::SignificantComponent;
use crate::message_display::{send_message, MessageDisplay};
use crate::{
    configure_tooling_menu, ui, AvailableKeybinds, Crosshair, CustomInput, Dragging, Editor,
    EditorState, TextureHandles, ToolingMenuItem, ToolingMenuState, TILE_SIZE,
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
}

pub struct ColliderModePlugin;

impl Plugin for ColliderModePlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin(app);
    }
}

impl ColliderModePlugin {
    fn populate_tooling_menu(mut tooling_menu: ResMut<ToolingMenuState>) {
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
}

impl EditorModePlugin for ColliderModePlugin {
    fn build_plugin(app: &mut App) {
        app.register_type::<ColliderObject>()
            .register_type::<Coordinate>()
            .register_type::<TCoordinate>()
            //startup systems (may need to be moved from here to maintain order)
            .add_systems(Startup, Self::init)
            //OnEnter systems
            .add_systems(
                OnEnter(EditorState::Editing(EditorObjectKind::Collider)),
                (
                    Self::populate_tooling_menu,
                    crate::ui::update_placeholder::<ColliderObject>,
                    Self::add_mode_kb,
                )
                    .chain(),
            )
            .add_systems(
                OnExit(EditorState::Editing(EditorObjectKind::Collider)),
                Self::remove_mode_kb,
            )
            //Update systems, that run only while TileEditor is active
            .add_systems(
                Update,
                (
                    Self::mode_keybinds::<ColliderObject>,
                    (Self::mode_click::<ColliderObject>).run_if(Editor::editor_is_dragging),
                    ui::update_placeholder::<ColliderObject>,
                )
                    .chain()
                    .run_if(in_state(EditorState::Editing(EditorObjectKind::Collider))),
            )
            //OnExit systems
            .add_systems(
                OnExit(EditorState::Editing(EditorObjectKind::Collider)),
                (Self::exit_mode,).chain(),
            );
    }

    fn init(
        mut spritesheets: ResMut<TextureHandles>,
        mut bottom_bar: ResMut<MessageDisplay>,
        asset_server: Res<AssetServer>,
    ) {
        let tex_path = PathBuf::from("textures/tiles/collider_debug.png");

        send_message!(
            Some('i'),
            bottom_bar.queue,
            format!("Tilesheet Loaded: \"{}\"", &tex_path.clone().display())
        );

        spritesheets
            .0
            .insert(EditorObjectKind::Collider, asset_server.load(tex_path));
    }

    fn exit_mode(mut bottom_bar: ResMut<MessageDisplay>) {
        bottom_bar.send_mode_exit_message("Collider");
    }

    fn mode_keybinds<T: SignificantComponent + Component + Default>(
        mut commands: Commands,

        mut bottom_bar: ResMut<MessageDisplay>,
        input: Res<ButtonInput<KeyCode>>,

        crosshair: Single<(&Transform, &Crosshair)>,
        items: Query<(Entity, &EditorObject), With<T>>,
        _selected_tile_id: ResMut<crate::SelectedTileID>,
    ) {
        //"P" handles placement of a collider and adding it to the scene
        if input.just_pressed(KeyCode::KeyP) {
            let coord = Coordinate::from(crosshair.0.translation).snap_to_grid();
            let to_place =
                EditorObject::new(EditorObjectKind::Collider, coord, EditorObjectKind::Other);

            ColliderObject::place(&mut commands, to_place, &items);
            bottom_bar.send_place_eo_message("collider", coord);
        }

        // "L" handles removal of a collider from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
        if input.just_pressed(KeyCode::KeyL) {
            let coord = Coordinate::from(crosshair.0.translation).snap_to_grid();

            ColliderObject::remove(
                &mut commands,
                coord,
                EditorObjectKind::Tile(TileID::Any),
                &items,
            );
            bottom_bar.send_remove_eo_message("colliders", coord);
        }
    }

    fn mode_click<T: SignificantComponent + Component + Default>(
        mut commands: Commands,
        window: Single<&Window, With<PrimaryWindow>>,
        camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
        items: Query<(Entity, &EditorObject), With<T>>,
        _selected_tile_id: Res<crate::SelectedTileID>,
        dragging: Res<Dragging>,
        bottom_bar: ResMut<MessageDisplay>,
        mouse_mode: ResMut<ui::MouseToolState>,
    ) {
        if let Some(mouse_pos) = window.cursor_position() {
            let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, mouse_pos) else {
                return;
            };

            let snapped_coord: Coordinate =
                Coordinate::new_world_space(world_pos.x as i64, world_pos.y as i64).snap_to_grid();

            dragging.drag_action(
                &mut commands,
                mouse_mode.current,
                snapped_coord,
                EditorObjectKind::Collider,
                bottom_bar,
                &items,
            );
        }
    }

    fn add_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
        available_keybinds
            .add_keycode(CustomInput::Single(KeyCode::KeyL), "Remove Collider".into());
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyP), "Place Collider".into());
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into());
        println!("populated collider keybinds");
    }
}

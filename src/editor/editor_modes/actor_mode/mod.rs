pub mod actor;
use actor::Actor;

pub mod animation;

pub mod player;
use player::Player;

use super::*;
use crate::editor_modes::significant_component::SignificantComponent;
use crate::message_display::MessageDisplay;
use crate::ui::ToolingMenuItem;
use crate::{
    configure_tooling_menu, AvailableKeybinds, Crosshair, CustomInput, EditorState, SelectedTileID,
    TextureHandles, ToolingMenuState,
};
use std::path::PathBuf;

use bevy_rapier2d::prelude::*;

pub struct ActorModePlugin;

impl Plugin for ActorModePlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin::<Actor>(app);
    }
}

impl EditorModePlugin for ActorModePlugin {
    fn mode() -> EditorState {
        EditorState::Editing(EditorObjectKind::Actor)
    }

    fn modify_app(app: &mut App) -> &mut App {
        app.register_type::<Player>()
    }

    fn init(
        mut spritesheets: ResMut<TextureHandles>,
        mut bottom_bar: ResMut<MessageDisplay>,
        asset_server: Res<AssetServer>,
    ) {
        //load the tilesheet for this mode
        let texpath = PathBuf::from("textures/player/player_anims-sheet.png");

        bottom_bar.send_message(format!(
            "Actor Spritesheet Loaded: \"{}\"",
            &texpath.clone().display()
        ));

        //load happens here
        spritesheets
            .0
            .insert(EditorObjectKind::Actor, asset_server.load(texpath));
    }

    fn add_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyL), "Remove Actor".into());
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyP), "Place Actor".into());
        available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into());
        println!("populated actor keybinds");
    }

    fn enter_mode(
        mut tooling_menu: ResMut<ToolingMenuState>,
        _selected_item_id: Res<crate::SelectedTileID>,
    ) {
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

    fn mode_keybinds<T: Component + SignificantComponent>(
        mut commands: Commands,
        input: Res<ButtonInput<KeyCode>>,

        crosshair: Single<(&Transform, &Crosshair)>,
        items: Query<(Entity, &EditorObject), With<T>>,
        _selected_tile_id: ResMut<SelectedTileID>,

        mut bottom_bar: ResMut<MessageDisplay>,
        _next_editor_state: ResMut<NextState<EditorState>>,
    ) {
        //"P" handles placement of an actor and adding it to the scene
        //places the first actor in the selection rect
        if input.just_pressed(KeyCode::KeyP) {
            let coord = Coordinate::from_vec3(crosshair.0.translation).snap_to_grid();
            let to_place = EditorObject::new(EditorObjectKind::Actor, coord);

            Actor::place(&mut commands, to_place, &items);
            bottom_bar.send_place_eo_message("actor", coord);
        }

        // "L" handles removal of a tile from the scene, similar to placing one just doesnt need to worry about the tile creation part afterwards
        if input.just_pressed(KeyCode::KeyL) {
            let coord = Coordinate::from_vec3(crosshair.0.translation).snap_to_grid();

            Actor::remove(&mut commands, coord, EditorObjectKind::Actor, &items);
            bottom_bar.send_remove_eo_message("actor", coord);
        }
    }
    fn exit_mode(mut bottom_bar: ResMut<MessageDisplay>) {
        bottom_bar.send_mode_exit_message("Actor");
    }

    fn get_mode_kb() -> Vec<(CustomInput, String)> {
        vec![
            (CustomInput::Single(KeyCode::KeyP), "Place Actor".into()),
            (CustomInput::Single(KeyCode::KeyL), "Remove Actor".into()),
            (CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into()),
        ]
    }
}

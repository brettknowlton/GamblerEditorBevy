use bevy::prelude::*;

use crate::{
    editor_modes::{editor_mode::EditorModePlugin, selection::SelectionRect},
    ui, EditorState,
};

pub struct NormalModePlugin;

impl Plugin for NormalModePlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin(app);
    }
}

impl EditorModePlugin for NormalModePlugin {
    fn mode() -> EditorState {
        EditorState::Normal
    }

    fn build_plugin(app: &mut App) {
        app
            //on entrance to this state, we give our placeholder object a handle to the default SignificantComponent of this mode- in normal mode this is a SelectionRect
            .add_systems(
                OnEnter(EditorState::Normal),
                (
                    ui::hide_tooling_menu,
                    ui::update_placeholder::<SelectionRect>,
                )
                    .chain(),
            )
            
            ;
    }

    fn init(
        _spritesheets: ResMut<crate::TextureHandles>,
        _bottom_bar: ResMut<crate::message_display::MessageDisplay>,
        _asset_server: Res<AssetServer>,
    ) {
        todo!()
    }

    fn exit_mode(bottom_bar: ResMut<crate::message_display::MessageDisplay>) {
        todo!()
    }

    fn mode_keybinds<T: Component + super::significant_component::SignificantComponent>(
        commands: Commands,

        bottom_bar: ResMut<crate::message_display::MessageDisplay>,
        input: Res<ButtonInput<KeyCode>>,

        crosshair: Single<(&Transform, &crate::Crosshair)>,
        tiles: Query<(Entity, &super::EditorObject), With<T>>,
        selected_tile_id: ResMut<crate::SelectedTileID>,
    ) {
        todo!()
    }

    fn mode_click<T: super::significant_component::SignificantComponent + Component + Default>(
        commands: Commands,
        window: Single<&Window, With<bevy::window::PrimaryWindow>>,
        camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
        items: Query<(Entity, &super::EditorObject), With<T>>,
        selected_tile_id: Res<crate::SelectedTileID>,
        dragging: Res<crate::Dragging>,
        bottom_bar: ResMut<crate::message_display::MessageDisplay>,
        mouse_mode: ResMut<crate::MouseToolState>,
    ) {
        todo!()
    }

    fn add_mode_kb(available_keybinds: ResMut<crate::AvailableKeybinds>) {
        todo!()
    }
}

impl NormalModePlugin {
    fn enter() {
        info!("Entered Normal Mode");
    }

    fn exit() {
        info!("Exited Normal Mode");
    }
}

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    AvailableKeybinds, Crosshair, Dragging, EditorState, MouseToolState, SelectedTileID, TextureHandles, editor_modes::{EditorObject, significant_component::SignificantComponent}, message_display::MessageDisplay
};

pub trait EditorModePlugin: Plugin {
    fn mode() -> EditorState;

    fn build(&self, app: &mut App) {
        Self::build_plugin(app);
    }

    fn build_plugin(app: &mut App);

    fn init(
        spritesheets: ResMut<TextureHandles>,
        bottom_bar: ResMut<MessageDisplay>,
        asset_server: Res<AssetServer>,
    );

    fn exit_mode(bottom_bar: ResMut<MessageDisplay>);

    fn mode_keybinds<T: Component + SignificantComponent>(
        commands: Commands,

        bottom_bar: ResMut<MessageDisplay>,
        input: Res<ButtonInput<KeyCode>>,

        crosshair: Single<(&Transform, &Crosshair)>,
        tiles: Query<(Entity, &EditorObject), With<T>>,
        selected_tile_id: ResMut<SelectedTileID>,
    );

    fn mode_click<T: SignificantComponent + Component + Default>(
        commands: Commands,
        window: Single<&Window, With<PrimaryWindow>>,
        camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
        items: Query<(Entity, &EditorObject), With<T>>,
        selected_tile_id: Res<SelectedTileID>,
        dragging: Res<Dragging>,
        bottom_bar: ResMut<MessageDisplay>,
        mouse_mode: ResMut<MouseToolState>,
    );
    fn add_mode_kb(available_keybinds: ResMut<AvailableKeybinds>);

    fn remove_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
        available_keybinds.clear();
    }
}

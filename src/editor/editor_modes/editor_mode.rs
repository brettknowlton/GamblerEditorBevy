use bevy::{prelude::*, reflect::GetTypeRegistration};

use crate::{
    editor_modes::{significant_component::SignificantComponent, EditorObject},
    message_display::MessageDisplay,
    mouse_state::MouseState,
    ui, AvailableKeybinds, Crosshair, CustomInput, Editor, EditorObjectKind, EditorState,
    SelectedTileID, TextureHandles, TileID, UpdatePlaceholderMessage,
};

pub trait EditorModePlugin: Plugin {
    fn mode() -> EditorState;

    fn modify_app(app: &mut App) -> &mut App {
        app
    }

    fn build_plugin<T: SignificantComponent + GetTypeRegistration>(app: &mut App) {
        let modified_app = Self::modify_app(app);

        modified_app
            .register_type::<T>()
            //startup systems (may need to be moved from here to maintain order)
            .add_systems(Startup, Self::init)
            //OnEnter systems
            .add_systems(
                OnEnter(Self::mode()),
                (
                    Self::enter_mode,
                    Self::add_mode_kb,
                    crate::ui::update_placeholder::<T>,
                )
                    .chain(),
            )
            //Update systems, that run only while TileEditor is active
            .add_systems(
                Update,
                (
                    Self::mode_kb_core::<T>,
                    ui::update_placeholder::<T>.run_if(on_message::<UpdatePlaceholderMessage>),
                    (Self::mode_click::<T>).run_if(Editor::editor_is_dragging),
                )
                    .chain()
                    .run_if(in_state(Self::mode())),
            )
            //OnExit systems
            .add_systems(
                OnExit(Self::mode()),
                (Self::exit_mode, Self::remove_mode_kb).chain(),
            );
    }

    fn init(
        _spritesheets: ResMut<TextureHandles>,
        _bottom_bar: ResMut<MessageDisplay>,
        _asset_server: Res<AssetServer>,
    ) {
        return;
    }

    fn enter_mode(
        _tooling_menu: ResMut<ui::ToolingMenuState>,
        _selected_item_id: Res<crate::SelectedTileID>,
    ) {
        return;
    }

    fn exit_mode(bottom_bar: ResMut<MessageDisplay>);

    fn mode_keybinds<T: Component + SignificantComponent>(
        _commands: Commands,
        _input: Res<ButtonInput<KeyCode>>,

        _crosshair: Single<(&Transform, &Crosshair)>,
        _items: Query<(Entity, &EditorObject), With<T>>,
        _selected_tile_id: ResMut<SelectedTileID>,

        _bottom_bar: ResMut<MessageDisplay>,
        _next_editor_state: ResMut<NextState<EditorState>>,
    ) {
        return;
    }

    fn mode_kb_core<T: Component + SignificantComponent>(
        commands: Commands,

        input: Res<ButtonInput<KeyCode>>,
        mut bottom_bar: ResMut<MessageDisplay>,
        mut next_editor_state: ResMut<NextState<EditorState>>,

        crosshair: Single<(&Transform, &Crosshair)>,
        items: Query<(Entity, &EditorObject), With<T>>,
        selected_tile_id: ResMut<SelectedTileID>,
    ) {
        Self::editor_mode_switch_keybinds::<T>(&input, &mut bottom_bar, &mut next_editor_state);
        Self::mode_keybinds::<T>(
            commands,
            input,
            crosshair,
            items,
            selected_tile_id,
            bottom_bar,
            next_editor_state,
        );
    }
    fn editor_mode_switch_keybinds<T: Component + SignificantComponent>(
        input: &Res<ButtonInput<KeyCode>>,

        bottom_bar: &mut ResMut<MessageDisplay>,
        next_editor_state: &mut ResMut<NextState<EditorState>>,
    ) {
        if input.just_pressed(KeyCode::KeyQ) {
            bottom_bar.send_mode_enter_message("Normal Mode");
            next_editor_state.set(EditorState::Normal);
        }

        if input.just_pressed(KeyCode::Digit1) || input.just_pressed(KeyCode::Numpad1) {
            bottom_bar.send_mode_enter_message("Tile Mode");
            next_editor_state.set(EditorState::Editing(EditorObjectKind::Tile(TileID::Any)));
        }

        if input.just_pressed(KeyCode::Digit2) || input.just_pressed(KeyCode::Numpad2) {
            bottom_bar.send_mode_enter_message("Collider Mode");
            next_editor_state.set(EditorState::Editing(EditorObjectKind::Collider));
        }

        if input.just_pressed(KeyCode::Digit3) || input.just_pressed(KeyCode::Numpad3) {
            bottom_bar.send_mode_enter_message("Actor Mode");
            next_editor_state.set(EditorState::Editing(EditorObjectKind::Actor));
        }
    }

    fn mode_click<T: SignificantComponent>(
        _commands: Commands,
        _window: Single<&Window, With<bevy::window::PrimaryWindow>>,
        _camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
        _items: Query<(Entity, &super::EditorObject), With<T>>,
        _selected_tile_id: ResMut<crate::SelectedTileID>,
        _mouse_state: Res<MouseState>,
        _bottom_bar: ResMut<crate::message_display::MessageDisplay>,
    ) {
        return;
    }

    fn get_mode_kb() -> Vec<(CustomInput, String)>;

    fn add_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
        for (input, description) in Self::get_mode_kb() {
            available_keybinds.add_keycode(input, description);
        }
    }
    fn remove_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
        available_keybinds.clear();
    }
}

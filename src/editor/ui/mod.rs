use bevy::prelude::*;

mod menu;
use bevy_egui::EguiPrimaryContextPass;
pub use menu::*;

#[macro_use]
pub mod message_display;

mod panels;
pub use panels::*;

mod custom_input;
pub use custom_input::*;

mod placeholder;
pub use placeholder::*;

pub mod grid;

use crate::EditorState;

pub fn editor_ui_plugin(app: &mut App) {
    app.init_resource::<ToolingMenuState>()
        .init_resource::<LeftPanelEdge>()
        .init_resource::<MouseToolState>()
        .init_resource::<AvailableKeybinds>()
        .init_resource::<panels::BottomPanel>()
        .init_resource::<LeftPanel>()
        .init_resource::<panels::ModeTabsPanel>()
        .init_resource::<panels::RightToolsPanel>()
        .add_message::<UpdatePlaceholderMessage>()
        .add_plugins(message_display::BottomBarPlugin)
        .add_systems(OnEnter(EditorState::Normal), (add_normal_mode_kb,).chain())
        .add_systems(OnExit(EditorState::Normal), (remove_normal_mode_kb,))
        .add_systems(
            Update,
            grid::draw_grid.run_if(in_state(grid::ShowGrid::Yes)),
        )
        .add_systems(Update, placeholder::trigger_placeholder_update)
        .add_systems(Update, menu::sync_tooling_menu_visibility)
        .add_systems(EguiPrimaryContextPass, menu::render_egui_panels);
}

fn add_normal_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.add_keycode(
        CustomInput::Combo(vec![KeyCode::ControlLeft, KeyCode::KeyS]),
        "Save Scene".into(),
    );
    available_keybinds.add_keycode(
        CustomInput::Combo(vec![KeyCode::ControlLeft, KeyCode::KeyT]),
        "Test Scene".into(),
    );
    available_keybinds.add_keycode(CustomInput::Single(KeyCode::KeyQ), "Quit Edit Mode".into());
}

fn remove_normal_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.clear();
}

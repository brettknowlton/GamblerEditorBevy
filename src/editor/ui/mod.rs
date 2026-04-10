use std::path::PathBuf;

use bevy::prelude::*;

mod menu;
use bevy_egui::EguiPrimaryContextPass;
pub use menu::*;

#[macro_use]
pub mod bottom_bar;

mod custom_input;
pub use custom_input::*;

mod placeholder;
pub use placeholder::*;

use crate::{EditorState, ShowGrid, TILE_SCALE, TILE_SIZE, ZONE_SIZE};

pub fn editor_ui_plugin(app: &mut App) {
    app.init_resource::<ToolingMenuState>()

        .init_resource::<AvailableKeybinds>()
        .init_resource::<menu::KBIcon>()
        .add_message::<UpdatePlaceholderMessage>()

        .add_plugins(bottom_bar::bottom_bar_plugin)

        .add_systems(OnEnter(EditorState::Normal), (add_normal_mode_kb,).chain())
        .add_systems(OnExit(EditorState::Normal), (remove_normal_mode_kb,))
        .add_systems(Update, draw_grid.run_if(in_state(ShowGrid::Yes)))
        .add_systems(Update, placeholder::trigger_placeholder_update)
        .add_systems(Update, menu::sync_tooling_menu_visibility)
        .add_systems(
            EguiPrimaryContextPass,
            (menu::egui_panel_render, menu::bottom_bar_ui).chain(),
        );
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
    available_keybinds.add_keycode(
        CustomInput::Single(KeyCode::KeyQ),
        "Quit Edit Mode".into(),
    );
}
fn remove_normal_mode_kb(mut available_keybinds: ResMut<AvailableKeybinds>) {
    available_keybinds.clear();
}

fn draw_grid(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Isometry2d::new(Vec2::new(0.0, 0.0), Rot2::degrees(0.0)),
            UVec2::new(100, 100),
            Vec2::new(
                (TILE_SIZE * TILE_SCALE) as f32,
                (TILE_SIZE * TILE_SCALE) as f32,
            ),
            Color::srgba(0.0, 1.0, 0.0, 0.5),
        )
        .outer_edges();

    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::new(10, 10),
            Vec2::new(
                (TILE_SIZE * TILE_SCALE * ZONE_SIZE) as f32,
                (TILE_SIZE * TILE_SCALE * ZONE_SIZE) as f32,
            ),
            Color::srgba(1.0, 0.0, 0.0, 0.5),
        )
        .outer_edges();
}

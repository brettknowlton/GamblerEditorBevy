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

use crate::mouse_state::MouseState;
use crate::SelectedTileID;

pub mod grid;

/// value to control Z-layering of UI items. May not be necessary anymore due to EGUI
pub const UI_Z_LAYER: f32 = 10.0; //z layer for UI elements, we can use this to make to relatively place UI elements correctly

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        Self::build_plugin(app);
    }
}

impl UIPlugin {
    pub fn build_plugin(app: &mut App) {
        app.init_resource::<ToolingMenuState>()
            .init_resource::<LeftPanelEdge>()
            .init_resource::<MouseState>()
            .init_resource::<SelectedTileID>()
            .init_resource::<AvailableKeybinds>()
            .init_resource::<panels::BottomPanel>()
            .init_resource::<LeftPanel>()
            .init_resource::<panels::ModeTabsPanel>()
            .init_resource::<panels::RightToolsPanel>()
            .add_message::<UpdatePlaceholderMessage>()
            .add_plugins(message_display::BottomBarPlugin)
            .add_systems(
                Update,
                grid::draw_grid.run_if(in_state(grid::ShowGrid::Yes)),
            )
            .add_systems(Update, placeholder::trigger_placeholder_update)
            .add_systems(Update, menu::sync_tooling_menu_visibility)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    menu::render_bottom_panel,
                    menu::render_mode_tabs_panel,
                    menu::render_left_panel,
                    menu::render_right_panel,
                    menu::sync_tile_selection,
                )
                    .chain(),
            );
    }
}

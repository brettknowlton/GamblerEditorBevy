use bevy::{prelude::*, window::WindowResolution};
use bevy_egui::EguiPlugin;
use bevy_rapier2d::prelude::*;

pub mod consts;
pub mod utilities;
pub mod editor;
pub mod game;

use bevy_rapier2d::{plugin::{NoUserData, RapierPhysicsPlugin}, render::RapierDebugRenderPlugin};
pub use consts::*;
pub use utilities::*;
pub use editor::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: WINDOW_TITLE2.to_string(),
                        resolution: WindowResolution::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32),
                        resizable: false,
                        decorations: true,
                        visible: true,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )

        .add_plugins(
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0)
        )
        .add_plugins(RapierDebugRenderPlugin::default())
        
        .add_plugins(EguiPlugin::default())
        

        
        .insert_resource(ClearColor(Color::from(WINDOW_DEFAULT_BACKGROUND_COLOR)))
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        
        .add_plugins(editor::editor_plugin)
        .add_plugins(game::game_plugin)

        .run();
}


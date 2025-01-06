use std::path::Path;

use bevy::math::Vec2;
use bevy::prelude::*;
use consts::{TEXTURES_PATH, WINDOW_HEIGHT, WINDOW_WIDTH};

pub(crate) mod consts;
pub(crate) mod utilities;
mod editor;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Harken".into(),
                        resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                        resizable: false,
                        decorations: true,
                        visible: true,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_plugins(editor::editor_plugin)
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(Color::srgb(0.2, 0.05, 0.1)))
        .run();
}


//bundles are a collection of components that are commonly used together
//OrthographicCameraBundle is a bundle that contains the following components:
//Transform, GlobalTransform, OrthographicCamera, Visible, and MainCamera
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d { ..default() });
}

fn spawn_players(mut commands: Commands, asset_server: Res<AssetServer>) {
    let textures_path = Path::new(&format!("{TEXTURES_PATH}/player.png")).to_path_buf();

    let tex1 = asset_server.load(textures_path);

    commands.spawn(Sprite {
        custom_size: Some(Vec2::new(100.0, 100.0)),
        image: tex1,
        ..default()
    });
}
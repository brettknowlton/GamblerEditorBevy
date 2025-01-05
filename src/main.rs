use std::path::Path;

use bevy::math::Vec2;
use bevy::prelude::*;
use consts::{SCREEN_HEIGHT, SCREEN_WIDTH, TEXTURES_PATH, WINDOW_DEFAULT_HEIGHT};

mod consts;
mod tiles;
mod interactables;
mod editor_ui;
mod actors;
mod utilities;


#[derive(Default)]
pub enum EditorState {
    #[default]
    NormalMode,
    TileMode,
    ActorMode,
    InteractableMode,
}


fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Harken".into(),
                        resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                        resizable: false,
                        decorations: true,
                        visible: false,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )
        .add_systems(Startup, (hello_world, setup))
        .run();
}

//bundles are a collection of components that are commonly used together
//OrthographicCameraBundle is a bundle that contains the following components:
//Transform, GlobalTransform, OrthographicCamera, Visible, and MainCamera
fn setup(mut commands: Commands) {
    
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgba(0.2, 0.05, 0.1, 1.0)),
            ..default()
        },
        ..default()
    });
}

#[derive(Component)]
pub struct Player {}

fn spawn_players(mut commands: Commands, asset_server: Res<AssetServer>) {
    let textures_path = Path::new(&format!("{TEXTURES_PATH}/player.png")).to_path_buf();
    
    
    let tex1 = asset_server.load(textures_path);

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        transform: Transform::from_xyz(
            consts::WINDOW_DEFAULT_WIDTH as f32 / 2.0,
            WINDOW_DEFAULT_HEIGHT as f32 / 2.0,
            0.0,
        ),
        texture: tex1,
        ..default()
    });
}

fn hello_world() {
    println!("Hello, world!");
}



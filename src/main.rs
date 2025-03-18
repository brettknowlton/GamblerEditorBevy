use bevy::prelude::*;

pub mod consts;
pub mod utilities;
pub mod editor;
pub mod game;

pub use consts::*;
pub use utilities::*;
pub use editor::*;
pub use game::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: WINDOW_TITLE2.to_string(),
                        resolution: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT).into(),
                        resizable: false,
                        decorations: true,
                        visible: true,
                        ..default()
                    }),
                    ..default()
                })
                .build(),
        )

        .insert_resource(ClearColor(Color::from(WINDOW_DEFAULT_BACKGROUND_COLOR)))
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .add_plugins(editor::editor_plugin)
        .add_plugins(game::game_plugin)

        .run();
}

//test line for commit
//bundles are a collection of components that are commonly used together
//OrthographicCameraBundle is a bundle that contains the following components:
//Transform, GlobalTransform, OrthographicCamera, Visible, and MainCamera

// fn spawn_players(mut commands: Commands, asset_server: Res<AssetServer>) {
//     let textures_path = Path::new(&format!("{TEXTURES_PATH}/player.png")).to_path_buf();

//     let tex1 = asset_server.load(textures_path);

//     commands.spawn(Sprite {
//         custom_size: Some(Vec2::new(100.0, 100.0)),
//         image: tex1,
//         ..default()
//     });
// }
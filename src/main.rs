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

        //Rapier physics plugins
        // .insert_resource(RapierConfiguration {
        //     gravity: Vec2::new(0.0, -9.81), 
        //     physics_pipeline_active: true,
        //     query_pipeline_active: true,
        //     ..Default::default()
        // })

        // .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(32.0))
        // .add_plugins(RapierDebugRenderPlugin::default())


        .insert_resource(ClearColor(Color::from(WINDOW_DEFAULT_BACKGROUND_COLOR)))
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        
        .add_plugins(editor::editor_plugin)
        .add_plugins(game::game_plugin)

        .run();
}


mod tile;
mod scene;
pub use tile::*;
pub use crate::consts::*;
pub use crate::utilities::*;
pub use std::{ fmt::Debug, path::PathBuf };

use bevy::{
    prelude::*,
    sprite::Anchor,
    // utils::HashMap,
};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum EditorState {
    #[default]
    Inactive,
    Normal,
    Saving,
    Tile,
    Interactable,
    Actor,
    Trigger,
}

pub fn editor_plugin(app: &mut App) {
    app.init_state::<EditorState>()
        .add_systems(Startup, (initialize, create_crosshair))
        .add_plugins(tile::tilemode_plugin)
        .add_systems(Update, move_camera)
        .add_systems(Update, keybinds);
    //.insert_resource(Scene(Scene::new()))
}

fn initialize(mut commands: Commands) {
    println!("Entering Gambler Editor");

    //create scene manager component that will read/write our scene data between the enviornment and a json file
    println!(
        "Prompt to go here eventually to ask if the user would like to load a specific file, for now we will always just load from DEFAULT_SCENE_PATH"
    );
    commands.spawn(scene::Scene::new(Some(PathBuf::from(DEFAULT_SCENE_PATH))));

    //create grid
    //a texture slightly larger than the window size? just keeps getting snapped to the nearest grid point... seems like it would work
}

fn create_crosshair(mut commands: Commands, asset_server: Res<AssetServer>) {
    //create crosshair
    let tex_path = PathBuf::from("textures/crosshairs/crosshair1.png");
    let tex1 = asset_server.load(tex_path);

    //spawn crosshair
    commands.spawn((
        Crosshair {
            location: Coordinate((WINDOW_WIDTH as i64) / 2, (WINDOW_HEIGHT as i64) / 2),
            vel_x: 0.0,
            vel_y: 0.0,
        },
        Sprite {
            image: tex1,
            anchor: Anchor::Center,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 5.0),
            ..default()
        },
    ));
}

fn move_camera(
    mut camera: Query<&mut Transform, (With<Camera>, Without<Crosshair>)>,
    crosshairs: Query<(&Transform, &Crosshair, &Sprite)>
) {
    //move camera to be centered on crosshair
    //crosshair's sprite is anchored to the bottom left
    for crosshair_transform in &mut crosshairs.iter() {
        let mut camera_transform = camera.single_mut();
        camera_transform.translation = crosshair_transform.0.translation;
    }
}

fn keybinds(
    mut next_state: ResMut<NextState<EditorState>>,
    state: ResMut<State<EditorState>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    // m_input: Res<ButtonInput<MouseButton>>,

    mut crosshairs: Query<(&mut Transform, &mut Crosshair, &mut Sprite)>,

    mut scenes: Query<&mut scene::Scene>
) {
    let scene = scenes.single();

    //manage the editor state, uses top row starting with "E" key to control which UI mode we will be in
    if input.just_pressed(KeyCode::KeyT) {
        next_state.set(EditorState::Tile);
    }
    if input.just_pressed(KeyCode::KeyE) {
        next_state.set(EditorState::Normal);
    }

    //Controlling the "Q" key
    if input.just_pressed(KeyCode::KeyQ) {
        if state.get() == &EditorState::Normal {
            //pressing q will enter "saving" mode if we are already in normal mode:
            next_state.set(EditorState::Saving);
            println!("Would you like to save the current scene?");
        } else if state.get() == &EditorState::Saving {
            //pressing q will cancel the save if we are already in saving mode:
            next_state.set(EditorState::Normal);
            println!("Save cancelled");
        } else {
            next_state.set(EditorState::Normal);
        }
    }
    //controlling the "E" key
    if input.just_pressed(KeyCode::KeyE) {
        //if we are in save mode, pressing "E" will save the current scene
        if state.get() == &EditorState::Saving {
            println!("Attempting to save scene");
            scene.write_serialized_scene(DEFAULT_SCENE_PATH.to_string());
            next_state.set(EditorState::Normal);
            println!("Scene saved, returning to Normal Mode");
        }
    }

    //WASD controls for crosshair movement
    for (mut transform, mut crosshair, mut sprite) in &mut crosshairs {
        if input.pressed(KeyCode::KeyW) && !input.pressed(KeyCode::KeyS) {
            crosshair.vel_y = 120.0;
        }
        if input.pressed(KeyCode::KeyS) && !input.pressed(KeyCode::KeyW) {
            crosshair.vel_y = -120.0;
        }
        if input.pressed(KeyCode::KeyD) && !input.pressed(KeyCode::KeyA) {
            crosshair.vel_x = 150.0;
            sprite.flip_x = false;
        }
        if input.pressed(KeyCode::KeyA) && !input.pressed(KeyCode::KeyD) {
            crosshair.vel_x = -150.0;
            sprite.flip_x = true;
        }

        transform.translation.y += crosshair.vel_y * time.delta_secs();
        transform.translation.x += crosshair.vel_x * time.delta_secs();

        //apply friction
        crosshair.vel_y = crosshair.vel_y * (0.99 as i32 as f32);
        crosshair.vel_x = crosshair.vel_x * (0.99 as i32 as f32);
        crosshair.location = Coordinate(
            transform.translation.x as i64,
            transform.translation.y as i64
        );
    }
}

#[derive(Component)]
struct Crosshair {
    location: Coordinate,
    vel_x: f32,
    vel_y: f32,
}

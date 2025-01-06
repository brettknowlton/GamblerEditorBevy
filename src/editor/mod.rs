use std::{collections::HashMap, env, fmt::Debug, path::PathBuf};
use serde::Deserialize;
use serde_json::*;

use bevy::{
    gizmos::cross,
    input::mouse::MouseButtonInput,
    prelude::*,
    render::camera,
    sprite::Anchor,
    state::state,
    transform,
    // utils::HashMap,
};
use resources::CurrentEditorObject;
use crate::{ consts::{ WINDOW_HEIGHT, WINDOW_WIDTH }, utilities::* };
pub(crate) mod tile;
use tile::*;

use super::utilities::*;

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum EditorMode {
    #[default]
    Inactive,
    Normal,
    Tile,
    Interactable,
    Actor,
    Trigger,
}

pub trait EditorObject: Sync + Send {
    //field for goid
    ///If implemented, the object will be able to return a GOID based on its position and style;
    ///
    ///
    fn get_goid(&self) -> String {
        "OXXYYST".to_string()
    }
    fn get_coordinate(&self) -> Coordinate {
        Coordinate(0, 0)
    }

    /*
    GOID is a generic way to uniquely identify game objects, broken down as such: 
    O: Object Type (ie. tile, actor, trigger, etc.), X: Object starting X position, Y: Object starting Y position, ST: Object style/state upon creation
    */
}

#[derive(Component)]
struct Crosshair {
    location: Coordinate,
    vel_x: f32,
    vel_y: f32,
}

pub fn editor_plugin(app: &mut App) {
    app.init_state::<EditorMode>()
        .add_systems(Startup, initialize)
        .add_plugins(tile::tilemode_plugin)
        .add_systems(Update, move_camera)
        .add_systems(Update, keybinds);
    //.insert_resource(Scene(Scene::new()))
}

fn initialize(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("Entering Gambler Editor");
    //create crosshair
    let tex_path = PathBuf::from("textures/crosshairs/crosshair1.png");
    let tex1 = asset_server.load(tex_path);

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
            scale: Vec3::new(5.0, 5.0, 5.0),
            ..default()
        },
    ));
    //create grid
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
    mut commands: Commands,
    mut next_state: ResMut<NextState<EditorMode>>,
    mut state: ResMut<State<EditorMode>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    m_input: Res<ButtonInput<MouseButton>>,

    mut crosshairs: Query<(&mut Transform, &mut Crosshair, &mut Sprite)>
) {
    let crosshair = crosshairs.iter_mut().next().unwrap().1;

    if input.just_pressed(KeyCode::KeyT) {
        next_state.set(EditorMode::Tile);
    }
    if input.just_pressed(KeyCode::Escape) {
        next_state.set(EditorMode::Normal);
    }
    if input.just_pressed(KeyCode::KeyQ) {
        //if state is anything but normal, return to normal
        if state.get() != &EditorMode::Normal {
            next_state.set(EditorMode::Normal);
        } else {
            //if state is normal, exit editor
            println!("Would you like to save the current scene?");

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
    }
}

#[derive(Component)]
pub struct Scene {
    data: HashMap<String, Box<dyn EditorObject>>,
}


impl Scene {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn push(&mut self, object: Box<dyn EditorObject>) {
        self.data.insert(object.get_goid(), object);
    }

    pub fn remove(&mut self, location: Coordinate) {
        // self.data.retain(|x| { x.clone().split_off(3) != format!("{}{}", location.0, location.1) });
    }

    pub fn get_data(&mut self) -> Option<Box<dyn EditorObject>> {
        let read_dir = std::fs::read(format!("{:?}/test.json", env::current_dir()));
        match read_dir {
            Ok(file_bytes) => {
                let to_string = String::from_utf8(file_bytes).unwrap_or_default();
                let contents: std::result::Result<Box<dyn EditorObject>, Error> = serde_json::from_str(&to_string);
                if let Ok(file_contents) = contents {
                    // println!("We got some file contents: {file_contents:?}");
                    // let conversion = 
                }
            },
            Err(e) => println!("Error reading file contents: {e:?}"),
        }
        
        None
    }
}



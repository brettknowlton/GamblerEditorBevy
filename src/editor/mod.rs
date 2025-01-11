mod tile;
mod scene;

use bevy::scene::*;
pub use tile::*;
pub use crate::consts::*;
pub use crate::utilities::*;
pub use std::{ fmt::Debug, path::PathBuf };

use bevy::{ prelude::*, sprite::Anchor };

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum EditorState {
    #[default]
    Inactive,
    Normal,
    SaveAsk,
    Saving,
    LoadAsk,
    Loading,
    Tile,
    Interactable,
    Actor,
    Trigger,
}

pub fn editor_plugin(app: &mut App) {
    app.init_state::<EditorState>()
        .add_plugins(ScenePlugin)
        .register_type::<EditorObject>()

        .add_systems(Startup, (initialize, create_crosshair))
        .add_plugins((tile::tilemode_plugin, scene::scene_plugin))
        // .add_systems(Update, move_camera)
        .add_systems(Update, keybinds);
    //placeholder resource for whatever tile we are trying to place
}

fn initialize(mut commands: Commands, mut next_state: ResMut<NextState<EditorState>>) {
    println!("Entering Gambler Editor");

    //create camera and add a UIItem component to it
    commands.spawn((Camera2d::default(), UIItem::default()));

    //create scene manager component that will read/write our scene data between the enviornment and a json file
    println!("Would you like to load a scene? Y/N");
    next_state.set(EditorState::LoadAsk);
}

fn create_crosshair(mut commands: Commands, asset_server: Res<AssetServer>) {
    //create crosshair
    let tex_path = PathBuf::from("textures/crosshairs/crosshair1.png");
    let tex1 = asset_server.load(tex_path);

    //spawn crosshair
    commands.spawn((
        Crosshair {},
        Sprite {
            image: tex1,
            anchor: Anchor::Center,
            ..default()
        },
        Transform {
            scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 5.0),
            ..default()
        },
    ));
}

fn keybinds(
    mut next_state: ResMut<NextState<EditorState>>,
    state: ResMut<State<EditorState>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    // m_input: Res<ButtonInput<MouseButton>>,

    mut uiitems: Query<(&mut UIItem, &mut Transform), Without<Camera2d>>,
    mut cameras: Query<(&mut UIItem, &mut Transform, &Camera2d)>,
) {
    //manage the editor state, you can switch between modes with ERTY except if you are attempting to save the document
    //"E" enters editor mode and aborts any saving operation
    if state.get() != &EditorState::LoadAsk {
        if input.just_pressed(KeyCode::KeyE) {
            if state.get() == &EditorState::SaveAsk {
                println!("Saving aborted.");
            }
            next_state.set(EditorState::Normal);
        }

        //"R" switches to interactable mode and aborts any saving operation
        if input.just_pressed(KeyCode::KeyR) {
            if state.get() != &EditorState::SaveAsk {
                println!("Saving aborted.");
            }
            next_state.set(EditorState::Interactable);
        }

        //"T" switches to tile mode and aborts any saving operation
        if input.just_pressed(KeyCode::KeyT) {
            if state.get() != &EditorState::SaveAsk {
                println!("Saving aborted.");
            }
            next_state.set(EditorState::Tile);
        }

        //"Y" switches to actor mode and aborts any saving operation
        if input.just_pressed(KeyCode::KeyY) {
            if state.get() != &EditorState::SaveAsk {
                println!("Saving aborted.");
            }
            next_state.set(EditorState::Actor);
        }
    } else{
        if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
            next_state.set(EditorState::Loading);
            println!("Attempting to load scene");
        }
        if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
            next_state.set(EditorState::Normal);
            println!("Returning to Normal Mode");
        }
    }

    // "Q" will prompt the user to save the scene if they are in normal mode,
    // This action will put the editor into "saving" mode- pressing "Q" again will save the scene
    // otherwise "Q"" will return to normal mode if in any mode other than normal or saving
    if input.just_pressed(KeyCode::KeyQ) {
        if state.get() == &EditorState::Normal {
            //pressing q will enter "saving" mode if we are already in normal mode:
            next_state.set(EditorState::SaveAsk);
            println!("Would you like to save the current scene?");
        } else if state.get() == &EditorState::SaveAsk {
            println!("Attempting to save scene");
            next_state.set(EditorState::Saving);
            return;
        } else {
            next_state.set(EditorState::Normal);
            println!("Returning to Normal Mode");
        }
    }

    //"E" (and escape) is used for switching back to editor mode from any other mode
    if input.just_pressed(KeyCode::KeyE) || input.just_pressed(KeyCode::Escape) {
        next_state.set(EditorState::Normal);
    }

    let mut vel_y = 0.0;
    let mut vel_x = 0.0;

    let list = &mut uiitems.iter_mut();
    //WASD controls for crosshair movement
    if input.pressed(KeyCode::KeyW) && !input.pressed(KeyCode::KeyS) {
        vel_y = 120.0;
    }
    if input.pressed(KeyCode::KeyS) && !input.pressed(KeyCode::KeyW) {
        vel_y = -120.0;
    }
    if input.pressed(KeyCode::KeyD) && !input.pressed(KeyCode::KeyA) {
        vel_x = 150.0;
    }
    if input.pressed(KeyCode::KeyA) && !input.pressed(KeyCode::KeyD) {
        vel_x = -150.0;
    }

    //update anything with a UIItem component
    for (mut ui, mut t) in list {
        ui.vel_x = vel_x;
        ui.vel_y = vel_y;
        t.translation.x += ui.vel_x * time.delta_secs();
        t.translation.y += ui.vel_y * time.delta_secs();
        //apply frictoin -1% of the velocity per frame
        ui.vel_x *= 0.99;
        ui.vel_y *= 0.99;
    }
    //update the camera in the same way
    for (mut ui, mut t, _) in cameras.iter_mut() {
        ui.vel_x = vel_x;
        ui.vel_y = vel_y;
        t.translation.x += ui.vel_x * time.delta_secs();
        t.translation.y += ui.vel_y * time.delta_secs();
        //apply frictoin -1% of the velocity per frame
        ui.vel_x *= 0.99;
        ui.vel_y *= 0.99;
    }
}

#[derive(Component)]
#[require(UIItem)]
struct Crosshair {}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Transform)]
struct UIItem {
    vel_x: f32,
    vel_y: f32,
}

#[derive(Component, Reflect,)]
#[reflect(Component)]
/// A component that marks an entity as a placeholder object, these are preview objects that are not yet placed into the scene.
struct PlaceholderObject;

/// A component that marks an entity as a savable editor item.
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct EditorObject {
    major_type: char, //'T' for tile, 'E' for entity, 'P' for player, etc.
    internal_type: u64, //ultimatley an index into which style of tile or entity we are using within the major type
    coordinate: TCoordinate,
}

impl EditorObject {
    fn new(mt: char, it: u64, coord: Coordinate) -> Self {
        Self {
            major_type: mt,
            internal_type: it,
            coordinate: TCoordinate::new(mt, coord),
        }
    }

    fn get_object_type(&self) -> char {
        self.major_type
    }
    fn get_internal_type(&self) -> u64 {
        self.internal_type
    }
    fn get_coordinate(&self) -> TCoordinate {
        self.coordinate.clone()
    }

    fn set_major_type(&mut self, v: char) {
        self.major_type = v;
    }
    fn set_internal_type(&mut self, v: u64) {
        self.internal_type = v;
    }
    fn set_coordinate(&mut self, coord: Coordinate) {
        self.coordinate = TCoordinate::new(self.get_object_type(), coord);
    }
}



#[macro_use]
pub mod ui;
pub use tile::*;
pub use crate::consts::*;
pub use crate::utilities::*;

mod tile;
mod scene;
use resources::*;
pub use std::{ fmt::Debug, path::PathBuf };


use bevy::{ prelude::*, sprite::Anchor };

//EditorState is an enum that defines the different states the editor can be in, this is used to determine what the editor is currently doing
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum EditorState {
    #[default]
    Inactive,
    Normal,
    LoadAsk,
    Loading,
    LoadEmpty,
    SaveAsk,
    Saving,
    Editing(EditingMode),
}

///The EditingMode enum is used to determine what type of object the user is currently trying to place in the scene, this is kind of like an "Internal Type" for the editor state. We will almost always be in EditorMode::Editing but its more granular than that
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum EditingMode {
    #[default]
    None,
    Interactable,
    Tile,
    Actor,
} 

pub fn editor_plugin(app: &mut App) {
    app.init_state::<EditorState>()
        .register_type::<EditorObject>()
        .add_event::<BottomBarUpdate>()

        .init_resource::<EditorBottomBarDisplayed>()
        .init_resource::<EditorBottomBarMessage>()
        .init_resource::<EditorBottomBarQueuedMessages>()

        //The only true startup systems here:
        .add_systems(Startup, (initialize, create_crosshair, ui::spawn_general_editor_ui).chain())

        //begin update system to update the bottom bar text
        .add_systems(Update, ui::send_messages)
        .add_plugins(tile::tilemode_plugin)
        .add_plugins(scene::scene_plugin)
        .add_systems(Update, stateful_keybinds.run_if(not(in_state(EditorState::Inactive))));
    //placeholder resource for whatever tile we are trying to place
}

fn initialize(mut commands: Commands, mut next_state: ResMut<NextState<EditorState>>, mut message_queue: ResMut<EditorBottomBarQueuedMessages>) {

    //create camera and add a UIItem component to it
    commands.spawn((Camera2d::default(), UIItem::default()));

    //set the state to ask about loading a scene
    next_state.set(EditorState::LoadAsk);

    //push a message to the bottom bar that asks the user if they would like to load a scene
    send_message!(Some('i'), message_queue, "Would you like to load a scene? Yenter/Noscape");
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
            scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 0.),
            ..default()
        },
    ));
}

//wow this is kind of dope if it work
#[derive(Event)]
struct BottomBarUpdate;

fn stateful_keybinds(
    mut next_state: ResMut<NextState<EditorState>>,
    state: ResMut<State<EditorState>>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    // m_input: Res<ButtonInput<MouseButton>>,

    mut uiitems: Query<(&mut UIItem, &mut Transform), Without<Camera2d>>,
    mut cameras: Query<(&mut UIItem, &mut Transform, &Camera2d)>
) {
    let messages = &mut message_queue;
    //manage the editor state, you can switch between modes with ERTY except if you are attempting to save the document
    //"E" enters editor mode and aborts any saving operation
    if state.get() != &EditorState::LoadAsk {
        if input.just_pressed(KeyCode::KeyE) || input.just_pressed(KeyCode::Digit1)  || input.just_pressed(KeyCode::Numpad1) {
            if state.get() == &EditorState::SaveAsk {
                send_message!(Some('w'), messages, "Saving aborted.");
            }
            send_message!(Some('i'), messages, "Returning to Normal Mode");
            next_state.set(EditorState::Normal);
        }

        //"T" switches to tile mode and aborts any saving operation
        if input.just_pressed(KeyCode::KeyT) || input.just_pressed(KeyCode::Digit2) || input.just_pressed(KeyCode::Numpad2) {
            if state.get() == &EditorState::SaveAsk {
                send_message!(Some('w'), messages, "Saving aborted.");
            }
            send_message!(Some('i'), messages, "Switching to Tile Mode");
            next_state.set(EditorState::Editing(EditingMode::Tile));
        }

        //"R" switches to interactable mode and aborts any saving operation
        if input.just_pressed(KeyCode::KeyR) || input.just_pressed(KeyCode::Digit3) || input.just_pressed(KeyCode::Numpad3) {
            if state.get() != &EditorState::SaveAsk {
                send_message!(Some('w'), messages, "Saving aborted.");
            }
            send_message!(Some('i'), messages, "Switching to Interactable Mode");
            next_state.set(EditorState::Editing(EditingMode::Interactable));
        }

        //"Y" switches to actor mode and aborts any saving operation
        if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Digit4) || input.just_pressed(KeyCode::Numpad4) {
            if state.get() != &EditorState::SaveAsk {
                send_message!(Some('w'), messages, "Saving aborted.");
            }
            send_message!(Some('i'), messages, "Switching to Actor Mode");
            next_state.set(EditorState::Editing(EditingMode::Actor));
        }
    } else {
        if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
            next_state.set(EditorState::Loading);
            send_message!(Some('i'), messages, "Attempting to load scene");
        }
        if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
            next_state.set(EditorState::LoadEmpty);
            send_message!(Some('w'), messages, "No scene loaded");
        }
    }

    if state.get() == &EditorState::SaveAsk {
        // "Y" will save the scene if the user is in saving mode, 'N" will abort the save
        if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
            next_state.set(EditorState::Saving);
            send_message!(Some('i'), messages, "Saving scene...");
        } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
            next_state.set(EditorState::Normal);
            send_message!(Some('w'), messages, "Saving aborted.");
        }
    }

    // "Q" will prompt the user to save the scene if they are in normal mode,
    // otherwise "Q"" will return to normal mode if in any mode other than normal
    if input.just_pressed(KeyCode::KeyQ) || input.all_pressed(vec!(KeyCode::ControlLeft, KeyCode::KeyS)) {
        if state.get() == &EditorState::Normal {
            //pressing q will enter "saving" mode if we are already in normal mode:
            next_state.set(EditorState::SaveAsk);
            send_message!(Some('i'), messages, "Would you like to save the scene? Yenter/Noscape");
        } else {
            next_state.set(EditorState::Normal);
            send_message!(Some('i'), messages, "Returning to Normal Mode");
        }
    }

    //"E" (and escape) is used for switching back to editor mode from any other mode
    if input.just_pressed(KeyCode::KeyE) || input.just_pressed(KeyCode::Escape) {
        next_state.set(EditorState::Normal);
        send_message!(Some('i'), messages, "Returning to Normal Mode");
    }

    let mut vel_y = 0.0;
    let mut vel_x = 0.0;

    let list = &mut uiitems.iter_mut();
    //WASD controls for crosshair movement
    if input.pressed(KeyCode::KeyW) && !input.pressed(KeyCode::KeyS) {
        vel_y = 200.0;
    }
    if input.pressed(KeyCode::KeyS) && !input.pressed(KeyCode::KeyW) {
        vel_y = -200.0;
    }
    if input.pressed(KeyCode::KeyD) && !input.pressed(KeyCode::KeyA) {
        vel_x = 200.0;
    }
    if input.pressed(KeyCode::KeyA) && !input.pressed(KeyCode::KeyD) {
        vel_x = -200.0;
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
pub struct Crosshair {}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Transform)]
struct UIItem {
    vel_x: f32,
    vel_y: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// A component that marks an entity as a placeholder object, these are preview objects that are not yet placed into the scene.
pub struct PlaceholderObjectTag;

/// A component that marks an entity as a savable editor item.
#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
pub struct EditorObject {
    internal_type: u64, //ultimatley an index into which style of tile or entity we are using within the major type
    coordinate: TCoordinate, //the coordinate of the object as well as the major type of the object
}

impl EditorObject {
    // fn new(mt: char, it: u64, coord: Coordinate) -> Self {
    //     Self {
    //         internal_type: it,
    //         coordinate: TCoordinate::new(mt, coord),
    //     }
    // }

    // fn get_object_type(&self) -> char {
    //     self.coordinate.object_type
    // }
    fn get_internal_type(&self) -> u64 {
        self.internal_type
    }
    fn get_coordinate(&self) -> TCoordinate {
        self.coordinate.clone()
    }

    // fn set_major_type(&mut self, v: char) {
    //     self.coordinate.object_type = v;
    // }
    // fn set_internal_type(&mut self, v: u64) {
    //     self.internal_type = v;
    // }
    // fn set_coordinate(&mut self, coord: Coordinate) {
    //     self.coordinate = TCoordinate::new(self.get_object_type(), coord);
    // }
}

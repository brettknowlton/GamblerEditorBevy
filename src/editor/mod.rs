#[macro_use]
pub mod ui;
pub use tile::*;
pub use crate::consts::*;
pub use crate::utilities::*;
use resources::*;

mod tile;
mod collider;
mod scene;
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
    LoadingEmpty,
    SaveAsk,
    Saving,
    QuitAsk,
    Editing(EditingComponent),
}

///The EditingMode enum is used to determine what type of object the user is currently trying to place in the scene, this is kind of like an "Internal Type" for the editor state. We will almost always be in EditorMode::Editing but its more granular than that
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum EditingComponent {
    #[default]
    None,
    Actor,
    Collider,
    Interactable,
    Tile,
}

#[derive(Clone, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum EditingMode {
    #[default]
    None,
    Selected(Vec<EditingModes>),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum EditingModes {
    GridSnap(bool), //whether or not we are snapping to the grid defined by TILE_SCALE
    CompositeTiles(bool), //whether or not we are placing colliders along with tiles
    CursorEnabled(bool), //whether or not the cursor is enabled
    CrosshairEnabled(bool), //whether or not the crosshair is enabled
}

pub fn editor_plugin(app: &mut App) {
    app
        //states
        .init_state::<EditorState>()
        .init_state::<EditingMode>()

        //registrations
        .register_type::<EditorObject>()
        .add_event::<BottomBarUpdate>()

        //resources
        .init_resource::<EditorBottomBarDisplayed>()
        .init_resource::<EditorBottomBarMessage>()
        .init_resource::<EditorBottomBarQueuedMessages>()
        .init_resource::<ActiveSelection>()

        //begin update system to update the bottom bar text
        .add_systems(Update, ui::send_messages)

        //plugins
        .add_plugins(tile::tilemode_plugin)
        .add_plugins(collider::collidermode_plugin)
        .add_plugins(scene::scene_plugin)

        //The only true startup systems here:
        .add_systems(Startup, (initialize, create_crosshair, ui::spawn_general_editor_ui).chain())
        .add_systems(Update, stateful_keybinds.run_if(not(in_state(EditorState::Inactive))));
    //placeholder resource for whatever tile we are trying to place
}

fn initialize(
    mut commands: Commands,
    mut next_state: ResMut<NextState<EditorState>>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>
) {
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
            scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 0.0),
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
    mut active_selection: ResMut<ActiveSelection>,
    // m_input: Res<ButtonInput<MouseButton>>,
    mut crosshairs: Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,

    mut uiitems: Query<(&mut UIItem, &mut Transform), (Without<Camera2d>, Without<Crosshair>)>,
    mut cameras: Query<(&mut UIItem, &mut Transform, &mut Camera2d)>
) {
    let messages = &mut message_queue;
    //manage the editor state, you can switch between modes with their letter call or number keys except if you are attempting to save/load the document

    //Universal keys, not dependant on state:

    // Q brings us back to normal mode
    if input.just_pressed(KeyCode::KeyQ) {
        next_state.set(EditorState::Normal);
        send_message!(Some('i'), messages, "Returning to Normal Mode");
    } else if
        //CTRL + S will enter saveAsk mode
        input.all_pressed(vec![KeyCode::KeyS, KeyCode::ControlLeft]) &&
        state.get() != &EditorState::SaveAsk
    {
        next_state.set(EditorState::SaveAsk);
        send_message!(Some('i'), messages, "Would you like to save the scene? Yenter/Noscape");
    } else if
        //CTRL + L will enter loadAsk mode
        input.all_pressed(vec![KeyCode::KeyL, KeyCode::ControlLeft]) &&
        state.get() != &EditorState::LoadAsk
    {
        next_state.set(EditorState::LoadAsk);
        send_message!(Some('i'), messages, "Would you like to load a scene? Yenter/Noscape");
    } else if
        //CTRL + Q will enter QuitAsk mode
        input.all_pressed(vec![KeyCode::KeyQ, KeyCode::ControlLeft]) &&
        state.get() != &EditorState::QuitAsk
    {
        next_state.set(EditorState::QuitAsk);
        send_message!(Some('i'), messages, "Would you like to exit the editor? Yenter/Noscape");
    }

    //O is the main button to directly use the Rectangle Tool
    if input.just_pressed(KeyCode::KeyO) {
        //use crosshair's coordinate as start
        let (_, t, _) = crosshairs.single();
        let coord = Coordinate { 0: t.translation.x as i64, 1: t.translation.y as i64 };

        active_selection.selection_rect = Some(SelectionRect::start(coord));

        send_message!(Some('i'), messages, "Rectangling.....");
    } else if input.just_released(KeyCode::KeyO) {
        //use crosshair's coordinate as end
        let (_, t, _) = crosshairs.single();
        let coord = Coordinate { 0: t.translation.x as i64, 1: t.translation.y as i64 };

        active_selection.selection_rect = active_selection.selection_rect.clone().map(|mut r| {
            r.end(coord);
            r
        });

        send_message!(
            Some('i'),
            messages,
            format!("Selection Rect: {:?}", active_selection.selection_rect)
        );
    }

    // 1 will switch to tile mode
    if input.just_pressed(KeyCode::Digit1) || input.just_pressed(KeyCode::Numpad1) {
        send_message!(Some('i'), messages, "Switching to Tile Mode");
        next_state.set(EditorState::Editing(EditingComponent::Tile));
    }

    // 2 will switch to collider mode
    if input.just_pressed(KeyCode::Digit2) || input.just_pressed(KeyCode::Numpad2) {
        send_message!(Some('i'), messages, "Switching to Collider Mode");
        next_state.set(EditorState::Editing(EditingComponent::Collider));
    }

    //state specific keybinds
    match state.get() {
        EditorState::Normal => {
            //there is not a lot unique to normal mode
        }
        EditorState::LoadAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_state.set(EditorState::Loading);
                send_message!(Some('i'), messages, "Attempting to load scene");
            }
            if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_state.set(EditorState::LoadingEmpty);
                send_message!(Some('w'), messages, "No scene loaded");
            }
        }
        EditorState::Loading => {
            //do nothing, we are waiting for the scene to load
        }
        EditorState::LoadingEmpty => {
            //do nothing, we are waiting for the scene to load
        }
        EditorState::Saving => {
            //do nothing, we are waiting for the scene to save
        }
        EditorState::Editing(_) => {
            //in any editing mode, Q will bring us back to normal mode
            if
                input.just_pressed(KeyCode::KeyQ) ||
                input.all_pressed(vec![KeyCode::ControlLeft, KeyCode::KeyS])
            {
                next_state.set(EditorState::Normal);
                send_message!(Some('i'), messages, "Returning to Normal Mode");
            }
        }

        EditorState::SaveAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_state.set(EditorState::Saving);
                send_message!(Some('i'), messages, "Saving scene...");
            } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_state.set(EditorState::Normal);
                send_message!(Some('w'), messages, "Saving aborted.");
            }
        }

        EditorState::QuitAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_state.set(EditorState::Inactive);
                send_message!(Some('i'), messages, "Exiting the editor...");
            } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_state.set(EditorState::Normal);
                send_message!(Some('w'), messages, "Exiting aborted.");
            }
        }
        _ => {}
    }

    //Anti-Stateful Keybinds
    if
        state.get() != &EditorState::SaveAsk &&
        state.get() != &EditorState::LoadAsk &&
        state.get() != &EditorState::QuitAsk
    {
        //Camera Controls, just dont work in the save/load/quit states
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
        //also update the crosshair in the same way
        for (_, mut t, _) in crosshairs.iter_mut() {
            t.translation.x += vel_x * time.delta_secs();
            t.translation.y += vel_y * time.delta_secs();
        }
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
    ///ultimatley an index into which style of tile or entity we are using within the major type, extra specificiation we can use to fine tune what object we are loading in this space.
    internal_type: u64,
    //the coordinate of the object as well as the major type of the object combined into a neat little package
    coordinate: TCoordinate,
}

impl EditorObject {
    // fn new(mt: char, it: u64, coord: Coordinate) -> Self {
    //     Self {
    //         internal_type: it,
    //         coordinate: TCoordinate::new(mt, coord),
    //     }
    // }

    fn get_major_type(&self) -> char {
        self.coordinate.type_char
    }
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

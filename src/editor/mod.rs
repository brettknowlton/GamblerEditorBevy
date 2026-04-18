use bevy::picking::window;
use bevy::window::PrimaryWindow;
use serde::Deserialize;
use serde::Serialize;

use crate::bottom_bar::EditorBottomBarQueuedMessages;
pub use crate::consts::*;
use crate::coordinate::*;

pub use crate::game::*;
pub use crate::resources::*;

use crate::selection::ActiveSelection;
use crate::selection::SelectionRect;

pub use crate::utilities::*;
pub use tools::SignificantComponent;

pub mod actor;

pub mod collider;

pub mod tile;
pub use tile::*;

#[macro_use]
pub mod ui;
pub use ui::*;

use bevy_rapier2d::prelude::*;

mod scene;
pub use std::{fmt::Debug, path::PathBuf};

use bevy::{prelude::*, sprite::Anchor};

//EditorState is an enum that defines the different states the editor can be in, this is used to determine what the editor is currently doing
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum EditorState {
    #[default]
    /// The editor is currently inactive and not performing any actions
    Inactive,
    /// The editor is in its normal state, ready for user interaction
    Normal,
    /// The editor is currently asking the user if they would like to load a scene
    LoadAsk,
    /// The editor is currently loading a scene
    Loading,
    /// The editor is currently loading an empty scene
    LoadingEmpty,
    /// The editor is currently asking the user if they would like to save the scene
    SaveAsk,
    /// The editor is currently saving the scene
    Saving,
    /// The editor is currently asking the user if they would like to quit
    QuitAsk,
    /// The editor is currently in an editing mode, allowing the user to place or modify objects in the scene
    Editing(EditorObjectKind),
}

impl EditorState {
    fn get_editing_kind(&self) -> Option<EditorObjectKind> {
        match self {
            EditorState::Editing(kind) => Some(*kind),
            _ => None,
        }
    }
}

/// A component that marks an entity as a savable editor item, from this we have systems that load Tiles, Colliders, and other objects based on preset-defaults and the other saved components we may have.
/// The main ones we need are the position of this object in the world, and the type of thing it is, and one more layer of optional specification on what "Kind of thing of thing" it is.
/// Other components will be used to determine the specifics of the object. but a tile for example can be completley determined from just this component.
/// eg.: Thing?: Tile. Kind of Thing?:0 (cut the spritesheet at index 0). Position: (0, 0), the logic for this is actually implemented on the SignificantComponent trait for each majortype of object
#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
pub struct EditorObject {
    /// ultimatley an index into which style of tile or entity we are using within the major type, extra specificiation we can use to fine tune what object we are loading in this space.
    /// for non-tile types this is currently always 0
    pub kind: EditorObjectKind,
    pub internal_kind: u64,
    //the coordinate of the object as well as the major type of the object combined into a neat little package
    pub coordinate: Coordinate,
    //this zone ID will track which zone the object is in, this is used to determine which zone to load the object into and to help with performance by only loading objects in the current/neighrboring zones
    pub zone_id: TCoordinate,
}

impl EditorObject {
    pub fn get_major_type(&self) -> EditorObjectKind {
        self.kind
    }
    pub fn get_internal_type(&self) -> u64 {
        self.internal_kind
    }
    pub fn get_coordinate(&self) -> Coordinate {
        self.coordinate.clone()
    }
}

#[derive(Default, Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum EditorObjectKind {
    #[default]
    Other,
    Tile,
    Collider,
    Actor,
    Selector,
}

/// This enum is used as a setting for the editor to determine wether or not we are trying to snap placed objects to the grid.
/// This is a user setting that can be toggled with CTRL + SHIFT + G, it is not saved to the document and will default to enabled on startup.
#[derive(Clone, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GridSnap {
    #[default]
    Enabled,
    Disabled,
}

#[derive(Clone, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ShowGrid {
    #[default]
    Yes,
    No,
}

pub fn build_editor_object(
    kind: EditorObjectKind,
    internal_kind: u64,
    coordinate: Coordinate,
    zone_kind: EditorObjectKind,
) -> EditorObject {
    EditorObject {
        kind,
        internal_kind,
        coordinate,
        zone_id: TCoordinate::new(
            zone_kind,
            coordinate.convert(CoordinateFormat::ZoneSpace, None, None),
        ),
    }
}

fn initialize(
    mut commands: Commands,

    mut next_state: ResMut<NextState<EditorState>>,
    mut message_queue: ResMut<ui::bottom_bar::EditorBottomBarQueuedMessages>,
    mut texture_handles: ResMut<TextureHandles>,

    asset_server: ResMut<AssetServer>,
) {
    //load the rect_debug texture into the RectHandle resource
    let tex_path = PathBuf::from("textures/tiles/tile_debug.png");
    texture_handles
        .0
        .insert(EditorObjectKind::Other, asset_server.load(tex_path));

    //create camera and add a UIItem component to it
    commands.spawn((Camera2d::default(), CameraLockedUI::default()));

    //set the state to ask about loading a scene
    next_state.set(EditorState::LoadAsk);

    //push a message to the bottom bar that asks the user if they would like to load a scene
    send_message!(Some('i'), message_queue, "Would you like to load a scene?");
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
            ..default()
        },
        Transform {
            scale: Vec3::new(TILE_SCALE as f32, TILE_SCALE as f32, 0.0),
            ..default()
        },
        Anchor::CENTER,
    ));
}

//wow this is kind of dope if it work
#[derive(Message)]
struct BottomBarUpdate;

#[derive(Message)]
struct ResetScene;

fn handle_global_editor_shortcuts(
    editor_state: &State<EditorState>,
    next_editor_state: &mut NextState<EditorState>,
    game_state: &State<GameState>,
    next_game_state: &mut NextState<GameState>,
    next_showgrid_state: &mut NextState<ShowGrid>,
    showgrid_state: &State<ShowGrid>,
    next_gridsnap_state: &mut NextState<GridSnap>,
    gridsnap_state: &State<GridSnap>,
    input: &ButtonInput<KeyCode>,

    message_queue: &mut ui::bottom_bar::EditorBottomBarQueuedMessages,

    crosshairs: &mut Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,
    message_writer: &mut MessageWriter<ResetScene>,
) {
    if input.just_pressed(KeyCode::KeyQ) {
        next_editor_state.set(EditorState::Normal);
        send_message!(Some('i'), message_queue, "Returning to Normal Mode");
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyS)
        && !input.just_pressed(KeyCode::ShiftLeft)
        && editor_state.get() != &EditorState::SaveAsk
    {
        next_editor_state.set(EditorState::SaveAsk);
        send_message!(
            Some('i'),
            message_queue,
            "Would you like to save the scene?"
        );
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyL)
        && editor_state.get() != &EditorState::LoadAsk
    {
        next_editor_state.set(EditorState::LoadAsk);
        send_message!(Some('i'), message_queue, "Would you like to load a scene?");
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyQ)
        && editor_state.get() != &EditorState::QuitAsk
    {
        next_editor_state.set(EditorState::QuitAsk);
        send_message!(
            Some('i'),
            message_queue,
            "Would you like to exit the editor?"
        );
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyT)
        && editor_state.get() != &EditorState::Inactive
    {
        if game_state.get() != &GameState::Inactive && editor_state.get() == &EditorState::Inactive
        {
            next_editor_state.set(EditorState::Normal);
            next_game_state.set(GameState::Inactive);
            send_message!(Some('i'), message_queue, "Exiting Test Mode");
        } else {
            next_editor_state.set(EditorState::Inactive);
            next_game_state.set(GameState::Running);
            send_message!(Some('i'), message_queue, "Entering Test Mode");
        }
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyR)
        && editor_state.get() != &EditorState::Inactive
    {
        send_message!(Some('i'), message_queue, "Resetting Scene");
        next_editor_state.set(EditorState::Normal);
        message_writer.write(ResetScene);
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyG)
        && !input.pressed(KeyCode::ShiftLeft)
        && editor_state.get() != &EditorState::Inactive
    {
        if showgrid_state.get() == &ShowGrid::Yes {
            next_showgrid_state.set(ShowGrid::No);
            send_message!(Some('i'), message_queue, "Hiding Grid");
        } else {
            next_showgrid_state.set(ShowGrid::Yes);
            send_message!(Some('i'), message_queue, "Showing Grid");
        }
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyG)
        && input.pressed(KeyCode::ShiftLeft)
        && editor_state.get() != &EditorState::Inactive
    {
        if gridsnap_state.get() == &GridSnap::Enabled {
            next_gridsnap_state.set(GridSnap::Disabled);
            send_message!(Some('i'), message_queue, "Disabling Grid Snap");
        } else {
            next_gridsnap_state.set(GridSnap::Enabled);
            send_message!(Some('i'), message_queue, "Enabling Grid Snap");
        }
    } else if input.pressed(KeyCode::ControlLeft)
        && input.just_pressed(KeyCode::KeyB)
        && editor_state.get() != &EditorState::Inactive
    {
        let Ok((_, transform, _)) = crosshairs.single() else {
            return;
        };

        let zone_id = Coordinate::zone_space(
            (transform.translation.x as i64) / ((ZONE_SIZE * SCALED_TILE_WIDTH) as i64),
            (transform.translation.y as i64) / ((ZONE_SIZE * SCALED_TILE_HEIGHT) as i64),
        );

        let path = PathBuf::from(format!("background{}{}.png", zone_id.x, zone_id.y));
        let aseprite_path =
            PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/Aseprite/Aseprite.exe");

        if path.exists() {
            send_message!(Some('i'), message_queue, "Opening background.png");
            std::process::Command::new(aseprite_path)
                .arg(path)
                .spawn()
                .expect("Failed to open aseprite");
        } else {
            send_message!(Some('i'), message_queue, "Creating background.png");
            std::fs::File::create(&path).expect("Failed to create background.png");
            std::process::Command::new("aseprite")
                .arg(path)
                .spawn()
                .expect("Failed to open aseprite");
        }
    }
}

fn handle_rectangle_tool_shortcuts(
    input: &ButtonInput<KeyCode>,
    message_queue: &mut EditorBottomBarQueuedMessages,
    crosshairs: &mut Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,
) {
    if input.just_pressed(KeyCode::KeyO) {
        let Ok((_, _, _)) = crosshairs.single() else {
            return;
        };

        send_message!(
            Some('i'),
            message_queue,
            "This feature is not yet implemented"
        );
    } else if input.just_released(KeyCode::KeyO) {
        let Ok((_, t, _)) = crosshairs.single() else {
            return;
        };
        let _ = Coordinate::game(t.translation.x as i64, t.translation.y as i64);
    }
}

fn handle_mode_switch_shortcuts(
    input: &ButtonInput<KeyCode>,
    next_editor_state: &mut NextState<EditorState>,
    message_queue: &mut EditorBottomBarQueuedMessages,
) {
    if input.just_pressed(KeyCode::Digit1) || input.just_pressed(KeyCode::Numpad1) {
        send_message!(Some('i'), message_queue, "Switching to Tile Mode");
        next_editor_state.set(EditorState::Editing(EditorObjectKind::Tile));
    }

    if input.just_pressed(KeyCode::Digit2) || input.just_pressed(KeyCode::Numpad2) {
        send_message!(Some('i'), message_queue, "Switching to Collider Mode");
        next_editor_state.set(EditorState::Editing(EditorObjectKind::Collider));
    }

    if input.just_pressed(KeyCode::Digit3) || input.just_pressed(KeyCode::Numpad3) {
        send_message!(Some('i'), message_queue, "Switching to Actor Mode");
        next_editor_state.set(EditorState::Editing(EditorObjectKind::Actor));
    }
}

fn handle_state_specific_keybinds(
    editor_state: &State<EditorState>,
    next_editor_state: &mut NextState<EditorState>,
    input: &ButtonInput<KeyCode>,
    message_queue: &mut EditorBottomBarQueuedMessages,
) {
    match editor_state.get() {
        EditorState::Normal
        | EditorState::Loading
        | EditorState::LoadingEmpty
        | EditorState::Saving => {}
        EditorState::LoadAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_editor_state.set(EditorState::Loading);
                send_message!(Some('i'), message_queue, "Attempting to load scene");
            }
            if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_editor_state.set(EditorState::LoadingEmpty);
                send_message!(Some('w'), message_queue, "No scene loaded");
            }
        }
        EditorState::Editing(_) => {
            if input.just_pressed(KeyCode::KeyQ)
                || input.all_pressed(vec![KeyCode::ControlLeft, KeyCode::KeyS])
            {
                next_editor_state.set(EditorState::Normal);
                send_message!(Some('i'), message_queue, "Returning to Normal Mode");
            }
        }
        EditorState::SaveAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_editor_state.set(EditorState::Saving);
                send_message!(Some('i'), message_queue, "Saving scene.");
            } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_editor_state.set(EditorState::Normal);
                send_message!(Some('w'), message_queue, "Saving aborted.");
            }
        }
        EditorState::QuitAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_editor_state.set(EditorState::Inactive);
                send_message!(Some('i'), message_queue, "Exiting the editor...");
            } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_editor_state.set(EditorState::Normal);
                send_message!(Some('w'), message_queue, "Exiting aborted.");
            }
        }
        _ => {}
    }
}

fn should_apply_editor_movement(editor_state: &State<EditorState>) -> bool {
    editor_state.get() != &EditorState::SaveAsk
        && editor_state.get() != &EditorState::LoadAsk
        && editor_state.get() != &EditorState::QuitAsk
}

fn camera_movement_velocity(input: &ButtonInput<KeyCode>) -> Vec2 {
    let mut velocity = Vec2::ZERO;

    if input.pressed(KeyCode::KeyW) && !input.pressed(KeyCode::KeyS) {
        velocity.y = 200.0;
    }
    if input.pressed(KeyCode::KeyS) && !input.pressed(KeyCode::KeyW) {
        velocity.y = -200.0;
    }
    if input.pressed(KeyCode::KeyD) && !input.pressed(KeyCode::KeyA) {
        velocity.x = 200.0;
    }
    if input.pressed(KeyCode::KeyA) && !input.pressed(KeyCode::KeyD) {
        velocity.x = -200.0;
    }

    velocity
}

fn apply_editor_movement(
    ui_items: &mut Query<
        (&mut CameraLockedUI, &mut Transform),
        (Without<Camera2d>, Without<Crosshair>),
    >,
    cameras: &mut Query<(&mut CameraLockedUI, &mut Transform), With<Camera2d>>,
    crosshairs: &mut Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,
    velocity: Vec2,
    delta_secs: f32,
) {
    //move ui_items (like the placeholder image)
    for (mut ui, mut t) in ui_items.iter_mut() {
        ui.vel_x = velocity.x;
        ui.vel_y = velocity.y;
        t.translation.x += ui.vel_x * delta_secs;
        t.translation.y += ui.vel_y * delta_secs;
        ui.vel_x *= 0.99;
        ui.vel_y *= 0.99;
    }
    //move camera
    for (mut ui, mut t) in cameras.iter_mut() {
        ui.vel_x = velocity.x;
        ui.vel_y = velocity.y;
        t.translation.x += ui.vel_x * delta_secs;
        t.translation.y += ui.vel_y * delta_secs;
        ui.vel_x *= 0.99;
        ui.vel_y *= 0.99;
    }

    //move crosshair
    for (_, mut t, _) in crosshairs.iter_mut() {
        t.translation.x += velocity.x * delta_secs;
        t.translation.y += velocity.y * delta_secs;
    }
}

#[derive(Resource)]
struct Dragging {
    dragging_btn: Option<MouseButton>,
    start_pos: Option<Vec2>,
}
impl Default for Dragging {
    fn default() -> Self {
        Self {
            dragging_btn: None,
            start_pos: None,
        }
    }
}

impl Dragging {
    pub fn is_dragging(&self) -> bool {
        self.dragging_btn.is_some()
    }
    
    fn start_drag(&mut self, btn: MouseButton, pos: Vec2) {
        self.dragging_btn = Some(btn);
        self.start_pos = Some(pos);
    }

    fn end_drag(&mut self) {
        self.dragging_btn = None;
        self.start_pos = None;
    }
}

fn listen_click_events(
    input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut dragging: ResMut<Dragging>,
) {
    if input.just_pressed(MouseButton::Left) {
        dragging.start_drag(
            MouseButton::Left,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Left) {
        dragging.end_drag();
    }

    if input.just_pressed(MouseButton::Right) {
        dragging.start_drag(
            MouseButton::Right,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Right) {
        dragging.end_drag();
    }

    if input.just_pressed(MouseButton::Middle) {
        dragging.start_drag(
            MouseButton::Middle,
            window.cursor_position().unwrap_or_default(),
        );
    }
    if input.just_released(MouseButton::Middle) {
        dragging.end_drag();
    }
}

fn stateful_keybinds(
    editor_state: ResMut<State<EditorState>>,
    mut next_editor_state: ResMut<NextState<EditorState>>,
    game_state: ResMut<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,

    mut next_showgrid_state: ResMut<NextState<ShowGrid>>,
    showgrid_state: ResMut<State<ShowGrid>>,

    mut next_gridsnap_state: ResMut<NextState<GridSnap>>,
    gridsnap_state: ResMut<State<GridSnap>>,

    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    // m_input: Res<ButtonInput<MouseButton>>,
    mut crosshairs: Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,
    // mut active_selection: ResMut<PlaceholderHandle>,
    mut cameras: Query<(&mut CameraLockedUI, &mut Transform), With<Camera2d>>,
    mut ui_items: Query<
        (&mut CameraLockedUI, &mut Transform),
        (Without<Camera2d>, Without<Crosshair>),
    >,
    mut message_writer: MessageWriter<ResetScene>,
) {
    handle_global_editor_shortcuts(
        editor_state.as_ref(),
        &mut next_editor_state,
        game_state.as_ref(),
        &mut next_game_state,
        &mut next_showgrid_state,
        showgrid_state.as_ref(),
        &mut next_gridsnap_state,
        gridsnap_state.as_ref(),
        &input,
        &mut message_queue,
        &mut crosshairs,
        &mut message_writer,
    );
    handle_rectangle_tool_shortcuts(&input, &mut message_queue, &mut crosshairs);

    handle_mode_switch_shortcuts(&input, &mut next_editor_state, &mut message_queue);

    handle_state_specific_keybinds(
        editor_state.as_ref(),
        &mut next_editor_state,
        &input,
        &mut message_queue,
    );

    if should_apply_editor_movement(editor_state.as_ref()) {
        let velocity = camera_movement_velocity(&input);
        apply_editor_movement(
            &mut ui_items,
            &mut cameras,
            &mut crosshairs,
            velocity,
            time.delta_secs(),
        );
    }
}

#[derive(Component)]
#[require(ui::CameraLockedUI)]
pub struct Crosshair; //tags the main crosshair entity, in the editor this happens to only be our camera, but may be taken over by a crosshair entity in the future that tracks the mouse

fn reset_scene(
    mut players: Query<
        (&mut actor::player::Player, &mut Transform, &mut Velocity),
        (Without<Crosshair>, Without<Camera2d>),
    >,
    mut cameras: Query<(&mut Transform, &mut Camera2d), Without<Crosshair>>,
    crosshairs: Query<&Transform, (With<Crosshair>, Without<Camera2d>)>, // mut ui_items: Query<(&mut UIItem, &mut Transform), Without<Crosshair>>,
) {
    //reset the player to the crosshair position
    let cs = crosshairs.single().unwrap().clone();

    for (_, mut t, mut vel) in players.iter_mut() {
        actor::player::move_player_to_cursor(cs, &mut t); // TODO This isnt working
        vel.linvel = Vec2::new(0.0, 0.0);
    }

    //reset the camera to the crosshair position
    let cs = crosshairs.single().unwrap().clone();
    for (mut t, _) in cameras.iter_mut() {
        t.translation = cs.translation;
    }
}

pub fn editor_plugin(app: &mut App) {
    app
        //states
        .init_state::<EditorState>()
        .init_state::<GridSnap>()
        .init_state::<ShowGrid>()
        //registrations
        .register_type::<EditorObject>()
        .add_message::<BottomBarUpdate>()
        .add_message::<ResetScene>()
        //resources
        .init_resource::<PlaceholderHandle>()
        .init_resource::<TextureHandles>()
        .init_resource::<ActiveSelection>()
        .init_resource::<Dragging>()
        // .init_resource::<ActiveSelection>()
        //plugins
        .add_plugins(ui::editor_ui_plugin)
        .add_plugins(tile::tilemode_plugin)
        .add_plugins(collider::collidermode_plugin)
        .add_plugins(scene::scene_plugin)
        .add_plugins(actor::actormode_plugin)
        //on entrance to this state, we give our placeholder object a handle to the default SignificantComponent of this mode- in normal mode this is a SelectionRect
        .add_systems(
            OnEnter(EditorState::Normal),
            (
                ui::hide_tooling_menu,
                ui::update_placeholder::<SelectionRect>,
                reset_scene,
            )
                .chain(),
        )
        //The only true startup systems here:
        .add_systems(Startup, (initialize, create_crosshair).chain())
        //universal update systems for all editing modes
        .add_systems(
            Update,
            (stateful_keybinds, listen_click_events)
                .chain()
                .run_if(not(in_state(EditorState::Inactive))),
        )
        .add_systems(Update, reset_scene.run_if(on_message::<ResetScene>));
}

//NOTHING BELOW THE PLUGINS >:(

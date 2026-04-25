pub mod editor_modes;

pub use crate::editor_modes::*;

use crate::grid::{GridSnap, ShowGrid};

use crate::message_display::{BottomBarUpdate, MessageDisplay};

pub mod mouse;
pub use mouse::*;

pub use crate::consts::*;
use crate::coordinate::*;

pub use crate::game::*;
pub use crate::resources::*;

pub use crate::utilities::*;



#[macro_use]
pub mod ui;
pub use ui::*;

use bevy_rapier2d::prelude::*;

mod scene;
use std::ops::DerefMut;
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
    pub fn get_editing_kind(&self) -> Option<EditorObjectKind> {
        match self {
            EditorState::Editing(kind) => Some(*kind),
            _ => None,
        }
    }
}

#[derive(Message)]
struct ResetScene;

pub struct Editor;

impl Editor {
    pub fn editor_plugin(app: &mut App) {
        app
            //states
            .init_state::<EditorState>()
            .init_state::<grid::GridSnap>()
            .init_state::<grid::ShowGrid>()
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
            .add_plugins(NormalModePlugin)
            .add_plugins(TileModePlugin)
            .add_plugins(ColliderModePlugin)
            .add_plugins(scene::scene_plugin)
            .add_plugins(actor_mode::actormode_plugin)
            //The only true startup systems here:
            .add_systems(Startup, Editor::initialize_editor)
            //universal update systems for all editor modes
            .add_systems(
                Update,
                (Editor::editor_keybinds, listen_click_events)
                    .chain()
                    .run_if(not(in_state(EditorState::Inactive))),
            )
            .add_systems(Update, Editor::reset_scene.run_if(on_message::<ResetScene>));
    }

    fn initialize_editor(
        mut commands: Commands,

        mut next_state: ResMut<NextState<EditorState>>,
        mut bottom_bar: ResMut<MessageDisplay>,
        mut texture_handles: ResMut<TextureHandles>,

        asset_server: Res<AssetServer>,
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

        Self::create_crosshair(commands, asset_server);

        //push a message to the bottom bar that asks the user if they would like to load a scene
        send_message!(
            Some('i'),
            bottom_bar.queue,
            "Would you like to load a scene?"
        );
    }

    fn reset_scene(
        commands: Commands,
        asset_server: Res<AssetServer>,
        mut player_q: Option<
            Single<
                (
                    Entity,
                    &mut actor_mode::player::Player,
                    &mut KinematicCharacterController,
                ),
                (Without<Crosshair>, Without<Camera2d>),
            >,
        >,
        mut cameras: Query<(&mut Transform, &mut Camera2d), Without<Crosshair>>,
        crosshair: Single<&Transform, (With<Crosshair>, Without<Camera2d>)>, // mut ui_items: Query<(&mut UIItem, &mut Transform), Without<Crosshair>>,
    ) {
        println!("Resetting scene...");
        //reset the player to the crosshair position
        let crosshair_position = crosshair.clone().translation;

        if let Some(player_q) = player_q.as_mut() {
            let (e, _, controller) = player_q.deref_mut();
            controller.translation = None;

            println!("Respawning player...");
            Player::respawn(commands, *e, asset_server, crosshair);
        } else {
            println!("No player found, spawning new player...");
            Player::spawn_player(commands, asset_server, crosshair);
        }

        for (mut t, _) in cameras.iter_mut() {
            t.translation = crosshair_position;
        }
    }

    fn editor_keybinds(
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
        mut bottom_bar: ResMut<MessageDisplay>,
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
        Editor::handle_global_editor_shortcuts(
            editor_state.as_ref(),
            &mut next_editor_state,
            game_state.as_ref(),
            &mut next_game_state,
            &mut next_showgrid_state,
            showgrid_state.as_ref(),
            &mut next_gridsnap_state,
            gridsnap_state.as_ref(),
            &input,
            &mut bottom_bar,
            &mut crosshairs,
            &mut message_writer,
        );
        Self::handle_rectangle_tool_shortcuts(&input, &mut bottom_bar, &mut crosshairs);

        Self::handle_mode_switch_shortcuts(&input, &mut next_editor_state, &mut bottom_bar);

        Self::handle_saving_controls(
            editor_state.as_ref(),
            &mut next_editor_state,
            &input,
            &mut bottom_bar,
        );

        if Editor::should_apply_editor_movement(editor_state.as_ref()) {
            let velocity = Editor::camera_movement_velocity(&input);
            Editor::apply_editor_movement(
                &mut ui_items,
                &mut cameras,
                &mut crosshairs,
                velocity,
                time.delta_secs(),
            );
        }
    }

    fn handle_rectangle_tool_shortcuts(
        input: &ButtonInput<KeyCode>,
        bottom_bar: &mut MessageDisplay,
        crosshairs: &mut Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,
    ) {
        if input.just_pressed(KeyCode::KeyO) {
            let Ok((_, _, _)) = crosshairs.single() else {
                return;
            };

            send_message!(
                Some('i'),
                bottom_bar.queue,
                "This feature is not yet implemented"
            );
        } else if input.just_released(KeyCode::KeyO) {
            let Ok((_, t, _)) = crosshairs.single() else {
                return;
            };
            let _ = Coordinate::new_world_space(t.translation.x as i64, t.translation.y as i64);
        }
    }

    fn handle_mode_switch_shortcuts(
        input: &ButtonInput<KeyCode>,
        next_editor_state: &mut NextState<EditorState>,
        bottom_bar: &mut MessageDisplay,
    ) {
        if input.just_pressed(KeyCode::Digit1) || input.just_pressed(KeyCode::Numpad1) {
            bottom_bar.send_mode_enter_message("Tile Mode");
            next_editor_state.set(EditorState::Editing(EditorObjectKind::Tile(TileID::Any)));
        }

        if input.just_pressed(KeyCode::Digit2) || input.just_pressed(KeyCode::Numpad2) {
            bottom_bar.send_mode_enter_message("Collider Mode");
            next_editor_state.set(EditorState::Editing(EditorObjectKind::Collider));
        }

        if input.just_pressed(KeyCode::Digit3) || input.just_pressed(KeyCode::Numpad3) {
            bottom_bar.send_mode_enter_message("Actor Mode");
            next_editor_state.set(EditorState::Editing(EditorObjectKind::Actor));
        }
    }

    fn handle_saving_controls(
        editor_state: &State<EditorState>,
        next_editor_state: &mut NextState<EditorState>,
        input: &ButtonInput<KeyCode>,
        bottom_bar: &mut MessageDisplay,
    ) {
        match editor_state.get() {
            EditorState::Editing(_) => {
                if input.just_pressed(KeyCode::KeyQ)
                    || input.all_pressed(vec![KeyCode::ControlLeft, KeyCode::KeyS])
                {
                    next_editor_state.set(EditorState::Normal);
                    send_message!(Some('i'), bottom_bar.queue, "Returning to Normal Mode");
                }
            }
            EditorState::LoadAsk => {
                if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                    next_editor_state.set(EditorState::Loading);
                    send_message!(Some('i'), bottom_bar.queue, "Attempting to load scene");
                }
                if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                    next_editor_state.set(EditorState::LoadingEmpty);
                    send_message!(Some('w'), bottom_bar.queue, "No scene loaded");
                }
            }
            EditorState::SaveAsk => {
                if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                    next_editor_state.set(EditorState::Saving);
                    send_message!(Some('i'), bottom_bar.queue, "Saving scene.");
                } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                    next_editor_state.set(EditorState::Normal);
                    send_message!(Some('w'), bottom_bar.queue, "Saving aborted.");
                }
            }
            EditorState::QuitAsk => {
                if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                    next_editor_state.set(EditorState::Inactive);
                    send_message!(Some('i'), bottom_bar.queue, "Exiting the editor...");
                } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                    next_editor_state.set(EditorState::Normal);
                    send_message!(Some('w'), bottom_bar.queue, "Exiting aborted.");
                }
            }
            _ => {}
        }
    }

    pub fn editor_is_dragging(dragging: Res<Dragging>) -> bool {
        dragging.is_dragging()
    }

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

        bottom_bar: &mut MessageDisplay,

        crosshairs: &mut Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,
        message_writer: &mut MessageWriter<ResetScene>,
    ) {
        if input.just_pressed(KeyCode::KeyQ) {
            next_editor_state.set(EditorState::Normal);
            bottom_bar.send_mode_exit_message("Normal Mode");
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyS)
            && !input.just_pressed(KeyCode::ShiftLeft)
            && editor_state.get() != &EditorState::SaveAsk
        {
            next_editor_state.set(EditorState::SaveAsk);
            bottom_bar.send_mode_exit_message("Save Ask Mode");
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyL)
            && editor_state.get() != &EditorState::LoadAsk
        {
            next_editor_state.set(EditorState::LoadAsk);
            bottom_bar.send_mode_exit_message("Load Ask Mode");
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyQ)
            && editor_state.get() != &EditorState::QuitAsk
        {
            next_editor_state.set(EditorState::QuitAsk);
            bottom_bar.send_mode_exit_message("Quit Ask Mode");
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyT)
            && editor_state.get() != &EditorState::Inactive
        {
            if game_state.get() != &GameState::Inactive
                && editor_state.get() == &EditorState::Inactive
            {
                next_editor_state.set(EditorState::Normal);
                next_game_state.set(GameState::Inactive);
                bottom_bar.send_mode_exit_message("Test Mode");
            } else {
                next_editor_state.set(EditorState::Inactive);
                next_game_state.set(GameState::Running);
                bottom_bar.send_mode_exit_message("Test Mode");
            }
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyR)
            && editor_state.get() != &EditorState::Inactive
        {
            bottom_bar.send_mode_exit_message("Reset Scene");
            next_editor_state.set(EditorState::Normal);
            message_writer.write(ResetScene);
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyG)
            && !input.pressed(KeyCode::ShiftLeft)
            && editor_state.get() != &EditorState::Inactive
        {
            if showgrid_state.get() == &ShowGrid::Yes {
                next_showgrid_state.set(ShowGrid::No);
                bottom_bar.send_mode_exit_message("Hiding Grid");
            } else {
                next_showgrid_state.set(ShowGrid::Yes);
                bottom_bar.send_mode_exit_message("Showing Grid");
            }
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyG)
            && input.pressed(KeyCode::ShiftLeft)
            && editor_state.get() != &EditorState::Inactive
        {
            if gridsnap_state.get() == &GridSnap::Enabled {
                next_gridsnap_state.set(GridSnap::Disabled);
                bottom_bar.send_setting_update_message("Grid Snap");
            } else {
                next_gridsnap_state.set(GridSnap::Enabled);
                bottom_bar.send_setting_update_message("Grid Snap");
            }
        } else if input.pressed(KeyCode::ControlLeft)
            && input.just_pressed(KeyCode::KeyB)
            && editor_state.get() != &EditorState::Inactive
        {
            let Ok((_, transform, _)) = crosshairs.single() else {
                return;
            };

            let zone_id = Coordinate::new(
                transform.translation.x as i64,
                transform.translation.y as i64,
                CoordinateSpace::WorldSpace,
            )
            .as_zone_space(None, None);

            let path = PathBuf::from(format!("background{}{}.png", zone_id.x, zone_id.y));
            let aseprite_path = PathBuf::from(
                "C:/Program Files (x86)/Steam/steamapps/common/Aseprite/Aseprite.exe",
            );

            if path.exists() {
                send_message!(Some('i'), bottom_bar.queue, "Opening background.png");
                std::process::Command::new(aseprite_path)
                    .arg(path)
                    .spawn()
                    .expect("Failed to open aseprite");
            } else {
                send_message!(Some('i'), bottom_bar.queue, "Creating background.png");
                std::fs::File::create(&path).expect("Failed to create background.png");
                std::process::Command::new("aseprite")
                    .arg(path)
                    .spawn()
                    .expect("Failed to open aseprite");
            }
        }
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
}

impl Plugin for Editor {
    fn build(&self, app: &mut App) {
        Editor::editor_plugin(app);
    }
}

#[derive(Component)]
#[require(ui::CameraLockedUI)]
pub struct Crosshair; //tags the main crosshair entity, in the editor this happens to only be our camera, but may be taken over by a crosshair entity in the future that tracks the mouse

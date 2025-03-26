#[macro_use]
pub mod ui;
use selection::ActiveSelection;
use selection::SelectionRect;
pub use ui::*;

pub use tile::*;
use tools::SignificantComponent;
pub use crate::consts::*;
use crate::game;
pub use crate::utilities::*;
pub use crate::resources::*;

pub mod tile;
pub mod collider;
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

///The EditingComponent enum is used to determine what type of object the user is currently trying to place in the scene, this is kind of like an "Internal Type" for the editor state.
///We will almost always be in EditorMode::Editing but its more granular than that
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

#[derive(Event)]
pub struct UpdatePlaceholderEvent{
    pub tcoord: TCoordinate,
    pub rect: Rect
}


fn initialize(
    mut commands: Commands,

    mut next_state: ResMut<NextState<EditorState>>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    mut texture_handles: ResMut<TextureHandles>,

    asset_server: ResMut<AssetServer>
) {
    //load the rect_debug texture into the RectHandle resource
    let tex_path = PathBuf::from("textures/tiles/tile_debug.png");
    texture_handles.0.insert('r', asset_server.load(tex_path));

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

#[derive(Event)]
struct ResetScene;

fn stateful_keybinds(
    editor_state: ResMut<State<EditorState>>,
    mut next_editor_state: ResMut<NextState<EditorState>>,
    game_state: ResMut<State<game::GameState>>,
    mut next_game_state: ResMut<NextState<game::GameState>>,

    
    mut next_showgrid_state: ResMut<NextState<ShowGrid>>,
    showgrid_state: ResMut<State<ShowGrid>>,

    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut message_queue: ResMut<EditorBottomBarQueuedMessages>,
    // m_input: Res<ButtonInput<MouseButton>>,
    mut crosshairs: Query<(&mut Crosshair, &mut Transform, &mut Sprite), Without<Camera2d>>,
    // mut active_selection: ResMut<PlaceholderHandle>,
    mut uiitems: Query<(&mut UIItem, &mut Transform), (Without<Camera2d>, Without<Crosshair>)>,
    mut cameras: Query<(&mut UIItem, &mut Transform, &mut Camera2d)>,
    mut event_writer: EventWriter<ResetScene>,
) {
    let messages = &mut message_queue;
    //manage the editor state, you can switch between modes with their letter call or number keys except if you are attempting to save/load the document

    //Universal keys, not dependant on state:

    // Q brings us back to normal mode
    if input.just_pressed(KeyCode::KeyQ) {
        next_editor_state.set(EditorState::Normal);
        send_message!(Some('i'), messages, "Returning to Normal Mode");
    } else if
        //CTRL + S will enter saveAsk mode
        input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyS) && ! input.just_pressed(KeyCode::ShiftLeft) &&
        editor_state.get() != &EditorState::SaveAsk
    {
        next_editor_state.set(EditorState::SaveAsk);
        send_message!(Some('i'), messages, "Would you like to save the scene? Yenter/Noscape");
    } else if
        //CTRL + L will enter loadAsk mode
        input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyL) &&
        editor_state.get() != &EditorState::LoadAsk
    {
        next_editor_state.set(EditorState::LoadAsk);
        send_message!(Some('i'), messages, "Would you like to load a scene? Yenter/Noscape");
    } else if
        //CTRL + Q will enter QuitAsk mode
        input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyQ) &&
        editor_state.get() != &EditorState::QuitAsk
    {
        next_editor_state.set(EditorState::QuitAsk);
        send_message!(Some('i'), messages, "Would you like to exit the editor? Yenter/Noscape");
    } else if 
        //CTRL + T will toggle TEST mode, disabling the editor and enabling the game functionality.
        input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyT) &&
        editor_state.get() != &EditorState::Inactive
    {
        if game_state.get() != &game::GameState::Inactive && editor_state.get() == &EditorState::Inactive {
            next_editor_state.set(EditorState::Normal);
            next_game_state.set(game::GameState::Inactive);
            send_message!(Some('i'), messages, "Exiting Test Mode");
        } else {
            next_editor_state.set(EditorState::Inactive);
            next_game_state.set(game::GameState::Running);
            send_message!(Some('i'), messages, "Entering Test Mode");
        }
    } else if 
        //CTRL + R will "reset" the editor, for now that willjust move the player to the crosshair position
        input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyR) && editor_state.get() != &EditorState::Inactive
    {
        // game::player::move_player_to_cursor(crosshairs.single_mut().unwrap().1, &mut cameras.single_mut().unwrap().1);
        send_message!(Some('i'), messages, "Resetting Scene");
        next_editor_state.set(EditorState::Normal);
        //send a ResetScene event to the ResetScene system
        event_writer.send(ResetScene);
    } else if 
        //CTRL + G will toggle the grid
        input.pressed(KeyCode::ControlLeft) && input.just_pressed(KeyCode::KeyG) && editor_state.get() != &EditorState::Inactive
    {
        if showgrid_state.get() == &ShowGrid::Yes {
            next_showgrid_state.set(ShowGrid::No);
            send_message!(Some('i'), messages, "Hiding Grid");
        } else {
            next_showgrid_state.set(ShowGrid::Yes);
            send_message!(Some('i'), messages, "Showing Grid");
        }
    }


    //O is the main button to directly use the Rectangle Tool
    if input.just_pressed(KeyCode::KeyO) {
        //use crosshair's coordinate as start
        let (_, _, _) = crosshairs.single();

        // active_selection.selection_rect = Some(selection::SelectionRect::start(coord));

        send_message!(Some('i'), messages, "This feature is not yet implemented");
    } else if input.just_released(KeyCode::KeyO) {
        //use crosshair's coordinate as end
        let (_, t, _) = crosshairs.single();
        let _ = Coordinate { 0: t.translation.x as i64, 1: t.translation.y as i64 };

        // active_selection.selection_rect = active_selection.selection_rect.clone().map(|mut r| {
        //     r.end(coord);
        //     r
        // });

        // send_message!(
        //     Some('i'),
        //     messages,
        //     format!("Selection Rect: {:?}", active_selection.selection_rect)
        // );
    }

    // 1 will switch to tile mode
    if input.just_pressed(KeyCode::Digit1) || input.just_pressed(KeyCode::Numpad1) {
        send_message!(Some('i'), messages, "Switching to Tile Mode");
        next_editor_state.set(EditorState::Editing(EditingComponent::Tile));
    }

    // 2 will switch to collider mode
    if input.just_pressed(KeyCode::Digit2) || input.just_pressed(KeyCode::Numpad2) {
        send_message!(Some('i'), messages, "Switching to Collider Mode");
        next_editor_state.set(EditorState::Editing(EditingComponent::Collider));
    }

    //state specific keybinds
    match editor_state.get() {
        EditorState::Normal => {
            //there is not a lot unique to normal mode
        }
        EditorState::LoadAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_editor_state.set(EditorState::Loading);
                send_message!(Some('i'), messages, "Attempting to load scene");
            }
            if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_editor_state.set(EditorState::LoadingEmpty);
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
                next_editor_state.set(EditorState::Normal);
                send_message!(Some('i'), messages, "Returning to Normal Mode");
            }
        }

        EditorState::SaveAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_editor_state.set(EditorState::Saving);
                send_message!(Some('i'), messages, "Saving scene.");
            } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_editor_state.set(EditorState::Normal);
                send_message!(Some('w'), messages, "Saving aborted.");
            }
        }

        EditorState::QuitAsk => {
            if input.just_pressed(KeyCode::KeyY) || input.just_pressed(KeyCode::Enter) {
                next_editor_state.set(EditorState::Inactive);
                send_message!(Some('i'), messages, "Exiting the editor...");
            } else if input.just_pressed(KeyCode::KeyN) || input.just_pressed(KeyCode::Escape) {
                next_editor_state.set(EditorState::Normal);
                send_message!(Some('w'), messages, "Exiting aborted.");
            }
        }
        _ => {}
    }

    //Anti-Stateful Keybinds
    if
        editor_state.get() != &EditorState::SaveAsk &&
        editor_state.get() != &EditorState::LoadAsk &&
        editor_state.get() != &EditorState::QuitAsk
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
#[require(ui::UIItem)]
pub struct Crosshair {} //tags the main crosshair entity, in the editor this happens to only be our camera, but may be taken over by a crosshair entity in the future that tracks the mouse

/// A component that marks an entity as a savable editor item, from this we have systems that load Tiles, Colliders, and other objects based on preset-defaults and the other saved components we may have.
/// The main ones we need are the position of this object in the world, and the type of thing it is, and one more layer of optional specification on what "Kind of thing of thing" it is.
/// Other components will be used to determine the specifics of the object. but a tile for example can be completley determined from just this component.
/// eg.: Thing?: Tile. Kind of Thing?:0 (cut the spritesheet at index 0). Position: (0, 0), the logic for this is actually implemented on the SignificantComponent trait for each majortype of object
#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
pub struct EditorObject {
    ///ultimatley an index into which style of tile or entity we are using within the major type, extra specificiation we can use to fine tune what object we are loading in this space.
    pub internal_type: u64,
    //the coordinate of the object as well as the major type of the object combined into a neat little package
    pub coordinate: TCoordinate,
    //this zone ID will track which zone the object is in, this is used to determine which zone to load the object into and to help with performance by only loading objects in the current/neighrboring zones
    pub zone_id: TCoordinate,
}

impl EditorObject {
    // fn new(mt: char, it: u64, coord: Coordinate) -> Self {
    //     Self {
    //         internal_type: it,
    //         coordinate: TCoordinate::new(mt, coord),
    //     }
    // }

    pub fn get_major_type(&self) -> char {
        self.coordinate.type_char
    }
    pub fn get_internal_type(&self) -> u64 {
        self.internal_type
    }
    pub fn get_coordinate(&self) -> TCoordinate {
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

fn reset_scene(
    mut players: Query<(&mut game::player::Player, &mut Transform), (Without<Crosshair>, Without<Camera2d>)>,
    mut cameras: Query<(&mut Transform, &mut Camera2d), Without<Crosshair>>,
    crosshairs: Query<&Transform, (With<Crosshair>, Without<Camera2d>)>,
    // mut ui_items: Query<(&mut UIItem, &mut Transform), Without<Crosshair>>,
){
    //reset the player to the crosshair position
    let cs = crosshairs.single().clone();
    for (mut player, mut t) in players.iter_mut() {
        game::player::move_player_to_cursor(cs, &mut t);
        player.velocity = Vec2::new(0.0, 0.0);
    }

    //reset the camera to the crosshair position
    let cs = crosshairs.single().clone();
    for (mut t, _) in cameras.iter_mut() {
        t.translation = cs.translation;
    }

}

fn draw_grid(mut gizmos: Gizmos) {
    gizmos
        .grid_2d(
            Isometry2d::new(Vec2::new(0.0, 0.0), Rot2::degrees(0.)),
            UVec2::new(100, 100),
            Vec2::new((TILE_SIZE * TILE_SCALE) as f32, (TILE_SIZE * TILE_SCALE) as f32),
            Color::srgba(0.0, 1.0, 0.0, 0.5)
        )
        .outer_edges();

    gizmos
        .grid_2d(
            Isometry2d::IDENTITY,
            UVec2::new(10, 10),
            Vec2::new((TILE_SIZE * TILE_SCALE * ZONE_SIZE) as f32, (TILE_SIZE * TILE_SCALE * ZONE_SIZE) as f32),
            Color::srgba(1.0, 0.0, 0.0, 0.5)
        )
        .outer_edges();
}

pub fn editor_plugin(app: &mut App) {
    app
        //states
        .init_state::<EditorState>()
        .init_state::<GridSnap>()
        .init_state::<ShowGrid>()

        //registrations
        .register_type::<EditorObject>()
        .add_event::<BottomBarUpdate>()
        .add_event::<UpdatePlaceholderEvent>()

        .add_event::<ResetScene>()

        //resources
        .init_resource::<EditorBottomBarDisplayed>()
        .init_resource::<EditorBottomBarMessage>()
        .init_resource::<EditorBottomBarQueuedMessages>()

        .init_resource::<PlaceholderHandle>()
        .init_resource::<TextureHandles>()
        .init_resource::<ActiveSelection>()

        // .init_resource::<ActiveSelection>()

        //begin update system to send debug messages (to bottom bar and to console)
        .add_systems(Update, ui::send_messages)

        //plugins
        .add_plugins(tile::tilemode_plugin)
        .add_plugins(collider::collidermode_plugin)
        .add_plugins(scene::scene_plugin)

        //on entrance to this state, we give our placeholder object a handle to the default SignificantComponent of this mode- in normal mode this is a SelectionRect
        .add_systems(
            OnEnter(EditorState::Normal),
            (ui::update_placeholder::<SelectionRect>).chain()
        )


        //The only true startup systems here:
        .add_systems(Startup, (initialize, create_crosshair, ui::spawn_general_editor_ui).chain())

        //universal update systems for all editing modes
        .add_systems(Update, stateful_keybinds.run_if(not(in_state(EditorState::Inactive))))
        .add_systems(Update, draw_grid.run_if(in_state(ShowGrid::Yes)))
        .add_systems(Update, ui::trigger_placeholder_update)
        
        .add_systems(Update, (reset_scene).chain().run_if(on_event::<ResetScene>));

}
//NOTHING BELOW THE PLUGINS >:(

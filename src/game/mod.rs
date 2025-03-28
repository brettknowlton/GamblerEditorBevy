use super::editor::collider::*;
use super::editor::*;

use bevy::prelude::*;

//EditorState is an enum that defines the different states the editor can be in, this is used to determine what the editor is currently doing
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Inactive,
    Running,
    Paused,
    GameOver,
    Saving,
    Loading,
    Cutscene,
}

pub fn game_plugin(app: &mut App) {
    app.init_state::<GameState>()
        //OnEnter systems
        .add_systems(
            OnEnter(GameState::Inactive),
            (load_save_data, actor::player::spawn_player).chain(),
        )
        .add_systems(
            FixedUpdate,
            (
                game_keybinds,
                actor::player::player_controls,
                // actor::player::player_physics,
                // player::do_player_collision,
                rapier_physics_systems
            )
                .chain()
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(Update, player_camera.run_if(in_state(GameState::Running)))
    // .add_systems(
    //     OnEnter(GameState::Loading),
    //     ().chain()
    // );
}


fn game_keybinds(
    editor_state: ResMut<State<EditorState>>,
    mut next_editor_state: ResMut<NextState<EditorState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.all_pressed(vec![KeyCode::KeyR, KeyCode::ControlLeft]) {
        if *editor_state.get() == EditorState::Inactive {
            next_editor_state.set(EditorState::Normal);
            next_game_state.set(GameState::Paused);
        }
    }
}

fn player_camera(
    players: Query<(&actor::player::Player, &Transform), Without<Camera>>,
    mut camera_query: Query<(&mut Camera, &mut Transform)>,
) {
    for (_, player_t) in players.iter() {
        for (_, mut t) in camera_query.iter_mut() {
            let mut new_t = player_t.clone();
            new_t.translation.z = t.translation.z;
            new_t.translation.x += SCALED_PLAYER_WIDTH as f32 / 2.;
            new_t.translation.y += SCALED_PLAYER_HEIGHT as f32 / 2.;

            t.translation = new_t.translation;
        }
    }
}

#[derive(States, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PlayerState {
    Idle,
    Walking,
    Running,
    Attacking,
    Hurt,
    Dead,
}

#[derive(Clone, Reflect, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Debug)]
pub struct AnimationDef {
    pub frame_size: Vec2,
    pub layout: Vec2, //(rows, columns), ie # of frames in each direction of the spritesheet a 3x4 spritesheet would be (3, 4)
    pub frame_count: u32, //total number of frames in the animation
    pub frame_duration: f32,
    pub current_frame: u32,
    pub frame_timer: f32,
}

fn load_save_data(_commands: Commands, _asset_server: Res<AssetServer>) {
    // let save_data = asset_server.load("save_data.json");
    // commands.insert_resource(SaveData(save_data));
    println!("you will be loading save data in this funciton... ");
}

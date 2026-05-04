use crate::editor_modes::actor_mode::{self, animation::drive_sprite_animations};

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
        .add_systems(OnEnter(GameState::Inactive), load_save_data)
        .add_systems(
            Update,
            (
                game_keybinds,
                Player::player_controls,
                Player::update_player_animation_state,
                drive_sprite_animations,
            )
                .chain()
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(
            FixedUpdate,
            (Player::player_physics).run_if(in_state(GameState::Running)),
        )
        .add_systems(Update, player_camera.run_if(in_state(GameState::Running)));
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
    player: Single<(&actor_mode::player::Player, &Transform), Without<Camera>>,
    mut camera_query: Query<(&mut Camera, &mut Transform)>,
    time: Res<Time>,
) {
    let player_t = player.1.translation;
    let alpha = (time.delta_secs() * 18.0).clamp(0.0, 1.0);
    for (_, mut t) in camera_query.iter_mut() {
        t.translation.x = t.translation.x.lerp(player_t.x, alpha);
        t.translation.y = t.translation.y.lerp(player_t.y, alpha);
    }
}

fn load_save_data(_commands: Commands, _asset_server: Res<AssetServer>) {
    // let save_data = asset_server.load("save_data.json");
    // commands.insert_resource(SaveData(save_data));
    println!("you will be loading save data in this funciton... ");
}

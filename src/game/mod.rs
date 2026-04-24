use crate::editor_modes::actor;

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
                actor::player::player_controls,
                actor::player::player_physics,
                // player::do_player_collision,
            )
                .chain()
                .run_if(in_state(GameState::Running)),
        )
        .add_systems(Update, player_camera.run_if(in_state(GameState::Running)));
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
    player: Single<(&actor::player::Player, &Transform), Without<Camera>>,
    mut camera_query: Query<(&mut Camera, &mut Transform)>,
) {
    let player_t = player.1.translation;
    for (_, mut t) in camera_query.iter_mut() {
        t.translation = Vec3::new(player_t.x, player_t.y, t.translation.z);
    }
}

fn load_save_data(_commands: Commands, _asset_server: Res<AssetServer>) {
    // let save_data = asset_server.load("save_data.json");
    // commands.insert_resource(SaveData(save_data));
    println!("you will be loading save data in this funciton... ");
}

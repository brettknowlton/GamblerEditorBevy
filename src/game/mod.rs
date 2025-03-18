use std::path::PathBuf;
use super::editor::*;

use bevy::prelude::*;

mod player;

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
        .add_systems(OnEnter(GameState::Inactive), (load_save_data, player::spawn_player).chain())

        .add_systems(Update, player::player_physics.run_if(in_state(GameState::Running)));
    // .add_systems(
    //     OnEnter(GameState::Loading),
    //     ().chain()
    // );
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Debug)]
pub struct AnimationDef {
    frame_size: Vec2,
    layout: Vec2,//(rows, columns), ie # of frames in each direction of the spritesheet a 3x4 spritesheet would be (3, 4)
    pub frame_count: u32,//total number of frames in the animation
    pub frame_duration: f32,
    pub current_frame: u32,
    pub frame_timer: f32,
}


fn load_save_data(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let save_data = asset_server.load("save_data.json");
    // commands.insert_resource(SaveData(save_data));
    println!("you will be loading save data in this funciton... ");
}

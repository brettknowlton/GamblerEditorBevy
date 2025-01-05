use bevy::prelude::*;

#[derive(Default)]
pub enum EditorState {
    #[default]
    Inactive,
    NormalMode,
    TileMode,
    ActorMode,
    InteractableMode,
}


 use tiles;
 use interactables;
 use editor_ui;
 use actors;
 use utilities;

 pub fn editor_plugin(app: &mut App) {
    app
    .add_systems(Startup, initialize)
    .add_systems(Update, move_camera.run_if(!in_state(EditorState::Inactive)))
 }


 fn initialize(mut commands: Commands) {
    //create grid

    //load assets

    todo!()
 }
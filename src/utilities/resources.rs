use bevy::prelude::*;
use crate::Tile;


#[derive(Resource, Debug, Component)]
pub struct PlaceholderTile(pub Tile);

/// A handle to the tilesheet image.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct TilesheetHandle(pub Handle<Image>);

#[derive(Resource)]
pub struct EditorBottomBarDisplayed{pub text: String}
impl Default for EditorBottomBarDisplayed {
    fn default() -> Self {
        Self{text: "".to_string()}
    }
}

#[derive(Resource)]
pub struct EditorBottomBarQueuedMessages{pub messages: Vec<(Option<char>, String)>}
impl Default for EditorBottomBarQueuedMessages {
    fn default() -> Self {
        Self{messages: vec![]}
    }
}

#[derive(Resource)]
pub struct EditorBottomBarQueued{pub text: String}
impl Default for EditorBottomBarQueued {
    fn default() -> Self {
        Self{text: "".to_string()}
    }
}

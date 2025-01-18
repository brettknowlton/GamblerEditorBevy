use bevy::prelude::*;
use crate::EditorObject;


#[derive(Resource, Debug, Component)]
pub struct PlaceholderObject(pub EditorObject);


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
pub struct EditorBottomBarMessage{pub text: String}
impl Default for EditorBottomBarMessage {
    fn default() -> Self {
        Self{text: "".to_string()}
    }
}

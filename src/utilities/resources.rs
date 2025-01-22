use bevy::{prelude::*, utils::HashMap};
use tools::SignificantComponent;
use super::*;
use crate::EditorObject;


#[derive(Resource, Default)]
/// A placeholder object is a temporary object that is used to represent an object that will be created later,
///  we display this as a half-alpha sprite of the sprite that would-be placed..
pub struct PlaceholderHandle(pub Handle<Image>);


#[derive(Resource, Default)]
///All loaded spritesheets are added to this hashmap, with a character key signiftying the mode
/// t: tiles
/// c: colliders
/// r: editor_rects
/// 
pub struct TextureHandles(pub HashMap<char, Handle<Image>>);



// /// A handle to the tilesheet image.
// #[derive(Resource, Default, Reflect)]
// #[reflect(Resource)]
// pub struct TilesheetHandle(pub Handle<Image>);

// /// A handle to our debug_rect image.
// /// A handle to the tilesheet image.
// #[derive(Resource, Default, Reflect)]
// #[reflect(Resource)]
// pub struct RectHandle(pub Handle<Image>);



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

use bevy::prelude::*;
use std::collections::HashMap;

use crate::editor_modes::EditorObjectKind;

#[derive(Resource, Default)]
/// A placeholder object is a temporary object that is used to represent an object that will be created later,
///  we display this as a half-alpha sprite of the sprite that would-be placed..
pub struct PlaceholderHandle(pub Handle<Image>);

#[derive(Resource, Default)]
///All loaded spritesheets are added to this hashmap, with a EditorObjectKind key signifying the mode it relates to
/// t: tiles
/// c: colliders
/// r: editor_rects
/// f: formatting

pub struct TextureHandles(pub HashMap<EditorObjectKind, Handle<Image>>);

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
pub struct SelectedTileID(pub u64);

impl Default for SelectedTileID {
    fn default() -> Self {
        Self(0)
    }
}

#[derive(Resource)]
pub struct TileUpdateNeeded(pub bool);
impl Default for TileUpdateNeeded {
    fn default() -> Self {
        Self(false)
    }
}

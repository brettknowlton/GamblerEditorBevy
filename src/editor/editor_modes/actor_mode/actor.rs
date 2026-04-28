use bevy::prelude::*;
use bevy::reflect::Reflect;

use crate::coordinate::Coordinate;
use crate::significant_component::SignificantComponent;
use crate::{EditorObject, EditorObjectKind};

#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[require(EditorObject)]
pub struct Actor {
    pub internal_type: u64,
    pub coordinate: Coordinate,
    pub rect: Rect,
}

impl Actor {
    pub fn new() -> Self {
        Self {
            internal_type: 0,
            coordinate: Coordinate::new_world_space(0, 0),
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}

impl Default for Actor {
    fn default() -> Self {
        Self::new()
    }
}
impl SignificantComponent for Actor {
    fn place_rectangle(_rect: Rect, _commands: Commands) {
        //make a tile like normal in this rect, but use sliced tiles over the sprite sheet selection
        todo!();
    }

    fn at_coordinate(_coord: Coordinate) -> Self {
        Self::new()
    }

    fn relevant_editor_object(&self) -> EditorObjectKind {
        EditorObjectKind::Actor
    }
    fn to_type_string(&self) -> String {
        format!("actor_{}", self.internal_type)
    }
}

use std::error::Error;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    coordinate::{Coordinate, CoordinateFormatConversion, CoordinateSpace, TCoordinate},
};

pub mod selection;
pub use selection::ActiveSelection;

pub mod significant_component;

pub mod actor_mode;
pub use actor_mode::player::Player;

pub mod collider_mode;
pub use collider_mode::ColliderModePlugin;

pub mod tile;
pub use tile::{TileModePlugin, TileID};


pub mod normal_mode;
pub use normal_mode::NormalModePlugin;

pub mod editor_mode;
pub use editor_mode::*;

#[derive(Default, Reflect, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum EditorObjectKind {
    #[default]
    None,
    Other,
    Tile(TileID),
    Collider,
    Actor,
    Selector,
}

#[derive(Debug, Clone)]
pub struct ErrorInvalidEditorObjectKind;

impl std::fmt::Display for ErrorInvalidEditorObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid EditorObjectKind")
    }
}

impl Error for ErrorInvalidEditorObjectKind {}

/// A component that marks an entity as a savable editor item, from this we have systems that load Tiles, Colliders, and other objects based on preset-defaults and the other saved components we may have.
/// The main ones we need are the position of this object in the world, and the type of thing it is, and one more layer of optional specification on what "Kind of thing of thing" it is.
/// Other components will be used to determine the specifics of the object. but a tile for example can be completley determined from just this component.
/// eg.: Thing?: Tile. Kind of Thing?:0 (cut the spritesheet at index 0). Position: (0, 0), the logic for this is actually implemented on the SignificantComponent trait for each majortype of object
#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
pub struct EditorObject {
    /// ultimatley an index into which style of tile or entity we are using within the major type, extra specificiation we can use to fine tune what object we are loading in this space.
    /// for non-tile types this is currently always 0
    pub kind: EditorObjectKind,
    //the coordinate of the object as well as the major type of the object combined into a neat little package
    pub coordinate: Coordinate,
    //this zone ID will track which zone the object is in, this is used to determine which zone to load the object into and to help with performance by only loading objects in the current/neighrboring zones
    pub zone_id: TCoordinate,
}

impl EditorObject {
    pub fn get_major_type(&self) -> EditorObjectKind {
        self.kind
    }
    pub fn get_internal_type(&self) -> Option<u64> {
        match self.kind {
            EditorObjectKind::Tile(internal_kind) => match internal_kind {
                TileID::Some(id) => Some(id),
                _ => None,
            },
            _ => None,
        }
    }
    pub fn get_coordinate(&self) -> Coordinate {
        self.coordinate.clone()
    }
    pub fn new(
        kind: EditorObjectKind,
        coordinate: Coordinate,
        zone_kind: EditorObjectKind,
    ) -> EditorObject {
        EditorObject {
            kind,
            coordinate,
            zone_id: TCoordinate::new(
                zone_kind,
                coordinate.convert(CoordinateSpace::ZoneSpace, None, None),
            ),
        }
    }
}

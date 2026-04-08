use bevy::prelude::*;
use serde::{ Serialize, Deserialize };



use crate::{ TILE_SCALE, TILE_SIZE };
use crate::editor::EditorObjectKind;

pub mod resources;
pub mod tools;
pub mod selection;

//Helper Functions
pub fn despawn_all<T: Component>(mut commands: Commands, to_despawn: Query<Entity, With<T>>) {
    for e in to_despawn.iter() {
        commands.entity(e).despawn();
    }
}

pub fn snap_value_to_grid(value: i64, grid_size: i64) -> i64 {
    //floor x and y values to the last multiple of grid_size
    //if coordinate is negative, this will round down to the nearest multiple of grid_size further negative
    let x: i64;
    if value < 0 {
        x = value - (grid_size + (value % grid_size));
    } else {
        x = value - (value % grid_size);
    }

    x
}

pub fn snap_coordinate_to_grid(coord: Coordinate) -> Coordinate {
    Coordinate(
        snap_value_to_grid(coord.0, (TILE_SIZE * TILE_SCALE) as i64),
        snap_value_to_grid(coord.1, (TILE_SIZE * TILE_SCALE) as i64),
    )
}



///A Coordinate is a simple struct that holds two i64 values, x and y identifying a point in our editor
/// most items are anchored to Bottom Left, so the x and y values (generally) define the bottom left corner of the object
#[derive(
    Component,
    Reflect,
    Deserialize,
    Serialize,
    Debug,
    Hash,
    Eq,
    PartialEq,
    Clone,
    Copy,
    Default
)]
#[reflect(Component)]
pub struct Coordinate(pub i64, pub i64);

impl Coordinate {
    pub fn new(x: i64, y: i64) -> Self {
        Self(x, y)
    }

    pub fn from(v: Vec3) -> Self {
        Self(v.x as i64, v.y as i64)
    }

    pub fn add_tile_scale(&self) -> Self {
        Self(self.0 + (TILE_SIZE as i64), self.1 + (TILE_SIZE as i64))
    }
}
impl Into<bevy::prelude::Vec2> for Coordinate {
    fn into(self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }
}

///A TCoordinate, or a "typed coordinate" is a coordinate that also includes an identifying character,
///this way the coordinate is unique, as no two objects of the same type can occupy the same space,
/// and makes for an efficient Unique Identifier both objects AND zones
/// 
/// Objects Example:
///     TCoordinate { type_char: 'T', coord: Coordinate(0, 0) }
/// Object Types: 
/// T: Tile
/// C: Collider
/// R: Editor Rect
/// P: Placeholder
/// 
/// 
/// Zones Example:
///    TCoordinate { type_char: 'F', coord: Coordinate(0, 0) }
/// Zone Types:
/// F: Foreground
/// B: Background
/// 
#[derive(Component, Reflect, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct TCoordinate {
    pub kind: EditorObjectKind,
    pub coord: Coordinate,
}
impl TCoordinate {
    pub fn new(kind: EditorObjectKind, coord: Coordinate) -> Self {
        Self {
            kind,
            coord,
        }
    }

    pub fn print(&self){
        println!("TCoordinate {{ kind: {:?}, coord: {:?} }}", self.kind, self.coord);
    }

}





impl Default for TCoordinate {
    fn default() -> Self {
        Self {
            kind: EditorObjectKind::Tile,
            coord: Coordinate(0, 0),
        }
    }
}

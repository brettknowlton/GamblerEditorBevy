use bevy::prelude::*;
use serde::{ Serialize, Deserialize };

pub mod resources;
pub mod tools;


//Helper Functions
pub fn despawn_all<T: Component>(mut commands: Commands, to_despawn: Query<Entity, With<T>>) {
    for e in to_despawn.iter() {
        commands.entity(e).despawn_recursive();
    }
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
    pub fn from(v: Vec3) -> Self {
        Self(v.x as i64, v.y as i64)
    }
}
impl Into<bevy::prelude::Vec2> for Coordinate {
    fn into(self) -> Vec2 {
        Vec2::new(self.0 as f32, self.1 as f32)
    }
}






///A TCoordinate, or a "typed coordinate" is a coordinate that also includes an identifying character,
///this way the coordinate is unique, as no two objects of the same type can occupy the same space,
/// and makes for an efficient Unique Identifier for the object
#[derive(Component, Reflect, Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct TCoordinate {
    pub type_char: char,
    pub coord: Coordinate,
}

impl TCoordinate {
    pub fn new(object_type: char, coord: Coordinate) -> Self {
        Self {
            type_char: object_type,
            coord,
        }
    }
}

impl Default for TCoordinate {
    fn default() -> Self {
        Self {
            type_char: ' ',
            coord: Coordinate(0, 0),
        }
    }
}

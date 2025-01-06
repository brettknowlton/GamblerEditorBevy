use bevy::{prelude::*, utils::HashMap};
use crate::editor::tile;

use super::Coordinate;

#[derive(Resource, Debug, Component)]
pub(crate) struct CurrentEditorObject(pub tile::TileData);
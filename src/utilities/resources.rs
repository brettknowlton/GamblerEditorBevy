use bevy::prelude::*;
use crate::Tile;


#[derive(Resource, Debug, Component)]
pub(crate) struct PlaceholderTile(pub Tile);
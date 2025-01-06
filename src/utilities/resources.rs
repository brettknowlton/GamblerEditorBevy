use bevy::prelude::*;
use crate::Tile;


#[derive(Resource, Debug, Component)]
pub(crate) struct CurrentEditorObject(pub Tile);